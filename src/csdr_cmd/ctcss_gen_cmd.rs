use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait CtcssGenCmd<'i> {
    fn build_ctcss_gen(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
}

impl<'i> CtcssGenCmd<'i> for Pair<'i, Rule> {
    fn build_ctcss_gen(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut it = self.clone().into_inner();
        let tone = if let Some(t) = it.next() {
            t.as_str().to_string()
        } else {
            "0".to_string()
        };

        let mut block = grc.create_block_instance("ctcss_gen");
        block.with_parameter("tone", tone);
        block.with_parameter("sample_rate", "48000"); // Default sample rate for audio
        block.assert_output(GrcItemType::F32);
        block.push_and_link()
    }
}
