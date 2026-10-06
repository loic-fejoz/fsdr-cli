use crate::blocks::DcBlockFf;
use crate::grc::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use crate::grc::BlockInstance;
use anyhow::{Context, Result};
use futuresdr::runtime::Flowgraph;

pub struct DcBlockFfConverter {}

impl BlockConverter for DcBlockFfConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let r: f32 = blk
            .parameters
            .get("r")
            .map(|s| s.parse::<f32>().context("failed to parse r in dcblock_ff"))
            .transpose()?
            .unwrap_or(0.999);

        let blk = DcBlockFf::new(r);
        let id = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
