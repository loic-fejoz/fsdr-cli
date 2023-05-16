use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use futuresdr::anyhow::Result;
use futuresdr::blocks::AgcBuilder;
use futuresdr::runtime::Flowgraph;

pub struct AnalogAgcXxConverter {}

impl BlockConverter for AnalogAgcXxConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let reference = Grc2FutureSdr::parameter_as_f64(blk, "reference", "1.0")? as f32;
        let max_gain = Grc2FutureSdr::parameter_as_f64(blk, "max_gain", "10.0")? as f32;
        let rate = Grc2FutureSdr::parameter_as_f64(blk, "rate", "10.0")? as f32;
        let item_type = blk
            .parameters
            .get("type")
            .expect("item type must be defined");

        let blk = match &(item_type[..]) {
            "float" => AgcBuilder::<f32>::new()
                .squelch(0.0)
                .reference_power(reference)
                .max_gain(max_gain)
                .adjustment_rate(rate)
                .build(),
            _ => todo!("Unhandled analog_agc_xx Type {item_type}"),
        };
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
