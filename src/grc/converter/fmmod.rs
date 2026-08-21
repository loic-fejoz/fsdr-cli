use crate::blocks::FmModFc;
use crate::grc::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use crate::grc::BlockInstance;
use anyhow::Result;
use futuresdr::runtime::Flowgraph;

pub struct FmModFcConverter {}

impl BlockConverter for FmModFcConverter {
    fn convert(
        &self,
        _blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let blk = FmModFc::new();
        let id = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
