use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use crate::blocks::PatternSearch;
use futuresdr::anyhow::{bail, Result};
use futuresdr::runtime::Flowgraph;

pub struct PatternSearchConverter {}

impl BlockConverter for PatternSearchConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let values_after = Grc2FutureSdr::parameter_as_f32(blk, "values_after", "8")? as usize;
        let pattern_values = blk.parameter_or("pattern_values", "0,1");
        let pattern_values: Vec<u8> = pattern_values
            .split(",")
            .map(|x| x.parse::<u8>().unwrap())
            .collect();
        let blk = PatternSearch::<u8>::build(values_after, pattern_values);
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
