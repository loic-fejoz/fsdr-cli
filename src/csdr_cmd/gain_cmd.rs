use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use futuresdr::anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait GainCmd<'i> {
    fn gain(&self) -> Result<&str>;

    fn build_gain(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let gain = self.gain()?;
        grc = grc
            .ensure_source(GrcItemType::F32)
            .create_block_instance("blocks_multiply_const_vxx")
            .with_parameter("const", gain)
            .with_parameter("type", GrcItemType::F32.as_grc())
            .assert_output(GrcItemType::F32)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> GainCmd<'i> for Pair<'i, Rule> {
    fn gain(&self) -> Result<&'i str> {
        if let Some(rate) = self.clone().into_inner().next() {
            Ok(rate.as_str())
        } else {
            bail!("missing mandatory <gain> parameters for gain_ff")
        }
    }
}
