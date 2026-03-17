use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use anyhow::{Context, Result};
use futuresdr::runtime::Flowgraph;
use crate::blocks::kiss_file_source::KissFileSource;

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
        Ok(Box::new(DefaultPortAdapter::new(fg.add_block(block).into())))
    }
}
