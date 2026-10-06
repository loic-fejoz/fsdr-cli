use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait FmModCmd<'i> {
    fn build_fmmod(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::F32)?
            .create_block_instance("fmmod_fc")
            .assert_output(GrcItemType::C32)
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> FmModCmd<'i> for Pair<'i, Rule> {}
