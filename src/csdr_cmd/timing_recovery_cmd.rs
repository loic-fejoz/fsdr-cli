use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use fsdr_blocks::futuresdr::anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait TimingRecoveryCmd<'i> {
    fn algorithm(&self) -> Result<&str>;

    fn decimation(&self) -> Result<&str>;

    fn mu(&self) -> Result<Option<&str>>;
    fn max_error(&self) -> Result<Option<&str>>;

    fn build_timing_recovery(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let algo = self.algorithm()?;
        let decim = self.decimation()?;
        let mu = self.mu()?.unwrap_or("0.5");
        let max_error = self.max_error()?.unwrap_or("2");
        grc = grc
            .ensure_source(GrcItemType::C32)
            .create_block_instance("timing_recovery")
            .with_parameter("algorithm", algo)
            .with_parameter("decimation", decim)
            .with_parameter("mu", mu)
            .with_parameter("max_error", max_error)
            .assert_output(GrcItemType::C32)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> TimingRecoveryCmd<'i> for Pair<'i, Rule> {
    fn algorithm(&self) -> Result<&'i str> {
        let mut inner = self.clone().into_inner();
        if let Some(value) = inner.next() {
            Ok(value.as_str())
        } else {
            bail!("missing mandatory <algorithm> parameters for timing_recovery_cc")
        }
    }

    fn decimation(&self) -> Result<&'i str> {
        let mut inner = self.clone().into_inner();
        inner.next();
        if let Some(value) = inner.next() {
            Ok(value.as_str())
        } else {
            bail!("missing mandatory <decimation> parameters for timing_recovery_cc")
        }
    }

    fn mu(&self) -> Result<Option<&'i str>> {
        let mut inner = self.clone().into_inner();
        inner.next();
        inner.next();
        if let Some(value) = inner.next() {
            Ok(Some(value.as_str()))
        } else {
            Ok(None)
        }
    }

    fn max_error(&self) -> Result<Option<&'i str>> {
        let mut inner = self.clone().into_inner();
        inner.next();
        inner.next();
        inner.next();
        if let Some(value) = inner.next() {
            Ok(Some(value.as_str()))
        } else {
            Ok(None)
        }
    }
}
