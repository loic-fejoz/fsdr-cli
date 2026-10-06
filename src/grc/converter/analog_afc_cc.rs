use super::super::converter_helper::{
    eval_expr_str, BlockConverter, ConnectorAdapter, DefaultPortAdapter,
};
use super::{BlockInstance, Grc2FutureSdr};
use crate::blocks::AfcCc;
use anyhow::Result;
use futuresdr::runtime::Flowgraph;

pub struct AnalogAfcCcConverter {}

impl BlockConverter for AnalogAfcCcConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let alpha = if let Some(a) = blk.parameters.get("alpha") {
            eval_expr_str(a)?
        } else {
            0.001
        };

        let samp_rate = Grc2FutureSdr::parameter_as_f64(blk, "samp_rate", "2400000.0")? as f32;

        let max_freq_hz = if let Some(f) = blk.parameters.get("max_freq") {
            eval_expr_str(f)?
        } else {
            samp_rate / 4.0
        };

        let max_freq_rad = 2.0 * std::f32::consts::PI * (max_freq_hz / samp_rate);

        let block = AfcCc::new(alpha, max_freq_rad);
        let id = fg.add(block)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
