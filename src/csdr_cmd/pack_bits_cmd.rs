use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use fsdr_blocks::futuresdr::anyhow::Result;
use pest::iterators::Pair;

pub trait PackBitsCmd<'i> {
    fn build_pack_bits(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::U8)
            .create_block_instance("blocks_pack_k_bits_bb")
            .with_parameter("k", "8")
            .assert_output(GrcItemType::U8)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> PackBitsCmd<'i> for Pair<'i, Rule> {}
