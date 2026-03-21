use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{parameter_as_f64, BlockInstance};
use crate::grc::backend::FsdrBackend;
use anyhow::{Context, Result};
use futuresdr::blocks::Apply;

pub struct AddConstVxConverter {}

impl<B: FsdrBackend> BlockConverter<B> for AddConstVxConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let constant = parameter_as_f64(blk, "const", "0.0")? as f32;
        let item_type = blk
            .parameters
            .get("type")
            .context("blocks_add_const_vxx: item type must be defined")?;

        let adapter: Box<dyn ConnectorAdapter<B::BlockRef>> = match &(item_type[..]) {
            "u8" => {
                let constant = constant as u8;
                let blk: Apply<_, u8, u8> = Apply::new(move |v: &u8| -> u8 { v + constant });
                let blk_ref = backend.add_block_runtime(blk)?;
                Box::new(DefaultPortAdapter::new(blk_ref))
            }
            "float" => {
                let blk: Apply<_, f32, f32> = Apply::new(move |v: &f32| -> f32 { v + constant });
                let blk_ref = backend.add_block_runtime(blk)?;
                Box::new(DefaultPortAdapter::new(blk_ref))
            }
            _ => todo!("Unhandled blocks_add_const_vxx Type {item_type}"),
        };
        Ok(adapter)
    }
}
