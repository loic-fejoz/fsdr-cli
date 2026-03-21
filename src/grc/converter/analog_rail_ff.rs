use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{parameter_as_f64, BlockInstance};
use crate::grc::backend::FsdrBackend;
use anyhow::Result;
use futuresdr::blocks::Apply;

pub struct AnalogRailFfConverter {}

impl<B: FsdrBackend> BlockConverter<B> for AnalogRailFfConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let low_threshold = parameter_as_f64(blk, "lo", "-1.0")? as f32;
        let max_threshold = parameter_as_f64(blk, "hi", "1.0")? as f32;
        let blk: Apply<_, f32, f32> =
            Apply::new(move |i: &f32| -> f32 { i.max(low_threshold).min(max_threshold) });
        let blk_ref = backend.add_block_runtime(blk)?;
        Ok(Box::new(DefaultPortAdapter::new(blk_ref)))
    }
}
