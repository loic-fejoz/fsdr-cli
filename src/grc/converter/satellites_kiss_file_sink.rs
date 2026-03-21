use super::super::converter_helper::{BlockConverter, ConnectorAdapter};
use super::BlockInstance;
use crate::grc::backend::FsdrBackend;
use anyhow::{bail, Context, Result};

#[derive(Clone, Copy)]
pub struct KissFileSinkPortAdapter<BlockRef: Clone> {
    blk: BlockRef,
}

impl<BlockRef: Clone> ConnectorAdapter<BlockRef> for KissFileSinkPortAdapter<BlockRef> {
    fn adapt_input_port(&self, port_name: &str) -> Result<(BlockRef, &str)> {
        match port_name {
            "0" | "in" | "in_port" | "input" => Ok((self.blk.clone(), "in_port")),
            _ => bail!("Unknown input port name {port_name}"),
        }
    }

    fn adapt_output_port(&self, port_name: &str) -> Result<(BlockRef, &str)> {
        bail!("KissFileSink has no output port: {port_name}");
    }
}

pub struct SatellitesKissFileSinkConverter {}

impl<B: FsdrBackend> BlockConverter<B> for SatellitesKissFileSinkConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let filename = blk
            .parameters
            .get("file")
            .context("satellites_kiss_file_sink: file must be defined")?;

        let blk_ref = backend.add_kiss_file_sink(filename.to_string())?;
        Ok(Box::new(KissFileSinkPortAdapter { blk: blk_ref }))
    }
}
