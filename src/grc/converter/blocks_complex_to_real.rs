use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use crate::grc::backend::FsdrBackend;
use anyhow::Result;

pub struct RealpartCfConverter {}

impl<B: FsdrBackend> BlockConverter<B> for RealpartCfConverter {
    fn convert(
        &self,
        _blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let blk_ref = backend.add_complex_to_real()?;
        Ok(Box::new(DefaultPortAdapter::new(blk_ref)))
    }
}
