use super::super::converter_helper::{BlockConverter, ConnectorAdapter};
use super::BlockInstance;
use crate::blocks::tcp_kiss_client::TcpKissClient;
use crate::grc::backend::FsdrBackend;
use anyhow::{bail, Context, Result};

#[derive(Clone, Copy)]
pub struct TcpKissClientSourcePortAdapter<BlockRef: Clone> {
    blk: BlockRef,
}

impl<BlockRef: Clone> ConnectorAdapter<BlockRef> for TcpKissClientSourcePortAdapter<BlockRef> {
    fn adapt_input_port(&self, port_name: &str) -> Result<(BlockRef, &str)> {
        bail!("TcpKissClient has no input port: {port_name}");
    }

    fn adapt_output_port(&self, port_name: &str) -> Result<(BlockRef, &str)> {
        match port_name {
            "0" | "out" | "out_port" | "output" | "pdus" => Ok((self.blk.clone(), "out")),
            _ => bail!("Unknown output port name {port_name}"),
        }
    }
}

pub struct SatellitesKissClientSourceConverter {}

impl<B: FsdrBackend> BlockConverter<B> for SatellitesKissClientSourceConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
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
        let blk_ref = backend.add_block_runtime(block)?;
        Ok(Box::new(TcpKissClientSourcePortAdapter { blk: blk_ref }))
    }
}
