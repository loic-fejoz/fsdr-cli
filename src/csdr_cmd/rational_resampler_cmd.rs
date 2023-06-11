use super::cmd::CsdrCmd;
use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use futuresdr::anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait RationalResamplerCmd<'i> {
    fn block_type(&self) -> Result<&str>;
    fn decimation(&self) -> Result<&str>;
    fn interpolation(&self) -> Result<&str>;
    fn bandwidth(&self) -> Result<Option<&str>>;
    fn window(&self) -> Result<Option<&str>>;

    fn build_rational_resampler(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let block_type = self.block_type()?;
        let interpolation = self.interpolation()?;
        let decimation = self.decimation()?;
        let bandwidth = self.bandwidth()?.unwrap_or("0.05");
        let window = self.window()?.unwrap_or("HAMMING");
        grc = grc
            .ensure_source(GrcItemType::C32)
            .create_block_instance("rational_resampler_xxx")
            .with_parameter("decim", decimation)
            .with_parameter("interp", interpolation)
            .with_parameter("fbw", bandwidth)
            .with_parameter("window", window)
            .with_parameter("taps", "")
            .with_parameter("type", block_type)
            .assert_output(GrcItemType::C32)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> RationalResamplerCmd<'i> for Pair<'i, Rule> {
    fn block_type(&self) -> Result<&str> {
        let mut inner = self.clone().into_inner();
        if let Some(value) = inner.next() {
            match value.as_str() {
                "cc" => Ok("ccc"),
                "ff" => Ok("fff"),
                _ => bail!("unknown rational_resampler type"),
            }
        } else {
            bail!("unknown rational_resampler type")
        }
    }

    fn interpolation(&self) -> Result<&str> {
        self.arg::<2>("missing mandatory <interp> parameters for fractional_decimator_ff")
    }

    fn decimation(&self) -> Result<&'i str> {
        self.arg::<3>("missing mandatory <decim> parameters for fractional_decimator_ff")
    }

    fn bandwidth(&self) -> Result<Option<&'i str>> {
        self.optional_arg::<4>()
    }

    fn window(&self) -> Result<Option<&'i str>> {
        self.optional_arg::<5>()
    }
}