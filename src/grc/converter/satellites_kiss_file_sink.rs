use super::super::converter_helper::{BlockConverter, ConnectorAdapter};
use super::BlockInstance;
use anyhow::{bail, Context, Result};
use futuresdr::runtime::{BlockId, Flowgraph};
use crate::blocks::kiss_file_sink::KissFileSink;

#[derive(Clone, Copy)]
pub struct KissFileSinkPortAdapter {
    blk: BlockId,
}

impl ConnectorAdapter for KissFileSinkPortAdapter {
    fn adapt_input_port(&self, port_name: &str) -> Result<(BlockId, &str)> {
        match port_name {
            "0" | "in" | "in_port" | "input" => Ok((self.blk, "in_port")),
            _ => bail!("Unknown input port name {port_name}"),
        }
    }

    fn adapt_output_port(&self, port_name: &str) -> Result<(BlockId, &str)> {
        bail!("KissFileSink has no output port: {port_name}");
    }
}

pub struct SatellitesKissFileSinkConverter {}

impl BlockConverter for SatellitesKissFileSinkConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let filename = blk
            .parameters
            .get("file")
            .context("satellites_kiss_file_sink: file must be defined")?;

        let block = KissFileSink::new(filename);
        Ok(Box::new(KissFileSinkPortAdapter { blk: fg.add_block(block).into() }))
    }
}
