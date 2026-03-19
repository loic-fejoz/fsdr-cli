use super::super::converter_helper::{BlockConverter, ConnectorAdapter};
use super::BlockInstance;
use crate::blocks::tcp_kiss_client::TcpKissClient;
use anyhow::{bail, Context, Result};
use futuresdr::runtime::{BlockId, Flowgraph};

#[derive(Clone, Copy)]
pub struct TcpKissClientSourcePortAdapter {
    blk: BlockId,
}

impl ConnectorAdapter for TcpKissClientSourcePortAdapter {
    fn adapt_input_port(&self, port_name: &str) -> Result<(BlockId, &str)> {
        bail!("TcpKissClient has no input port: {port_name}");
    }

    fn adapt_output_port(&self, port_name: &str) -> Result<(BlockId, &str)> {
        match port_name {
            "0" | "out" | "out_port" | "output" | "pdus" => Ok((self.blk, "out")),
            _ => bail!("Unknown output port name {port_name}"),
        }
    }
}

pub struct SatellitesKissClientSourceConverter {}

impl BlockConverter for SatellitesKissClientSourceConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let address = blk
            .parameters
            .get("address")
            .context("satellites_kiss_client_source: address must be defined")?;
        let port = blk
            .parameters
            .get("port")
            .context("satellites_kiss_client_source: port must be defined")?;

        let address = address.trim_matches('"');
        let block = TcpKissClient::new(&format!("{}:{}", address, port))?;
        Ok(Box::new(TcpKissClientSourcePortAdapter {
            blk: fg.add_block(block).into(),
        }))
    }
}
