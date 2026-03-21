use futuresdr::blocks::Apply;

use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{parameter_as_f64, BlockInstance};
use crate::grc::backend::FsdrBackend;
use anyhow::Result;

pub struct AnalogFmDeemphConverter {}

impl<B: FsdrBackend> BlockConverter<B> for AnalogFmDeemphConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let sample_rate = parameter_as_f64(blk, "samp_rate", "48000")? as f32;
        let tau = parameter_as_f64(blk, "tau", "50e-6")? as f32;
        let dt = 1.0 / sample_rate;
        let alpha = dt / (tau + dt);
        let mut last = 0.0; // store sample x[n-1]
        let blk: Apply<_, f32, f32> = Apply::new(move |v: &f32| -> f32 {
            let r = alpha * v + (1.0 - alpha) * last; //this is the simplest IIR LPF
            last = r;
            r
        });
        let blk_ref = backend.add_block_runtime(blk)?;
        Ok(Box::new(DefaultPortAdapter::new(blk_ref)))
    }
}
