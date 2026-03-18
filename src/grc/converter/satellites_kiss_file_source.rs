use super::super::converter_helper::{BlockConverter, ConnectorAdapter};
use super::BlockInstance;
use anyhow::{bail, Context, Result};
use futuresdr::runtime::{BlockId, Flowgraph};
use crate::blocks::kiss_file_source::KissFileSource;

#[derive(Clone, Copy)]
pub struct KissFileSourcePortAdapter {
    blk: BlockId,
}

impl ConnectorAdapter for KissFileSourcePortAdapter {
    fn adapt_input_port(&self, port_name: &str) -> Result<(BlockId, &str)> {
        bail!("KissFileSource has no input port: {port_name}");
    }

    fn adapt_output_port(&self, port_name: &str) -> Result<(BlockId, &str)> {
        match port_name {
            "0" | "out" | "output" => Ok((self.blk, "output")),
            _ => bail!("Unknown output port name {port_name}"),
        }
    }
}

pub struct SatellitesKissFileSourceConverter {}

impl BlockConverter for SatellitesKissFileSourceConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let filename = blk
            .parameters
            .get("file")
            .context("satellites_kiss_file_source: file must be defined")?;

        let filename = if "-" == filename {
            "/proc/self/fd/0"
        } else {
            filename
        };

        let block = KissFileSource::new(filename);
        Ok(Box::new(KissFileSourcePortAdapter { blk: fg.add_block(block).into() }))
    }
}
