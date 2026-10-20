use crate::beacon::BeaconTracker;
use futures::channel::mpsc;
use futures::StreamExt;
use futuresdr::runtime::{BlockId, FlowgraphHandle, Pmt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Debug, Clone)]
pub struct ChannelShifterInfo {
    pub channel_id: String,
    pub rx_shifter_id: Option<BlockId>,
    pub tx_shifter_id: Option<BlockId>,
    pub sdr_sample_rate: f32,
    pub rx_center_freq: f64,
    pub tx_center_freq: f64,
    pub current_frequency: u64,
}

#[derive(Debug, Clone)]
pub enum DspControlMsg {
    SetFrequency { channel: String, frequency: u64 },
    SetRxFrequency { channel: String, frequency: u64 },
    SetTxFrequency { channel: String, frequency: u64 },
    SetOffset { channel: String, offset: i64 },
}

/// Manages and propagates VFO frequency and offset changes directly into running
/// FutureSDR DSP flowgraphs (specifically targeting `blocks_freqshift_cc` / `decimating_shift_addition_cc`).
#[derive(Clone)]
pub struct DspFrequencyController {
    shifters: Arc<RwLock<HashMap<String, ChannelShifterInfo>>>,
    tx: mpsc::UnboundedSender<DspControlMsg>,
    rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<DspControlMsg>>>>,
}

impl Default for DspFrequencyController {
    fn default() -> Self {
        Self::new()
    }
}

impl DspFrequencyController {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded();
        Self {
            shifters: Arc::new(RwLock::new(HashMap::new())),
            tx,
            rx: Arc::new(Mutex::new(Some(rx))),
        }
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<DspControlMsg> {
        self.tx.clone()
    }

    pub fn register_channel(&self, info: ChannelShifterInfo) {
        if let Ok(mut map) = self.shifters.write() {
            if let Some(existing) = map.get_mut(&info.channel_id) {
                if info.rx_shifter_id.is_some() {
                    existing.rx_shifter_id = info.rx_shifter_id;
                }
                if info.tx_shifter_id.is_some() {
                    existing.tx_shifter_id = info.tx_shifter_id;
                }
                existing.sdr_sample_rate = info.sdr_sample_rate;
                existing.rx_center_freq = info.rx_center_freq;
                existing.tx_center_freq = info.tx_center_freq;
                if info.current_frequency > 0 {
                    existing.current_frequency = info.current_frequency;
                }
            } else {
                map.insert(info.channel_id.clone(), info);
            }
        }
    }

    pub fn update_rx_shifter(&self, channel_id: &str, block_id: BlockId) {
        if let Ok(mut map) = self.shifters.write() {
            if let Some(info) = map.get_mut(channel_id) {
                info.rx_shifter_id = Some(block_id);
            }
        }
    }

    pub fn update_tx_shifter(&self, channel_id: &str, block_id: BlockId) {
        if let Ok(mut map) = self.shifters.write() {
            if let Some(info) = map.get_mut(channel_id) {
                info.tx_shifter_id = Some(block_id);
            }
        }
    }

    pub fn set_frequency(&self, channel: &str, frequency: u64) {
        let _ = self.tx.unbounded_send(DspControlMsg::SetFrequency {
            channel: channel.to_string(),
            frequency,
        });
    }

    pub fn set_rx_frequency(&self, channel: &str, frequency: u64) {
        let _ = self.tx.unbounded_send(DspControlMsg::SetRxFrequency {
            channel: channel.to_string(),
            frequency,
        });
    }

    pub fn set_tx_frequency(&self, channel: &str, frequency: u64) {
        let _ = self.tx.unbounded_send(DspControlMsg::SetTxFrequency {
            channel: channel.to_string(),
            frequency,
        });
    }

    pub fn set_offset(&self, channel: &str, offset: i64) {
        let _ = self.tx.unbounded_send(DspControlMsg::SetOffset {
            channel: channel.to_string(),
            offset,
        });
    }

    /// Calculate normalized frequency shift rate for RX downconversion.
    /// `shift = -(freq - rx_center - delta_f) / sample_rate`
    pub fn calc_rx_shift(freq: u64, rx_center: f64, delta_f: i64, sample_rate: f32) -> f32 {
        let offset = (freq as i64) - (rx_center as i64) - delta_f;
        -(offset as f32) / sample_rate
    }

    /// Calculate normalized frequency shift rate for TX upconversion.
    /// `shift = +(freq - tx_center) / sample_rate`
    pub fn calc_tx_shift(freq: u64, tx_center: f64, sample_rate: f32) -> f32 {
        let offset = (freq as i64) - (tx_center as i64);
        (offset as f32) / sample_rate
    }

    /// Run the dispatcher loop that listens for DSP frequency update messages and
    /// propagates them asynchronously to the running FutureSDR flowgraph blocks.
    pub async fn run_dispatcher(
        self,
        fg_handle: FlowgraphHandle,
        beacon_tracker: BeaconTracker,
        verbose: bool,
    ) {
        let mut rx = {
            let mut lock = self.rx.lock().unwrap();
            match lock.take() {
                Some(r) => r,
                None => return,
            }
        };

        let mut ticker = tokio::time::interval(std::time::Duration::from_millis(500));
        let mut last_delta_f = beacon_tracker.delta_f();

        loop {
            tokio::select! {
                maybe_msg = rx.next() => {
                    match maybe_msg {
                        Some(msg) => {
                            let delta_f = beacon_tracker.delta_f();
                            self.handle_dsp_msg(msg, delta_f, &fg_handle, verbose).await;
                        }
                        None => break,
                    }
                }
                _ = ticker.tick() => {
                    let current_delta_f = beacon_tracker.delta_f();
                    if current_delta_f != last_delta_f {
                        last_delta_f = current_delta_f;
                        self.sync_beacon_drift(current_delta_f, &fg_handle, verbose).await;
                    }
                }
            }
        }
    }

    async fn handle_dsp_msg(
        &self,
        msg: DspControlMsg,
        delta_f: i64,
        fg_handle: &FlowgraphHandle,
        verbose: bool,
    ) {
        match msg {
            DspControlMsg::SetFrequency { channel, frequency } => {
                let info_opt = {
                    let mut shifters = self.shifters.write().unwrap();
                    if let Some(info) = shifters.get_mut(&channel) {
                        info.current_frequency = frequency;
                        Some(info.clone())
                    } else {
                        None
                    }
                };

                if let Some(info) = info_opt {
                    let s_rx = Self::calc_rx_shift(
                        frequency,
                        info.rx_center_freq,
                        delta_f,
                        info.sdr_sample_rate,
                    );
                    let s_tx =
                        Self::calc_tx_shift(frequency, info.tx_center_freq, info.sdr_sample_rate);

                    if let Some(blk_id) = info.rx_shifter_id {
                        let res = Self::call_shifter_rate(fg_handle, blk_id, s_rx).await;
                        if verbose {
                            println!(
                                "[fsdr-hub][dsp] Channel '{}' RX shifter (block {:?}): freq={} Hz -> shift={:+.6}, res={:?}",
                                channel, blk_id, frequency, s_rx, res
                            );
                        }
                    }

                    if let Some(blk_id) = info.tx_shifter_id {
                        let res = Self::call_shifter_rate(fg_handle, blk_id, s_tx).await;
                        if verbose {
                            println!(
                                "[fsdr-hub][dsp] Channel '{}' TX shifter (block {:?}): freq={} Hz -> shift={:+.6}, res={:?}",
                                channel, blk_id, frequency, s_tx, res
                            );
                        }
                    }
                }
            }
            DspControlMsg::SetRxFrequency { channel, frequency } => {
                let info_opt = {
                    let mut shifters = self.shifters.write().unwrap();
                    if let Some(info) = shifters.get_mut(&channel) {
                        info.current_frequency = frequency;
                        Some(info.clone())
                    } else {
                        None
                    }
                };

                if let Some(info) = info_opt {
                    let s_rx = Self::calc_rx_shift(
                        frequency,
                        info.rx_center_freq,
                        delta_f,
                        info.sdr_sample_rate,
                    );
                    if let Some(blk_id) = info.rx_shifter_id {
                        let res = Self::call_shifter_rate(fg_handle, blk_id, s_rx).await;
                        if verbose {
                            println!(
                                "[fsdr-hub][dsp] Channel '{}' RX shifter (block {:?}): rx_freq={} Hz -> shift={:+.6}, res={:?}",
                                channel, blk_id, frequency, s_rx, res
                            );
                        }
                    }
                }
            }
            DspControlMsg::SetTxFrequency { channel, frequency } => {
                let info_opt = self.shifters.read().unwrap().get(&channel).cloned();
                if let Some(info) = info_opt {
                    let s_tx =
                        Self::calc_tx_shift(frequency, info.tx_center_freq, info.sdr_sample_rate);
                    if let Some(blk_id) = info.tx_shifter_id {
                        let res = Self::call_shifter_rate(fg_handle, blk_id, s_tx).await;
                        if verbose {
                            println!(
                                "[fsdr-hub][dsp] Channel '{}' TX shifter (block {:?}): tx_freq={} Hz -> shift={:+.6}, res={:?}",
                                channel, blk_id, frequency, s_tx, res
                            );
                        }
                    }
                }
            }
            DspControlMsg::SetOffset { channel, offset } => {
                let info_opt = self.shifters.read().unwrap().get(&channel).cloned();
                if let Some(info) = info_opt {
                    let s_rx = -(offset as f32) / info.sdr_sample_rate;
                    let s_tx = (offset as f32) / info.sdr_sample_rate;

                    if let Some(blk_id) = info.rx_shifter_id {
                        let res = Self::call_shifter_rate(fg_handle, blk_id, s_rx).await;
                        if verbose {
                            println!(
                                "[fsdr-hub][dsp] Channel '{}' RX shifter (block {:?}): offset={} Hz -> shift={:+.6}, res={:?}",
                                channel, blk_id, offset, s_rx, res
                            );
                        }
                    }

                    if let Some(blk_id) = info.tx_shifter_id {
                        let res = Self::call_shifter_rate(fg_handle, blk_id, s_tx).await;
                        if verbose {
                            println!(
                                "[fsdr-hub][dsp] Channel '{}' TX shifter (block {:?}): offset={} Hz -> shift={:+.6}, res={:?}",
                                channel, blk_id, offset, s_tx, res
                            );
                        }
                    }
                }
            }
        }
    }

    pub async fn call_shifter_rate(
        fg_handle: &FlowgraphHandle,
        blk_id: BlockId,
        rate: f32,
    ) -> Result<Pmt, futuresdr::runtime::Error> {
        match fg_handle.call(blk_id, "set_rate", Pmt::F32(rate)).await {
            Ok(Pmt::Ok) => Ok(Pmt::Ok),
            _ => {
                fg_handle
                    .call(blk_id, "set_frequency", Pmt::F32(rate))
                    .await
            }
        }
    }

    async fn sync_beacon_drift(
        &self,
        current_delta_f: i64,
        fg_handle: &FlowgraphHandle,
        verbose: bool,
    ) {
        let list: Vec<ChannelShifterInfo> =
            { self.shifters.read().unwrap().values().cloned().collect() };

        for info in list {
            if let Some(blk_id) = info.rx_shifter_id {
                if info.current_frequency > 0 {
                    let s_rx = Self::calc_rx_shift(
                        info.current_frequency,
                        info.rx_center_freq,
                        current_delta_f,
                        info.sdr_sample_rate,
                    );
                    let _ = Self::call_shifter_rate(fg_handle, blk_id, s_rx).await;
                    if verbose {
                        println!(
                            "[fsdr-hub][beacon] Doppler correction updated channel '{}' RX shifter (block {:?}): delta_f={} Hz, shift={:+.6}",
                            info.channel_id, blk_id, current_delta_f, s_rx
                        );
                    }
                }
            }
        }
    }
}
