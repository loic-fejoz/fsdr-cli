use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use fsdr_blocks::futuresdr::anyhow::Result;
use fsdr_blocks::futuresdr::blocks::Apply;
use fsdr_blocks::futuresdr::runtime::Flowgraph;

pub struct AnalogRailFfConverter {}

impl BlockConverter for AnalogRailFfConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let low_threshold = Grc2FutureSdr::parameter_as_f64(blk, "lo", "-1.0")? as f32;
        let max_threshold = Grc2FutureSdr::parameter_as_f64(blk, "hi", "1.0")? as f32;
        let blk = Apply::new(move |i: &f32| -> f32 { i.max(low_threshold).min(max_threshold) });
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
