use crate::blocks::{VaricodeDecoder, VaricodeEncoder};
use crate::grc::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use crate::grc::BlockInstance;
use anyhow::Result;
use futuresdr::runtime::Flowgraph;

pub struct VaricodeDecoderConverter {}

impl BlockConverter for VaricodeDecoderConverter {
    fn convert(
        &self,
        _blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let blk = VaricodeDecoder::new();
        let id = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}

pub struct VaricodeEncoderConverter {}

impl BlockConverter for VaricodeEncoderConverter {
    fn convert(
        &self,
        _blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let blk = VaricodeEncoder::new();
        let id = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
