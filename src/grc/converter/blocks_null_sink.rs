use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use crate::grc::backend::FsdrBackend;
use anyhow::Result;
use futuresdr::blocks::NullSink;
use futuresdr::num_complex::Complex32;

pub struct NullSinkConverter {}

impl<B: FsdrBackend> BlockConverter<B> for NullSinkConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let item_type = blk
            .parameters
            .get("type")
            .expect("item type must be defined");
        let blk_ref = match &(item_type[..]) {
            "char" | "u8" => {
                let blk = NullSink::<u8>::new();
                backend.add_block_runtime(blk)?
            }
            "short" | "i16" => {
                let blk = NullSink::<i16>::new();
                backend.add_block_runtime(blk)?
            }
            "float" | "f32" => {
                let blk = NullSink::<f32>::new();
                backend.add_block_runtime(blk)?
            }
            "complex" | "c32" => {
                let blk = NullSink::<Complex32>::new();
                backend.add_block_runtime(blk)?
            }
            _ => todo!("Unhandled blocks_null_sink Type {item_type}"),
        };
        Ok(Box::new(DefaultPortAdapter::new(blk_ref)))
    }
}
