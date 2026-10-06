use crate::blocks::{DifferentialDecoderU8, DifferentialEncoderU8, InvertU8};
use crate::grc::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use crate::grc::BlockInstance;
use anyhow::Result;
use futuresdr::runtime::Flowgraph;

pub struct DifferentialEncoderConverter {}

impl BlockConverter for DifferentialEncoderConverter {
    fn convert(
        &self,
        _blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let blk = DifferentialEncoderU8::new();
        let id = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}

pub struct DifferentialDecoderConverter {}

impl BlockConverter for DifferentialDecoderConverter {
    fn convert(
        &self,
        _blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let blk = DifferentialDecoderU8::new();
        let id = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}

pub struct InvertU8Converter {}

impl BlockConverter for InvertU8Converter {
    fn convert(
        &self,
        _blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let blk = InvertU8::new();
        let id = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
