use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use fsdr_blocks::futuresdr::anyhow::Result;
use pest::iterators::Pair;

pub trait RealPartCmd<'i> {
    fn build_realpart(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::C32)
            .create_block_instance("blocks_complex_to_real")
            .assert_output(GrcItemType::F32)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> RealPartCmd<'i> for Pair<'i, Rule> {}
