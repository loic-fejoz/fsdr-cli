use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use futuresdr::anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait DeemphasisWfmCmd<'i> {
    fn sample_rate(&self) -> Result<&str>;
    fn tau(&self) -> Result<&str>;

    fn build_deemphasis_wfm(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc.clone();
        let rate = self.sample_rate()?;
        let tau = self.tau()?;
        grc = grc
            .ensure_source(GrcItemType::F32)
            .create_block_instance("analog_fm_deemph")
            .with_parameter("samp_rate", rate)
            .with_parameter("tau", tau)
            .assert_output(GrcItemType::F32)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> DeemphasisWfmCmd<'i> for Pair<'i, Rule> {
    fn sample_rate(&self) -> Result<&'i str> {
        if let Some(rate) = self.clone().into_inner().next() {
            Ok(rate.as_str())
        } else {
            bail!("missing mandatory <sample_rate> parameters for deemphasis_wfm_ff")
        }
    }

    fn tau(&self) -> Result<&'i str> {
        let mut inner = self.clone().into_inner();
        inner.next();
        if let Some(tau) = inner.next() {
            Ok(tau.as_str())
        } else {
            bail!("missing mandatory <sample_rate> parameters for deemphasis_wfm_ff")
        }
    }
}
