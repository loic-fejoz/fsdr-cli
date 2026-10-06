use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use crate::blocks::CtcssGenerator;
use anyhow::Result;
use futuresdr::runtime::Flowgraph;

pub struct CtcssGeneratorConverter {}

impl BlockConverter for CtcssGeneratorConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let sample_rate = Grc2FutureSdr::parameter_as_f64(blk, "sample_rate", "48000")? as f32;
        let blk = CtcssGenerator::new(sample_rate);
        let blk = fg.add(blk)?.id();
        let adapter = DefaultPortAdapter::new(blk);
        Ok(Box::new(adapter))
    }
}
