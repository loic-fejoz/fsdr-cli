use crate::blocks::Mono2StereoS16;
use crate::grc::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use crate::grc::BlockInstance;
use anyhow::Result;
use futuresdr::runtime::Flowgraph;

pub struct Mono2StereoS16Converter {}

impl BlockConverter for Mono2StereoS16Converter {
    fn convert(
        &self,
        _blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let blk = Mono2StereoS16::new();
        let id = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
