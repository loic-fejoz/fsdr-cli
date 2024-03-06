use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use fsdr_blocks::futuresdr::anyhow::Result;
use fsdr_blocks::futuresdr::blocks::Sink;
use fsdr_blocks::futuresdr::num_complex::Complex32;
use fsdr_blocks::futuresdr::runtime::Flowgraph;

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
            "u8" => Sink::new(|x: &u8| print!("{:02x} ", *x)),
            "f" | "float" => Sink::new(|x: &f32| print!("{:e} ", *x)),
            "c" => Sink::new(|x: &Complex32| print!("({:e}, {:e})", x.re, x.im)),
            _ => todo!("Unhandled dump of Type {item_type}"),
        };
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
