use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder};
use anyhow::{Context, Result};
use pest::iterators::Pair;

pub trait TcpKissClientCmd<'i> {
    fn build_tcp_kiss_client(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
}

impl<'i> TcpKissClientCmd<'i> for Pair<'i, Rule> {
    fn build_tcp_kiss_client(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut inner_rules = self.clone().into_inner();
        let target = inner_rules.next().context("filepath expected")?.as_str();

        let (address, port) = match target.rsplit_once(':') {
            Some((addr, p)) => (addr, p),
            None => (target, "8100"),
        };

        let grc = grc
            .create_block_instance("satellites_kiss_client_source")
            .with_parameter("address", format!("\"{}\"", address))
            .with_parameter("port", port)
            .push()?;

        Ok(grc)
    }
}
