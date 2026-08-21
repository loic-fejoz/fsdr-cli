use crate::blocks::FixedAmplitudeCc;
use crate::grc::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use crate::grc::BlockInstance;
use anyhow::{Context, Result};
use futuresdr::runtime::Flowgraph;

pub struct FixedAmplitudeConverter {}

impl BlockConverter for FixedAmplitudeConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let amplitude: f32 = blk
            .parameters
            .get("amplitude")
            .map(|s| s.parse::<f32>().context("failed to parse amplitude"))
            .transpose()?
            .unwrap_or(1.0);

        let blk = FixedAmplitudeCc::new(amplitude);
        let id = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
