use super::super::converter_helper::{
    eval_expr_str, BlockConverter, ConnectorAdapter, DefaultPortAdapter,
};
use super::BlockInstance;
use crate::blocks::AfcFf;
use anyhow::Result;
use futuresdr::runtime::Flowgraph;

pub struct AnalogAfcFfConverter {}

impl BlockConverter for AnalogAfcFfConverter {
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

        let limit = if let Some(l) = blk.parameters.get("limit") {
            eval_expr_str(l)?
        } else {
            0.0
        };

        let block = AfcFf::new(alpha, limit);
        let id = fg.add(block)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
