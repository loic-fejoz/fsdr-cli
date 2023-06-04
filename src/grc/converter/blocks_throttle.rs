use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use futuresdr::anyhow::Result;
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
        let blk = match &(item_type[..]) {
            "char" => Throttle::<u8>::new(rate),
            "short" => Throttle::<i16>::new(rate),
            "float" => Throttle::<f32>::new(rate),
            "complex" => Throttle::<Complex32>::new(rate),
            _ => todo!("Unhandled blocks_throttle Type {item_type}"),
        };
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
