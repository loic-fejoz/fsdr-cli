use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait DecimatingShiftAdditionCmd<'i> {
    fn rate(&self) -> Result<&str>;
    fn decimation(&self) -> Result<&str>;

    fn build_decimating_shift_addition_cc(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let rate = self.rate()?;
        let decimation = self.decimation()?;
        grc = grc
            .ensure_source(GrcItemType::C32)?
            .create_block_instance("decimating_shift_addition_cc")
            .with_parameter("rate", rate)
            .with_parameter("decimation", decimation)
            .assert_output(GrcItemType::C32)
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> DecimatingShiftAdditionCmd<'i> for Pair<'i, Rule> {
    fn rate(&self) -> Result<&'i str> {
        let mut inner = self.clone().into_inner();
        if let Some(value) = inner.next() {
            Ok(value.as_str())
        } else {
            bail!("missing mandatory <rate> parameter for decimating_shift_addition_cc")
        }
    }

    fn decimation(&self) -> Result<&'i str> {
        let mut inner = self.clone().into_inner();
        inner.next();
        if let Some(value) = inner.next() {
            Ok(value.as_str())
        } else {
            bail!("missing mandatory <decimation> parameter for decimating_shift_addition_cc")
        }
    }
}
