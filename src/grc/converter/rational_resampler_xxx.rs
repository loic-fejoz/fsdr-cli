use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use fsdr_blocks::futuresdr::anyhow::{bail, Result};
use fsdr_blocks::futuresdr::blocks::FirBuilder;
use fsdr_blocks::futuresdr::num_complex::Complex32;
use fsdr_blocks::futuresdr::runtime::Flowgraph;

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
        let blk = match kind {
            "fff" => FirBuilder::new_resampling::<f32, f32>(interp, decim),
            "ccc" => FirBuilder::new_resampling::<Complex32, Complex32>(interp, decim),
            _ => bail!("Unknown rational resampler type: {kind}"),
        };
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
