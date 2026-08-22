use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait CmdTriggerCmd<'i> {
    fn build_cmd_trigger(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
}

impl<'i> CmdTriggerCmd<'i> for Pair<'i, Rule> {
    fn build_cmd_trigger(&self, mut grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let it = self.clone().into_inner();
        let grc = grc.ensure_source(GrcItemType::F32)?;
        let mut block = grc.create_block_instance("blocks_cmd_trigger_f");
        block.with_parameter("type", "f32");

        for param in it {
            match param.as_rule() {
                Rule::cmd_trig_samp_rate => {
                    let mut inner = param.into_inner();
                    block.with_parameter("samp_rate", inner.next().unwrap().as_str());
                }
                Rule::cmd_trig_start => {
                    let mut inner = param.into_inner();
                    block.with_parameter("start_tag", inner.next().unwrap().as_str());
                }
                Rule::cmd_trig_end => {
                    let mut inner = param.into_inner();
                    block.with_parameter("end_tag", inner.next().unwrap().as_str());
                }
                Rule::cmd_trig_cmd => {
                    let mut inner = param.into_inner();
                    block.with_parameter("cmd", inner.next().unwrap().as_str());
                }
                Rule::cmd_trig_debug => {
                    block.with_parameter("debug", "true");
                }
                _ => {}
            }
        }
        block.push_and_link()
    }
}
