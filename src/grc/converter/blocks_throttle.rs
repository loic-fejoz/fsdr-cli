use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use anyhow::{Result, Context};
use futuresdr::blocks::Throttle;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;

pub struct ThrottleConverter {}

impl BlockConverter for ThrottleConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let rate = Grc2FutureSdr::parameter_as_f64(blk, "samples_per_second", "48000")?;
        let item_type = blk
            .parameters
            .get("type")
            .expect("item type must be defined");
        let blk: Box<dyn ConnectorAdapter> = match &(item_type[..]) {
            "char" => {
                let blk = Throttle::<u8>::new(rate);
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            "short" => {
                let blk = Throttle::<i16>::new(rate);
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            "float" => {
                let blk = Throttle::<f32>::new(rate);
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            "complex" => {
                let blk = Throttle::<Complex32>::new(rate);
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            _ => todo!("Unhandled blocks_throttle Type {item_type}"),
        };
        Ok(blk)
    }
}
