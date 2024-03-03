use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use futuresdr::anyhow::Result;
use pest::iterators::Pair;

pub trait BinarySlicerCmd<'i> {
    fn build_binary_slicer(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::F32)
            .create_block_instance("digital_binary_slicer_fb")
            .assert_output(GrcItemType::U8)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> BinarySlicerCmd<'i> for Pair<'i, Rule> {}
