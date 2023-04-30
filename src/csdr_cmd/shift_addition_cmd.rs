use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use futuresdr::anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait ShiftAdditionCmd<'i> {
    fn phase_rate(&self) -> Result<&str>;

    fn build_shift_addition(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc.clone();
        let phase_rate = self.phase_rate()?;
        grc = grc
            .ensure_source(GrcItemType::C32)
            .create_block_instance("blocks_freqshift_cc")
            .with_parameter("freq", phase_rate)
            .with_parameter("sample_rate", "6.283185307179586")
            .assert_output(GrcItemType::C32)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> ShiftAdditionCmd<'i> for Pair<'i, Rule> {
    fn phase_rate(&self) -> Result<&'i str> {
        if let Some(rate) = self.clone().into_inner().next() {
            Ok(rate.as_str())
        } else {
            bail!("missing mandatory <rate> parameters for shift_addition_cc")
        }
    }
}
