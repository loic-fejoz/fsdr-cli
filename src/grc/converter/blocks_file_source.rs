use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use crate::grc::backend::FsdrBackend;
use anyhow::{Context, Result};

pub struct FileSourceConverter {}

impl<B: FsdrBackend> BlockConverter<B> for FileSourceConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
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
            "/proc/self/fd/0".to_string()
        } else {
            filename.clone()
        };
        let blk_ref = match &(item_type[..]) {
            "u8" | "uchar" | "byte" => backend.add_file_source_u8(filename, repeat)?,
            "f32" | "float" => backend.add_file_source_f32(filename, repeat)?,
            "c32" | "complex" => backend.add_file_source_c32(filename, repeat)?,
            _ => todo!("Unhandled FileSource Type {item_type}"),
        };
        Ok(Box::new(DefaultPortAdapter::new(blk_ref)))
    }
}
