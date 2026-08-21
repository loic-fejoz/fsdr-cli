use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait DetectNanCmd<'i> {
    fn build_detect_nan(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::F32)?
            .create_block_instance("detect_nan_ff")
            .assert_output(GrcItemType::F32)
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> DetectNanCmd<'i> for Pair<'i, Rule> {}
