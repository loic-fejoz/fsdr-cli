use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{parameter_as_f32, BlockInstance};
use crate::grc::backend::FsdrBackend;
use anyhow::{Context, Result};

pub struct MulConstVxConverter {}

impl<B: FsdrBackend> BlockConverter<B> for MulConstVxConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let constant = parameter_as_f32(blk, "const", "0.0")?;
        let item_type = blk
            .parameters
            .get("type")
            .context("blocks_multiply_const_vxx: item type must be defined")?;

        let adapter: Box<dyn ConnectorAdapter<B::BlockRef>> = match &(item_type[..]) {
            "float" => {
                let blk_ref = backend.add_multiply_const_f32(constant)?;
                Box::new(DefaultPortAdapter::new(blk_ref))
            }
            _ => todo!("Unhandled blocks_multiply_const_vxx Type {item_type}"),
        };
        Ok(adapter)
    }
}
