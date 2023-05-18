use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use futuresdr::anyhow::Result;
use futuresdr::blocks::FirBuilder;
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
        let blk = FirBuilder::new_resampling::<f32, f32>(interp, decim);
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
