use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait PowerTaggerCmd<'i> {
    fn build_power_tagger(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
}

impl<'i> PowerTaggerCmd<'i> for Pair<'i, Rule> {
    fn build_power_tagger(
        &self,
        mut grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let it = self.clone().into_inner();
        let grc = grc.ensure_source(GrcItemType::C32)?;
        let mut block = grc.create_block_instance("analog_power_tagger_cc");
        block.with_parameter("type", "ccc");

        for param in it {
            match param.as_rule() {
                Rule::power_tagger_samp_rate => {
                    let mut inner = param.into_inner();
                    block.with_parameter("samp_rate", inner.next().unwrap().as_str());
                }
                Rule::power_tagger_threshold => {
                    let mut inner = param.into_inner();
                    block.with_parameter("threshold", inner.next().unwrap().as_str());
                }
                Rule::power_tagger_off_threshold => {
                    let mut inner = param.into_inner();
                    block.with_parameter("off_threshold", inner.next().unwrap().as_str());
                }
                Rule::power_tagger_on_threshold => {
                    let mut inner = param.into_inner();
                    block.with_parameter("on_threshold", inner.next().unwrap().as_str());
                }
                Rule::power_tagger_delay => {
                    let mut inner = param.into_inner();
                    block.with_parameter("delay", inner.next().unwrap().as_str());
                }
                Rule::power_tagger_off_delay => {
                    let mut inner = param.into_inner();
                    block.with_parameter("off_delay", inner.next().unwrap().as_str());
                }
                Rule::power_tagger_on_delay => {
                    let mut inner = param.into_inner();
                    block.with_parameter("on_delay", inner.next().unwrap().as_str());
                }
                Rule::power_tagger_window => {
                    let mut inner = param.into_inner();
                    block.with_parameter("window_size", inner.next().unwrap().as_str());
                }
                Rule::power_tagger_off_tag => {
                    let mut inner = param.into_inner();
                    block.with_parameter("off_tag", inner.next().unwrap().as_str());
                }
                Rule::power_tagger_on_tag => {
                    let mut inner = param.into_inner();
                    block.with_parameter("on_tag", inner.next().unwrap().as_str());
                }
                Rule::power_tagger_debug => {
                    block.with_parameter("debug", "true");
                }
                _ => {}
            }
        }
        block.assert_output(GrcItemType::C32).push_and_link()
    }
}
