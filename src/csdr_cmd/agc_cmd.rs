use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use fsdr_blocks::futuresdr::anyhow::{Context, Result};
use pest::iterators::Pair;

pub trait AgcCmd<'i> {
    fn reference(&self) -> Result<Option<&str>>;
    fn max_gain(&self) -> Result<Option<&str>>;
    fn rate(&self) -> Result<Option<&str>>;

    fn build_agc(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let reference = self.reference()?.or(Some("0.8")).expect("");
        let max_gain = self.max_gain()?.or(Some("65536.0")).expect("");
        let rate = self.rate()?.or(Some("0.0001")).expect("");
        grc = grc
            .ensure_source(GrcItemType::F32)
            .create_block_instance("analog_agc_xx")
            .with_parameter("reference", reference)
            .with_parameter("max_gain", max_gain)
            .with_parameter("rate", rate)
            .with_parameter("type", GrcItemType::F32.as_grc())
            .assert_output(GrcItemType::F32)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> AgcCmd<'i> for Pair<'i, Rule> {
    fn reference(&self) -> Result<Option<&'i str>> {
        for arg in self.clone().into_inner() {
            if arg.as_rule() == Rule::agc_ref_param {
                return Ok(Some(arg.into_inner().next().context("context")?.as_str()));
            }
        }
        Ok(None)
    }

    fn max_gain(&self) -> Result<Option<&'i str>> {
        for arg in self.clone().into_inner() {
            if arg.as_rule() == Rule::agc_max_param {
                return Ok(Some(
                    arg.into_inner()
                        .next()
                        .context("max_gain expected")?
                        .as_str(),
                ));
            }
        }
        Ok(None)
    }

    fn rate(&self) -> Result<Option<&'i str>> {
        for arg in self.clone().into_inner() {
            if arg.as_rule() == Rule::agc_rate_param {
                return Ok(Some(
                    arg.into_inner().next().context("rate expected")?.as_str(),
                ));
            }
        }
        Ok(None)
    }
}
