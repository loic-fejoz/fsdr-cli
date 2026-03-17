use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use crate::grc::builder::GrcItemType;
use anyhow::{Result, Context};
use futuresdr::blocks::FileSource;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;

pub struct FileSourceConverter {}

impl BlockConverter for FileSourceConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let item_type = blk
            .parameters
            .get("type")
            .context("blocks_file_source: item type must be defined")?;

        let filename = blk
            .parameters
            .get("file")
            .context("blocks_file_source: filename must be defined")?;
        let repeat = blk
            .parameters
            .get("repeat")
            .unwrap_or(&"false".into())
            .to_lowercase()
            .parse::<bool>()?;
        let filename = if "-" == filename {
            "/proc/self/fd/0"
        } else {
            filename
        };
        let blk: Box<dyn ConnectorAdapter> = match &(item_type[..]) {
            "u8" | "uchar" => {
                let blk = FileSource::<u8>::new(filename, repeat);
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            "byte" => {
                let blk = FileSource::<i8>::new(filename, repeat);
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            "f32" | "float" => {
                let blk = FileSource::<f32>::new(filename, repeat);
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            "c32" | "complex" => {
                let blk = FileSource::<Complex32>::new(filename, repeat);
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            _ => todo!("Unhandled FileSource Type {item_type}"),
        };
        Ok(blk)
    }
}
