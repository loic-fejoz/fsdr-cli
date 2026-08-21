use crate::blocks::AddConstCc;
use crate::grc::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use crate::grc::BlockInstance;
use anyhow::{Context, Result};
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;

pub struct AddConstConverter {}

impl BlockConverter for AddConstConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let real: f32 = blk
            .parameters
            .get("real")
            .map(|s| s.parse::<f32>().context("failed to parse real"))
            .transpose()?
            .unwrap_or(0.0);

        let imag: f32 = blk
            .parameters
            .get("imag")
            .map(|s| s.parse::<f32>().context("failed to parse imag"))
            .transpose()?
            .unwrap_or(0.0);

        let blk = AddConstCc::new(Complex32::new(real, imag));
        let id = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
