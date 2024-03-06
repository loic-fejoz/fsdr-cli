use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use fsdr_blocks::futuresdr::anyhow::Result;
use pest::iterators::Pair;

pub trait FastDCBlockCmd<'i> {
    fn build_fastdcblock(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::F32)
            .create_block_instance("dc_blocker_xx")
            .with_parameter("length", "32")
            .with_parameter("long_form", "False")
            .with_parameter("type", "ff")
            .assert_output(GrcItemType::F32)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> FastDCBlockCmd<'i> for Pair<'i, Rule> {}
