use super::super::converter_helper::{BlockConverter, ConnectorAdapter};
use super::BlockInstance;
use crate::blocks::tcp_kiss_server::TcpKissServer;
use anyhow::{bail, Context, Result};
use futuresdr::runtime::{BlockId, Flowgraph};

#[derive(Clone, Copy)]
pub struct TcpKissServerSinkPortAdapter {
    blk: BlockId,
}

impl ConnectorAdapter for TcpKissServerSinkPortAdapter {
    fn adapt_input_port(&self, port_name: &str) -> Result<(BlockId, &str)> {
        match port_name {
            "0" | "in" | "in_port" | "input" | "pdus" => Ok((self.blk, "in_port")),
            _ => bail!("Unknown input port name {port_name}"),
        }
    }

    fn adapt_output_port(&self, port_name: &str) -> Result<(BlockId, &str)> {
        bail!("TcpKissServer has no output port: {port_name}");
    }
}

pub struct SatellitesKissServerSinkConverter {}

impl BlockConverter for SatellitesKissServerSinkConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let address = blk
            .parameters
            .get("address")
            .context("satellites_kiss_server_sink: address must be defined")?;
        let port = blk
            .parameters
            .get("port")
            .context("satellites_kiss_server_sink: port must be defined")?;

        let address = address.trim_matches('"');
        let block = TcpKissServer::new(&format!("{}:{}", address, port))?;
        Ok(Box::new(TcpKissServerSinkPortAdapter {
            blk: fg.add_block(block).into(),
        }))
    }
}
