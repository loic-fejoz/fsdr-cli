use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use futuresdr::anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait FirDecimateCmd<'i> {
    fn decimation(&self) -> Result<&str>;
    fn bandwidth(&self) -> Result<Option<&str>>;
    fn window(&self) -> Result<Option<&str>>;

    fn build_fir_decimate(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let decimation = self.decimation()?;
        let bandwidth = self.bandwidth()?.unwrap_or("0.05");
        let window = self.window()?.unwrap_or("HAMMING");
        grc = grc
            .ensure_source(GrcItemType::C32)
            .create_block_instance("fir_filter_xxx")
            .with_parameter("decim", decimation)
            .with_parameter("transition_bw", bandwidth)
            .with_parameter("window", window)
            .with_parameter("taps", "")
            .with_parameter("samp_delay", "0")
            .with_parameter("type", "ccc")
            .assert_output(GrcItemType::C32)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> FirDecimateCmd<'i> for Pair<'i, Rule> {
    fn decimation(&self) -> Result<&'i str> {
        if let Some(rate) = self.clone().into_inner().next() {
            Ok(rate.as_str())
        } else {
            bail!("missing mandatory <decim> parameters for fractional_decimator_ff")
        }
    }

    fn bandwidth(&self) -> Result<Option<&'i str>> {
        let mut inner = self.clone().into_inner();
        inner.next();
        if let Some(value) = inner.next() {
            Ok(Some(value.as_str()))
        } else {
            Ok(None)
        }
    }

    fn window(&self) -> Result<Option<&'i str>> {
        let mut inner = self.clone().into_inner();
        inner.next();
        inner.next();
        if let Some(value) = inner.next() {
            Ok(Some(value.as_str()))
        } else {
            Ok(None)
        }
    }
}
