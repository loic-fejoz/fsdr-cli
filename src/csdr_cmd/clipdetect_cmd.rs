use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use futuresdr::anyhow::Result;
use pest::iterators::Pair;

pub trait ClipDetectCmd<'i> {

    fn build_clipdetect(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc.clone();
        grc = grc
            .ensure_source(GrcItemType::F32)
            .create_block_instance("clipdetect_ff")
            .assert_output(GrcItemType::F32)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> ClipDetectCmd<'i> for Pair<'i, Rule> {
}
