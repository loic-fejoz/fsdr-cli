use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use futuresdr::anyhow::{bail, Result};
use futuresdr::blocks::zeromq::PubSinkBuilder;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;

pub struct ZeromqPubSinkConverter {}

impl BlockConverter for ZeromqPubSinkConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let address = blk
            .parameters
            .get("address")
            .expect("addres must be defined");
        let item_type = blk
            .parameters
            .get("type")
            .expect("item type must be defined");
        let blk = match &(item_type[..]) {
            "u8" | "uchar" => PubSinkBuilder::<u8>::new().address(address).build(),
            "byte" => PubSinkBuilder::<i8>::new().address(address).build(),
            "f32" | "float" => PubSinkBuilder::<f32>::new().address(address).build(),
            "c32" | "complex" => PubSinkBuilder::<Complex32>::new().address(address).build(),
            _ => bail!("Unhandled ZeromqPubSinkConverter Type {item_type}"),
        };

        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
