use crate::blocks::RepeatU8;
use crate::grc::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use crate::grc::BlockInstance;
use anyhow::{Context, Result};
use futuresdr::runtime::Flowgraph;

pub struct RepeatU8Converter {}

impl BlockConverter for RepeatU8Converter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let repeat: usize = blk
            .parameters
            .get("repeat")
            .context("missing repeat parameter in repeat_u8")?
            .parse()
            .context("failed to parse repeat parameter")?;

        let blk = RepeatU8::new(repeat);
        let id = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
