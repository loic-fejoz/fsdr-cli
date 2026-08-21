use crate::blocks::DecimatingShiftAdditionCc;
use crate::grc::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use crate::grc::BlockInstance;
use anyhow::{Context, Result};
use futuresdr::runtime::Flowgraph;

pub struct DecimatingShiftAdditionCcConverter {}

impl BlockConverter for DecimatingShiftAdditionCcConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let rate: f32 = blk
            .parameters
            .get("rate")
            .context("missing rate parameter in decimating_shift_addition_cc")?
            .parse()
            .context("failed to parse rate in decimating_shift_addition_cc")?;

        let decimation: usize = blk
            .parameters
            .get("decimation")
            .context("missing decimation parameter in decimating_shift_addition_cc")?
            .parse()
            .context("failed to parse decimation in decimating_shift_addition_cc")?;

        let blk = DecimatingShiftAdditionCc::new(rate, decimation);
        let id = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
