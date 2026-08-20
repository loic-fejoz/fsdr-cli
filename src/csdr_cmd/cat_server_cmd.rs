use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder};
use anyhow::Result;
use pest::iterators::Pair;

pub trait CatServerCmd<'i> {
    fn build_cat_server(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
}

impl<'i> CatServerCmd<'i> for Pair<'i, Rule> {
    fn build_cat_server(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let it = self.clone().into_inner();
        let mut block = grc.create_block_instance("cat_server");

        for param in it {
            match param.as_rule() {
                Rule::cat_server_port_param => {
                    let mut inner = param.into_inner();
                    block.with_parameter("port", inner.next().unwrap().as_str());
                }
                Rule::cat_server_rx_freq_param => {
                    let mut inner = param.into_inner();
                    block.with_parameter("rx_freq", inner.next().unwrap().as_str());
                }
                Rule::cat_server_tx_freq_param => {
                    let mut inner = param.into_inner();
                    block.with_parameter("tx_freq", inner.next().unwrap().as_str());
                }
                Rule::cat_server_ctcss_tone_param => {
                    let mut inner = param.into_inner();
                    block.with_parameter("ctcss_tone", inner.next().unwrap().as_str());
                }
                _ => unreachable!(),
            }
        }
        block.push()
    }
}
