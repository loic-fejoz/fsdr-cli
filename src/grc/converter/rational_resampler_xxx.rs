use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use anyhow::{bail, Result};
use futuresdr::blocks::FirBuilder;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;

pub struct RationalResamplerXxConverter {}

impl BlockConverter for RationalResamplerXxConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let interp = Grc2FutureSdr::parameter_as_f64(blk, "interp", "1")? as usize;
        let decim = Grc2FutureSdr::parameter_as_f64(blk, "decim", "1")? as usize;
        let kind = blk.parameter_or("type", "fff");
        let blk: Box<dyn ConnectorAdapter> = match kind {
            "fff" => {
                let blk = FirBuilder::resampling::<f32, f32>(interp, decim);
                let blk = fg.add_block(blk);
                Box::new(DefaultPortAdapter::new(blk.into()))
            }
            "ccc" => {
                let blk = FirBuilder::resampling::<Complex32, Complex32>(interp, decim);
                let blk = fg.add_block(blk);
                Box::new(DefaultPortAdapter::new(blk.into()))
            }
            _ => bail!("Unknown rational resampler type: {kind}"),
        };
        Ok(blk)
    }
}
