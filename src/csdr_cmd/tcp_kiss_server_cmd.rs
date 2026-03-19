use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder};
use anyhow::{Context, Result};
use pest::iterators::Pair;

pub trait TcpKissServerCmd<'i> {
    fn build_tcp_kiss_server(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
}

impl<'i> TcpKissServerCmd<'i> for Pair<'i, Rule> {
    fn build_tcp_kiss_server(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut inner_rules = self.clone().into_inner();
        let target = inner_rules.next().context("filepath expected")?.as_str();

        let (address, port) = match target.rsplit_once(':') {
            Some((addr, p)) => (addr, p),
            None => (target, "8100"),
        };

        let grc = grc
            .create_block_instance("satellites_kiss_server_sink")
            .with_parameter("address", format!("\"{}\"", address))
            .with_parameter("port", port)
            .push_and_link()?;

        Ok(grc)
    }
}
