use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use futuresdr::anyhow::Result;
use futuresdr::blocks::Sink;
use futuresdr::runtime::Flowgraph;

pub struct DumpConverter {}

impl BlockConverter for DumpConverter {
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
            "float" => Sink::new(|x: &u8| print!("{:02x} ", *x)),
            "u8" => Sink::new(|x: &f32| print!("{:e} ", *x)),
            _ => todo!("Unhandled dump of Type {item_type}"),
        };
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
