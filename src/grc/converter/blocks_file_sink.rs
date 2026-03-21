use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use crate::grc::backend::FsdrBackend;
use anyhow::{Context, Result};
use fsdr_blocks::stdinout::StdInOutBuilder;
use futuresdr::blocks::FileSink;
use futuresdr::num_complex::Complex32;

pub struct FileSinkConverter {}

impl<B: FsdrBackend> BlockConverter<B> for FileSinkConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        // Parameters "file" and "type" match the keys defined in GNU Radio's File Sink block.
        let filename = blk
            .parameters
            .get("file")
            .context("blocks_file_sink: filename must be defined")?;
        let item_type = blk
            .parameters
            .get("type")
            .context("blocks_file_sink: item type must be defined")?;
        let blk_ref = if "-" == filename {
            match &(item_type[..]) {
                "u8" => {
                    let blk = StdInOutBuilder::<u8>::stdout().as_ne().build();
                    backend.add_block_runtime(blk)?
                }
                "i16" | "ishort" | "short" => {
                    let blk = StdInOutBuilder::<i16>::stdout().as_ne().build();
                    backend.add_block_runtime(blk)?
                }
                "f32" | "float" => {
                    let blk = StdInOutBuilder::<f32>::stdout().as_ne().build();
                    backend.add_block_runtime(blk)?
                }
                "c32" | "complex" => {
                    let blk = StdInOutBuilder::<Complex32>::stdout().as_ne().build();
                    backend.add_block_runtime(blk)?
                }
                _ => todo!("Unhandled StdIn FileSink Type {item_type}"),
            }
        } else {
            match &(item_type[..]) {
                "u8" => {
                    let blk = FileSink::<u8>::new(filename);
                    backend.add_block_runtime(blk)?
                }
                "i16" | "short" => {
                    let blk = FileSink::<i16>::new(filename);
                    backend.add_block_runtime(blk)?
                }
                "f32" | "float" => {
                    let blk = FileSink::<f32>::new(filename);
                    backend.add_block_runtime(blk)?
                }
                "c32" | "complex" => {
                    let blk = FileSink::<Complex32>::new(filename);
                    backend.add_block_runtime(blk)?
                }
                _ => todo!("Unhandled FileSink Type {item_type}"),
            }
        };
        Ok(Box::new(DefaultPortAdapter::new(blk_ref)))
    }
}
