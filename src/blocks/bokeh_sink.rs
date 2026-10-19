use anyhow::Result;
use futures::channel::mpsc;
use futuresdr::runtime::dev::prelude::*;
use std::time::Instant;

/// Broadcast channel sender for Bokeh FFT vectors
pub type BokehSender = mpsc::UnboundedSender<Vec<f32>>;

static GLOBAL_BOKEH_TX: std::sync::RwLock<Option<BokehSender>> = std::sync::RwLock::new(None);

pub fn set_global_bokeh_sender(tx: BokehSender) {
    if let Ok(mut lock) = GLOBAL_BOKEH_TX.write() {
        *lock = Some(tx);
    }
}

pub fn get_global_bokeh_sender() -> Option<BokehSender> {
    GLOBAL_BOKEH_TX.read().ok().and_then(|lock| lock.clone())
}

/// Bokeh Spectrum Sink: collects FFT bins into frames and sends to WebSocket clients at target fps
#[derive(Block)]
pub struct BokehSpectrumSink<I: CpuBufferReader<Item = f32> = DefaultCpuReader<f32>> {
    fft_size: usize,
    fps: u32,
    buffer: Vec<f32>,
    last_send: Instant,
    sender: Option<BokehSender>,
    #[input]
    input: I,
}

impl BokehSpectrumSink<DefaultCpuReader<f32>> {
    pub fn new(fft_size: usize, fps: u32) -> Self {
        Self {
            fft_size: fft_size.max(64),
            fps: fps.max(1),
            buffer: Vec::with_capacity(fft_size),
            last_send: Instant::now(),
            sender: get_global_bokeh_sender(),
            input: DefaultCpuReader::default(),
        }
    }
}

#[doc(hidden)]
impl<I> Kernel for BokehSpectrumSink<I>
where
    I: CpuBufferReader<Item = f32>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let in_slice = self.input.slice();
        let m = in_slice.len();

        if m > 0 {
            if self.sender.as_ref().map(|s| s.is_closed()).unwrap_or(true) {
                self.sender = get_global_bokeh_sender();
            }

            let min_interval = std::time::Duration::from_millis(1000 / self.fps as u64);

            for &val in in_slice {
                self.buffer.push(val);
                if self.buffer.len() >= self.fft_size {
                    if self.last_send.elapsed() >= min_interval {
                        if let Some(ref tx) = self.sender {
                            let _ = tx.unbounded_send(self.buffer.clone());
                        }
                        self.last_send = Instant::now();
                    }
                    self.buffer.clear();
                }
            }
            self.input.consume(m);
        }

        if !self.input.slice().is_empty() {
            io.call_again = true;
        }

        if self.input.slice().is_empty() && self.input.finished() {
            io.finished = true;
        }

        Ok(())
    }
}
