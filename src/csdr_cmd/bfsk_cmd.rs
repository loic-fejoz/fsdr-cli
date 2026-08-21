use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait BfskCmd<'i> {
    fn build_bfsk_demod(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::C32)?
            .create_block_instance("bfsk_demod_cf")
            .assert_output(GrcItemType::F32)
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> BfskCmd<'i> for Pair<'i, Rule> {}
