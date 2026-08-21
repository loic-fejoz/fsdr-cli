use crate::blocks::YesF;
use crate::grc::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use crate::grc::BlockInstance;
use anyhow::{Context, Result};
use futuresdr::runtime::Flowgraph;

pub struct YesFConverter {}

impl BlockConverter for YesFConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let value: f32 = blk
            .parameters
            .get("value")
            .map(|s| s.parse::<f32>().context("failed to parse value in yes_f"))
            .transpose()?
            .unwrap_or(1.0);

        let blk = YesF::new(value);
        let id = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
