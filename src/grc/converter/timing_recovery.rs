use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use anyhow::{bail, Result, Context};
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;
use crate::blocks::synchronizers::{TimingAlgorithm, TimingRecovery};

pub struct TimingRecoveryConverter {}

impl BlockConverter for TimingRecoveryConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let algo = blk.parameter_or("algorithm", "GARDNER");
        let decim = Grc2FutureSdr::parameter_as_f32(blk, "decimation", "8")? as usize;
        if decim <= 4 || decim % 4 != 0 {
            bail!("decimation factor for timing recovery must be divisible by 4, and strictly greater than 4.")
        }
        let mu = Grc2FutureSdr::parameter_as_f32(blk, "mu", "0.5")? as f32;
        let max_error = Grc2FutureSdr::parameter_as_f32(blk, "max_error", "2")?;
        let algo = match algo {
            "GARDNER" => TimingAlgorithm::GARDNER,
            "EARLYLATE" => TimingAlgorithm::EARLYLATE,
            _ => bail!("Unknown timing recovery algorithm: {algo}"),
        };
        let blk = TimingRecovery::<Complex32>::new(algo, decim, mu, max_error);
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk.into());
        let blk = Box::new(blk);
        Ok(blk)
    }
}
