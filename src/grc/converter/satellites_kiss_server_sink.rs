use super::super::converter_helper::{BlockConverter, ConnectorAdapter};
use super::BlockInstance;
use crate::blocks::tcp_kiss_server::TcpKissServer;
use crate::grc::backend::FsdrBackend;
use anyhow::{bail, Context, Result};

#[derive(Clone, Copy)]
pub struct TcpKissServerSinkPortAdapter<BlockRef: Clone> {
    blk: BlockRef,
}

impl<BlockRef: Clone> ConnectorAdapter<BlockRef> for TcpKissServerSinkPortAdapter<BlockRef> {
    fn adapt_input_port(&self, port_name: &str) -> Result<(BlockRef, &str)> {
        match port_name {
            "0" | "in" | "in_port" | "input" | "pdus" => Ok((self.blk.clone(), "in_port")),
            _ => bail!("Unknown input port name {port_name}"),
        }
    }

    fn adapt_output_port(&self, port_name: &str) -> Result<(BlockRef, &str)> {
        bail!("TcpKissServer has no output port: {port_name}");
    }
}

pub struct SatellitesKissServerSinkConverter {}

impl<B: FsdrBackend> BlockConverter<B> for SatellitesKissServerSinkConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
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
        let blk_ref = backend.add_block_runtime(block)?;
        Ok(Box::new(TcpKissServerSinkPortAdapter { blk: blk_ref }))
    }
}
