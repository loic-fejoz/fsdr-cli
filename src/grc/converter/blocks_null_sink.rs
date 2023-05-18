use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use futuresdr::anyhow::Result;
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
        let blk = match &(item_type[..]) {
            "char" => NullSink::<u8>::new(),
            "short" => NullSink::<i16>::new(),
            "float" => NullSink::<f32>::new(),
            "complex" => NullSink::<Complex32>::new(),
            _ => todo!("Unhandled blocks_null_sink Type {item_type}"),
        };
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
