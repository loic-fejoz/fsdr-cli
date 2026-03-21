use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{parameter_as_f32, BlockInstance};
use crate::grc::backend::FsdrBackend;
use anyhow::Result;

pub struct DsbConverter {}

impl<B: FsdrBackend> BlockConverter<B> for DsbConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let q_value = parameter_as_f32(blk, "q_value", "0.0")?;
        let blk_ref = backend.add_dsb_fc(q_value)?;
        Ok(Box::new(DefaultPortAdapter::new(blk_ref)))
    }
}
