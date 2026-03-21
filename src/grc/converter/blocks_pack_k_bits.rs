use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{parameter_as_f32, BlockInstance};
use crate::grc::backend::FsdrBackend;
use anyhow::Result;

pub struct PackBitsConverter {}

impl<B: FsdrBackend> BlockConverter<B> for PackBitsConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let _k = parameter_as_f32(blk, "k", "8")? as usize;

        let blk_ref = backend.add_pack_bits_8to1()?;
        Ok(Box::new(DefaultPortAdapter::new(blk_ref)))
    }
}
