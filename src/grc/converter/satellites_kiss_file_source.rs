use super::super::converter_helper::{BlockConverter, ConnectorAdapter};
use super::BlockInstance;
use crate::blocks::kiss_file_source::KissFileSource;
use crate::grc::backend::FsdrBackend;
use anyhow::{bail, Context, Result};

#[derive(Clone, Copy)]
pub struct KissFileSourcePortAdapter<BR: Clone> {
    blk: BR,
}

impl<BR: Clone> ConnectorAdapter<BR> for KissFileSourcePortAdapter<BR> {
    fn adapt_input_port(&self, port_name: &str) -> Result<(BR, &str)> {
        bail!("KissFileSource has no input port: {port_name}");
    }

    fn adapt_output_port(&self, port_name: &str) -> Result<(BR, &str)> {
        match port_name {
            "0" | "out" | "output" => Ok((self.blk.clone(), "output")),
            _ => bail!("Unknown output port name {port_name}"),
        }
    }
}

pub struct SatellitesKissFileSourceConverter {}

impl<B: FsdrBackend> BlockConverter<B> for SatellitesKissFileSourceConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let filename = blk
            .parameters
            .get("file")
            .context("satellites_kiss_file_source: file must be defined")?;

        let filename = if "-" == filename {
            "/proc/self/fd/0"
        } else {
            filename
        };

        let block = KissFileSource::new(filename)?;
        let blk_ref = backend.add_block_runtime(block)?;
        Ok(Box::new(KissFileSourcePortAdapter { blk: blk_ref }))
    }
}
