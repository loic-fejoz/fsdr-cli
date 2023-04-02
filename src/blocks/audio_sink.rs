use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::BufferSize;
use cpal::SampleRate;
use cpal::Stream;
use cpal::StreamConfig;
use futures::channel::mpsc;
use futures::SinkExt;

use futuresdr::anyhow::Result;
use futuresdr::runtime::Block;
use futuresdr::runtime::BlockMeta;
use futuresdr::runtime::BlockMetaBuilder;
use futuresdr::runtime::Kernel;
use futuresdr::runtime::MessageIo;
use futuresdr::runtime::MessageIoBuilder;
use futuresdr::runtime::StreamIo;
use futuresdr::runtime::StreamIoBuilder;
use futuresdr::runtime::WorkIo;

/// Audio Sink.
#[allow(clippy::type_complexity)]
pub struct AudioSink {
    sample_rate: u32,
    channels: u16,
    stream: Option<Stream>,
    min_buffer_size: usize,
    vec: Vec<f32>,
    tx: Option<mpsc::Sender<Vec<f32>>>,
}

// cpal::Stream is !Send
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for AudioSink {}

const QUEUE_SIZE: usize = 5;
const STANDARD_RATES: [u32; 4] = [24000, 44100, 48000, 96000];

impl AudioSink {
    /// Create AudioSink block
    #[allow(clippy::new_ret_no_self)]
    pub fn new(sample_rate: u32, channels: u16) -> Block {
        Block::new(
            BlockMetaBuilder::new("AudioSink").build(),
            StreamIoBuilder::new().add_input::<f32>("in").build(),
            MessageIoBuilder::new().build(),
            AudioSink {
                sample_rate,
                channels,
                stream: None,
                min_buffer_size: 2048,
                vec: Vec::new(),
                tx: None,
            },
        )
    }
    /// Get default sample rate
    pub fn default_sample_rate() -> Option<u32> {
        Some(
            cpal::default_host()
                .default_output_device()?
                .default_output_config()
                .ok()?
                .sample_rate()
                .0,
        )
    }
    /// Get supported sample rates
    pub fn supported_sample_rates() -> Vec<u32> {
        if let Some(d) = cpal::default_host().default_output_device() {
            if let Ok(configs) = d.supported_output_configs() {
                let mut v = Vec::new();
                for c in configs {
                    // println!("{:?}", c);
                    let min = c.min_sample_rate().0;
                    let max = c.max_sample_rate().0;
                    if min >= 10000 {
                        v.push(min);
                    }
                    if max >= 10000 {
                        v.push(max);
                    }

                    v.extend(STANDARD_RATES.iter().filter(|x| *x >= &min && *x <= &max));
                }
                v.sort();
                v.dedup();
                return v;
            }
        }
        Vec::new()
    }
}

#[doc(hidden)]
#[async_trait]
impl Kernel for AudioSink {
    async fn init(
        &mut self,
        _s: &mut StreamIo,
        _m: &mut MessageIo<Self>,
        _b: &mut BlockMeta,
    ) -> Result<()> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");

        let mut actual_channels: Option<u16> = None;
        // On Windows there is an issue in cpal with
        // shared devices, if the requested configuration
        // does not match the device configuration.
        // https://github.com/RustAudio/cpal/issues/593
        // On MS/Windows, the channels might be greater than the one needed.
        // So find a compatible configuration, esp. (sample_rate, channels)
        if let Ok(configs) = device.supported_output_configs() {
            for c in configs {
                // println!("{:?}", c);
                if actual_channels.is_none()
                    && c.min_sample_rate().0 >= self.sample_rate
                    && self.sample_rate <= c.max_sample_rate().0
                {
                    actual_channels = Some(c.channels());
                    break;
                }
            }
        }

        let config = StreamConfig {
            channels: if let Some(actual_channels) = actual_channels {
                actual_channels
            } else {
                self.channels
            },
            sample_rate: SampleRate(self.sample_rate),
            buffer_size: BufferSize::Default,
        };
        let duplicate_time = (config.channels / self.channels) as usize;
        let (tx, mut rx) = mpsc::channel(QUEUE_SIZE);
        let mut iter: Option<Vec<f32>> = None;

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut i = 0;

                    while let Some(mut v) =
                        iter.take().or_else(|| rx.try_next().ok().and_then(|x| x))
                    {
                        let n = std::cmp::min(v.len(), data.len() / duplicate_time - i);
                        if duplicate_time == 1 {
                            // Case where channels was compatible
                            data[i..i + n].copy_from_slice(&v[..n]);
                        } else {
                            // Case where we need to duplicate inputs so as to match channels number
                            for (target, source) in
                                data.chunks_exact_mut(duplicate_time).zip(v.iter())
                            {
                                for t in target.iter_mut().take(duplicate_time) {
                                    *t = *source;
                                }
                            }
                        }

                        i += n;

                        if n < v.len() {
                            iter = Some(v.split_off(n));
                            return;
                        } else if i == data.len() {
                            return;
                        }
                    }
                },
                move |err| {
                    panic!("cpal stream error {err:?}");
                },
            )
            .expect("could not build output stream");
        // On Windows there is an issue in cpal with
        // shared devices, if the requested configuration
        // does not match the device configuration.
        // https://github.com/RustAudio/cpal/issues/593

        stream.play()?;

        self.tx = Some(tx);
        self.stream = Some(stream);

        Ok(())
    }

    async fn deinit(
        &mut self,
        _s: &mut StreamIo,
        _m: &mut MessageIo<Self>,
        _b: &mut BlockMeta,
    ) -> Result<()> {
        for _ in 0..QUEUE_SIZE {
            let _ = self.tx.as_mut().unwrap().send(Vec::new()).await;
        }
        Ok(())
    }

    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i = sio.input(0).slice::<f32>();
        self.vec.extend_from_slice(i);

        if self.vec.len() >= self.min_buffer_size {
            self.tx
                .as_mut()
                .unwrap()
                .send(std::mem::take(&mut self.vec))
                .await?;
        }

        sio.input(0).consume(i.len());

        if sio.input(0).finished() {
            io.finished = true;
        }

        Ok(())
    }
}