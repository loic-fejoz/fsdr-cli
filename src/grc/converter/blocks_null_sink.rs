use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use anyhow::{Result, Context};
use futuresdr::blocks::NullSink;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;

pub struct NullSinkConverter {}

impl BlockConverter for NullSinkConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let item_type = blk
            .parameters
            .get("type")
            .expect("item type must be defined");
        let blk: Box<dyn ConnectorAdapter> = match &(item_type[..]) {
            "char" => {
                let blk = NullSink::<u8>::new();
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            "short" => {
                let blk = NullSink::<i16>::new();
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            "float" => {
                let blk = NullSink::<f32>::new();
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            "complex" => {
                let blk = NullSink::<Complex32>::new();
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            _ => todo!("Unhandled blocks_null_sink Type {item_type}"),
        };
        Ok(blk)
    }
}
