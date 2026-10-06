use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait AfcCmd<'i> {
    fn build_afc_ff(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
    fn build_afc_cc(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
}

impl<'i> AfcCmd<'i> for Pair<'i, Rule> {
    fn build_afc_ff(&self, mut grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        grc = grc.ensure_source(GrcItemType::F32)?;
        let mut block = grc.create_block_instance("analog_afc_ff");
        block.with_parameter("type", "ff");

        for param in self.clone().into_inner() {
            match param.as_rule() {
                Rule::afc_alpha_param => {
                    let mut inner = param.into_inner();
                    block.with_parameter("alpha", inner.next().unwrap().as_str());
                }
                Rule::afc_limit_param => {
                    let mut inner = param.into_inner();
                    block.with_parameter("limit", inner.next().unwrap().as_str());
                }
                _ => {}
            }
        }
        block.assert_output(GrcItemType::F32).push_and_link()
    }

    fn build_afc_cc(&self, mut grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        grc = grc.ensure_source(GrcItemType::C32)?;
        let mut block = grc.create_block_instance("analog_afc_cc");
        block.with_parameter("type", "cc");

        for param in self.clone().into_inner() {
            match param.as_rule() {
                Rule::afc_alpha_param | Rule::afc_gain_param => {
                    let mut inner = param.into_inner();
                    block.with_parameter("alpha", inner.next().unwrap().as_str());
                }
                Rule::afc_max_freq_param => {
                    let mut inner = param.into_inner();
                    block.with_parameter("max_freq", inner.next().unwrap().as_str());
                }
                Rule::afc_samp_rate_param => {
                    let mut inner = param.into_inner();
                    block.with_parameter("samp_rate", inner.next().unwrap().as_str());
                }
                _ => {}
            }
        }
        block.assert_output(GrcItemType::C32).push_and_link()
    }
}
