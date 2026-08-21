use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait PackBitsCmd<'i> {
    fn is_1to8(&self) -> bool;

    fn build_pack_bits(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let block_id = if self.is_1to8() {
            "blocks_unpack_k_bits_bb"
        } else {
            "blocks_pack_k_bits_bb"
        };
        grc = grc
            .ensure_source(GrcItemType::U8)?
            .create_block_instance(block_id)
            .with_parameter("k", "8")
            .assert_output(GrcItemType::U8)
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> PackBitsCmd<'i> for Pair<'i, Rule> {
    fn is_1to8(&self) -> bool {
        self.as_str().contains("1to8")
    }
}
