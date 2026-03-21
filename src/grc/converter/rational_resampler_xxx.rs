use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{parameter_as_f64, BlockInstance};
use crate::grc::backend::{CodegenTaps, FsdrBackend};
use anyhow::{bail, Result};
use futuresdr::blocks::FirBuilder;
use futuresdr::num_complex::Complex32;

pub struct RationalResamplerXxConverter {}

impl<B: FsdrBackend> BlockConverter<B> for RationalResamplerXxConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let interp = parameter_as_f64(blk, "interp", "1")? as usize;
        let decim = parameter_as_f64(blk, "decim", "1")? as usize;
        let kind = blk.parameter_or("type", "fff");
        let adapter: Box<dyn ConnectorAdapter<B::BlockRef>> = match kind {
            "fff" => {
                let blk_ref =
                    backend.add_rational_resampler_ff(interp, decim, CodegenTaps(vec![]))?;
                Box::new(DefaultPortAdapter::new(blk_ref))
            }
            "ccc" => {
                let blk = FirBuilder::resampling::<Complex32, Complex32>(interp, decim);
                let blk_ref = backend.add_block_runtime(blk)?;
                Box::new(DefaultPortAdapter::new(blk_ref))
            }
            _ => bail!("Unknown rational resampler type: {kind}"),
        };
        Ok(adapter)
    }
}
