use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use fsdr_blocks::futuresdr::anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait FractionalDecimatorCmd<'i> {
    fn resampling_rate(&self) -> Result<&str>;

    fn build_fractional_decimator(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let resampling_rate = self.resampling_rate()?;
        grc = grc
            .ensure_source(GrcItemType::F32)
            .create_block_instance("rational_resampler_xxx")
            .with_parameter("decim", resampling_rate)
            .with_parameter("interp", "1")
            .assert_output(GrcItemType::F32)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> FractionalDecimatorCmd<'i> for Pair<'i, Rule> {
    fn resampling_rate(&self) -> Result<&'i str> {
        if let Some(rate) = self.clone().into_inner().next() {
            Ok(rate.as_str())
        } else {
            bail!("missing mandatory <decim> parameters for fractional_decimator_ff")
        }
    }
}
