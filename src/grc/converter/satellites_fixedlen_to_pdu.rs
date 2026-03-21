use super::super::converter_helper::{BlockConverter, ConnectorAdapter};
use super::BlockInstance;
use crate::grc::backend::FsdrBackend;
use anyhow::{bail, Context, Result};

#[derive(Clone, Copy)]
pub struct FixedlenToPduPortAdapter<BlockRef: Clone> {
    blk: BlockRef,
}

impl<BlockRef: Clone> ConnectorAdapter<BlockRef> for FixedlenToPduPortAdapter<BlockRef> {
    fn adapt_input_port(&self, port_name: &str) -> Result<(BlockRef, &str)> {
        match port_name {
            "0" | "in" | "input" => Ok((self.blk.clone(), "input")),
            _ => bail!("Unknown input port name {port_name}"),
        }
    }

    fn adapt_output_port(&self, port_name: &str) -> Result<(BlockRef, &str)> {
        match port_name {
            "0" | "pdus" | "out" | "output" => Ok((self.blk.clone(), "pdus")),
            _ => bail!("FixedlenToPdu unknown output port: {port_name}"),
        }
    }
}

pub struct SatellitesFixedlenToPduConverter {}

impl<B: FsdrBackend> BlockConverter<B> for SatellitesFixedlenToPduConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let packet_len_str = blk
            .parameters
            .get("packet_len")
            .context("satellites_fixedlen_to_pdu: packet_len must be defined")?;

        let packet_len = packet_len_str
            .parse::<usize>()
            .context("packet_len must be an integer")?;

        let blk_ref = backend.add_fixedlen_to_pdu(packet_len)?;
        Ok(Box::new(FixedlenToPduPortAdapter { blk: blk_ref }))
    }
}
