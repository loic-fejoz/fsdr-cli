use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use crate::grc::backend::FsdrBackend;
use anyhow::Result;

pub struct DigitalBinarySlicerConverter {}

impl<B: FsdrBackend> BlockConverter<B> for DigitalBinarySlicerConverter {
    fn convert(
        &self,
        _blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let blk_ref = backend.add_binary_slicer_fb()?;
        Ok(Box::new(DefaultPortAdapter::new(blk_ref)))
    }
}
