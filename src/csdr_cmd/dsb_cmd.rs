use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use futuresdr::anyhow::Result;
use pest::iterators::Pair;

pub trait DsbCmd<'i> {
    fn build_dsb(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::F32)
            .create_block_instance("dsb")
            .with_parameter("q_value", "0.0")
            .assert_output(GrcItemType::C32)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> DsbCmd<'i> for Pair<'i, Rule> {}
