use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait TimerTaggerCmd<'i> {
    fn build_timer_tagger(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
}

impl<'i> TimerTaggerCmd<'i> for Pair<'i, Rule> {
    fn build_timer_tagger(
        &self,
        mut grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let it = self.clone().into_inner();
        let grc = grc.ensure_source(GrcItemType::F32)?;
        let mut block = grc.create_block_instance("blocks_timer_tagger_ff");
        block.with_parameter("type", "fff");

        for param in it {
            match param.as_rule() {
                Rule::timer_samp_rate => {
                    let mut inner = param.into_inner();
                    block.with_parameter("samp_rate", inner.next().unwrap().as_str());
                }
                Rule::timer_duration => {
                    let mut inner = param.into_inner();
                    block.with_parameter("duration", inner.next().unwrap().as_str());
                }
                Rule::timer_start => {
                    let mut inner = param.into_inner();
                    block.with_parameter("start", inner.next().unwrap().as_str());
                }
                Rule::timer_end => {
                    let mut inner = param.into_inner();
                    block.with_parameter("end", inner.next().unwrap().as_str());
                }
                Rule::timer_debug => {
                    block.with_parameter("debug", "true");
                }
                _ => {}
            }
        }
        block.assert_output(GrcItemType::F32).push_and_link()
    }
}
