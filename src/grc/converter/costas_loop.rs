use crate::blocks::CostasLoopCc;
use crate::grc::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use crate::grc::BlockInstance;
use anyhow::{Context, Result};
use futuresdr::runtime::Flowgraph;

pub struct CostasLoopConverter {}

impl BlockConverter for CostasLoopConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let loop_bw: f32 = blk
            .parameters
            .get("w")
            .or_else(|| blk.parameters.get("loop_bw"))
            .map(|s| s.parse::<f32>().context("failed to parse loop_bw/w"))
            .transpose()?
            .unwrap_or(0.05);

        let damping: f32 = blk
            .parameters
            .get("q")
            .or_else(|| blk.parameters.get("damping"))
            .map(|s| s.parse::<f32>().context("failed to parse damping/q"))
            .transpose()?
            .unwrap_or(0.707);

        let blk = CostasLoopCc::new(loop_bw, damping);
        let id = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
