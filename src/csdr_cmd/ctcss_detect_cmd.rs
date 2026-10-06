use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait CtcssDetectCmd<'i> {
    fn build_ctcss_detect(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
}

impl<'i> CtcssDetectCmd<'i> for Pair<'i, Rule> {
    fn build_ctcss_detect(
        &self,
        mut grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let it = self.clone().into_inner();
        let grc = grc.ensure_source(GrcItemType::F32)?;
        let mut block = grc.create_block_instance("analog_ctcss_detect_ff");

        for param in it {
            match param.as_rule() {
                Rule::ctcss_samp_rate => {
                    let mut inner = param.into_inner();
                    block.with_parameter("samp_rate", inner.next().unwrap().as_str());
                }
                Rule::ctcss_tone => {
                    let mut inner = param.into_inner();
                    block.with_parameter("tone", inner.next().unwrap().as_str());
                }
                Rule::ctcss_threshold => {
                    let mut inner = param.into_inner();
                    block.with_parameter("threshold", inner.next().unwrap().as_str());
                }
                Rule::ctcss_duration => {
                    let mut inner = param.into_inner();
                    block.with_parameter("duration", inner.next().unwrap().as_str());
                }
                Rule::ctcss_tag => {
                    let mut inner = param.into_inner();
                    block.with_parameter("tag", inner.next().unwrap().as_str());
                }
                Rule::ctcss_debug => {
                    block.with_parameter("debug", "true");
                }
                _ => {}
            }
        }
        block.assert_output(GrcItemType::F32).push_and_link()
    }
}
