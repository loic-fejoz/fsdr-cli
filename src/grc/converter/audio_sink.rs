use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{parameter_as_f32, BlockInstance};
use crate::grc::backend::FsdrBackend;
use anyhow::{bail, Result};
use futuresdr::blocks::audio::AudioSink;
use futuresdr::blocks::Apply;
use futuresdr::prelude::DefaultCpuReader;

pub struct AudioSinkConverter {}

impl<B: FsdrBackend> BlockConverter<B> for AudioSinkConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let sample_rate = parameter_as_f32(blk, "samp_rate", "48000")?;
        let num_inputs = blk.parameter_or("num_inputs", "1").parse::<usize>()?;

        if num_inputs == 1 {
            let snk: AudioSink<DefaultCpuReader<f32>> = AudioSink::new(sample_rate as u32, 1);
            let snk_ref = backend.add_block_runtime(snk)?;
            Ok(Box::new(DefaultPortAdapter::new(snk_ref)))
        } else if num_inputs == 2 {
            let snk: AudioSink<DefaultCpuReader<f32>> = AudioSink::new(sample_rate as u32, 2);
            let snk_ref = backend.add_block_runtime(snk)?;

            let mono_to_stereo: Apply<_, f32, [f32; 2]> = Apply::new(|v: &f32| [*v, *v]);
            let mono_ref = backend.add_block_runtime(mono_to_stereo)?;

            backend.connect(&mono_ref, "output", &snk_ref, "input")?;

            Ok(Box::new(DefaultPortAdapter::new(mono_ref)))
        } else {
            bail!("audio_sink: Unhandled number of inputs {num_inputs}");
        }
    }
}
