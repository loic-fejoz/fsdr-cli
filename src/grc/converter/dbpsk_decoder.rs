use crate::blocks::DBPskDecoder;
use crate::grc::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use crate::grc::BlockInstance;
use anyhow::Result;
use futuresdr::runtime::Flowgraph;

pub struct DBPskDecoderConverter {}

impl BlockConverter for DBPskDecoderConverter {
    fn convert(
        &self,
        _blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let blk = DBPskDecoder::new();
        let id = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
