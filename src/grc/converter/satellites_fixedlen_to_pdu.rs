use super::super::converter_helper::{BlockConverter, ConnectorAdapter};
use super::BlockInstance;
use crate::blocks::FixedlenToPdu;
use anyhow::{bail, Context, Result};
use futuresdr::prelude::DefaultCpuReader;
use futuresdr::runtime::{BlockId, Flowgraph};

#[derive(Clone, Copy)]
pub struct FixedlenToPduPortAdapter {
    blk: BlockId,
}

impl ConnectorAdapter for FixedlenToPduPortAdapter {
    fn adapt_input_port(&self, port_name: &str) -> Result<(BlockId, &str)> {
        match port_name {
            "0" | "in" | "input" => Ok((self.blk, "input")),
            _ => bail!("Unknown input port name {port_name}"),
        }
    }

    fn adapt_output_port(&self, port_name: &str) -> Result<(BlockId, &str)> {
        match port_name {
            "pdus" | "out" | "output" => Ok((self.blk, "pdus")),
            _ => bail!("FixedlenToPdu unknown output port: {port_name}"),
        }
    }
}

pub struct SatellitesFixedlenToPduConverter {}

impl BlockConverter for SatellitesFixedlenToPduConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let packet_len_str = blk
            .parameters
            .get("packet_len")
            .context("satellites_fixedlen_to_pdu: packet_len must be defined")?;

        let packet_len = packet_len_str
            .parse::<usize>()
            .context("packet_len must be an integer")?;

        let block = FixedlenToPdu::<DefaultCpuReader<u8>>::new(packet_len);
        Ok(Box::new(FixedlenToPduPortAdapter {
            blk: fg.add_block(block).into(),
        }))
    }
}
