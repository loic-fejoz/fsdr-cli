use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{BufferSize, SampleRate, StreamConfig};
use fsdr_blocks::futuresdr::anyhow::{bail, Result};
use fsdr_blocks::futuresdr::blocks::audio::AudioSink;
use fsdr_blocks::futuresdr::blocks::ApplyNM;
use fsdr_blocks::futuresdr::runtime::Flowgraph;

// #[derive(Clone, Copy)]
// pub struct AudioSinkPortAdapter {
//     blk_1: usize,
//     sink_blk: usize,
// }

// impl AudioSinkPortAdapter {
//     pub fn new(blk_1: usize, sink_blk: usize) -> AudioSinkPortAdapter {
//         AudioSinkPortAdapter { blk_1, sink_blk }
//     }
// }

// impl ConnectorAdapter for AudioSinkPortAdapter {
//     fn adapt_input_port(&self, port_name: &str) -> Result<(usize, &str)> {
//         match port_name {
//             "0" => Ok((self.blk, "in")),
//             "in" => Ok((self.blk, "in")),
//             _ => bail!("Unknown input port name {port_name}"),
//         }
//     }

//     fn adapt_output_port(&self, port_name: &str) -> Result<(usize, &str)> {
//         match port_name {
//             "0" => Ok((self.blk, "out")),
//             "out" => Ok((self.blk, "out")),
//             _ => bail!("Unknown output port name {port_name}"),
//         }
//     }
// }

pub struct AudioSinkConverter {}

impl BlockConverter for AudioSinkConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let sample_rate = Grc2FutureSdr::parameter_as_f64(blk, "samp_rate", "48000")? as u32;
        let num_inputs = Grc2FutureSdr::parameter_as_f64(blk, "num_inputs", "1")? as u16;

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
                    && c.min_sample_rate().0 >= sample_rate
                    && sample_rate <= c.max_sample_rate().0
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
                num_inputs
            },
            sample_rate: SampleRate(sample_rate),
            buffer_size: BufferSize::Default,
        };

        let blk = AudioSink::new(sample_rate, config.channels);
        let blk = fg.add_block(blk);

        match (num_inputs, config.channels) {
            (1, 1) => {
                let blk = DefaultPortAdapter::new(blk);
                let blk = Box::new(blk);
                Ok(blk)
            }
            (1, 2) => {
                // Need to duplicate & interlave inputs
                let mono_to_stereo =
                    ApplyNM::<_, _, _, 1, 2>::new(move |v: &[f32], d: &mut [f32]| {
                        d[0] = v[0]; // left
                        d[1] = v[0]; // right
                    });
                let mono_to_stereo = fg.add_block(mono_to_stereo);
                fg.connect_stream(mono_to_stereo, "out", blk, "in")?;

                let adapter = DefaultPortAdapter::new(mono_to_stereo);
                let adapter = Box::new(adapter);
                Ok(adapter)
            }
            // (2, 1) => {
            //     // TODO: Need to combine a stereo inputs into a mono outputs
            // },
            // (2, 2) => {
            //     // TODO: Need to combine a stereo inputs into an interleaved outputs
            // }
            _ => {
                let expected_channels = config.channels;
                bail!("Do not yet handle audio sink with num_inputs={num_inputs} and expected channels={expected_channels}.");
            }
        }
    }
}
