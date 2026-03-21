use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use crate::grc::backend::FsdrBackend;
use anyhow::{Context, Result};

pub struct DumpConverter {}

impl<B: FsdrBackend> BlockConverter<B> for DumpConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let item_type = blk
            .parameters
            .get("type")
            .context("dump: item type must be defined")?;

        let blk_ref = match &(item_type[..]) {
            "float" | "f" => backend.add_dump_f32()?,
            "u8" => backend.add_dump_u8()?,
            "c" | "complex" | "c32" => backend.add_dump_c32()?,
            _ => todo!("Unhandled dump of Type {item_type}"),
        };
        Ok(Box::new(DefaultPortAdapter::new(blk_ref)))
    }
}
