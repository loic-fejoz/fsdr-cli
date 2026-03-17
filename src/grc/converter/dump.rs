use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use anyhow::{Context, Result};
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
            .context("dump: item type must be defined")?;

        let blk: Box<dyn ConnectorAdapter> = match &(item_type[..]) {
            "float" | "f" => {
                let blk: futuresdr::blocks::Sink<_, f32> = Sink::new(|x: &f32| print!("{:e} ", *x));
                let blk = fg.add_block(blk);
                Box::new(DefaultPortAdapter::new(blk.into()))
            }
            "u8" => {
                let blk: futuresdr::blocks::Sink<_, u8> = Sink::new(|x: &u8| print!("{:02x} ", *x));
                let blk = fg.add_block(blk);
                Box::new(DefaultPortAdapter::new(blk.into()))
            }
            _ => todo!("Unhandled dump of Type {item_type}"),
        };
        Ok(blk)
    }
}
