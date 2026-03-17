use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use anyhow::{Context, Result};
use fsdr_blocks::stdinout::StdInOutBuilder;
use futuresdr::blocks::FileSink;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;

pub struct FileSinkConverter {}

impl BlockConverter for FileSinkConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        // Parameters "file" and "type" match the keys defined in GNU Radio's File Sink block.
        let filename = blk
            .parameters
            .get("file")
            .context("blocks_file_sink: filename must be defined")?;
        let item_type = blk
            .parameters
            .get("type")
            .context("blocks_file_sink: item type must be defined")?;
        let blk = if "-" == filename {
            match &(item_type[..]) {
                "u8" => {
                    let blk = StdInOutBuilder::<u8>::stdout().as_ne().build();
                    Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
                }
                "i16" | "ishort" | "short" => {
                    let blk = StdInOutBuilder::<i16>::stdout().as_ne().build();
                    Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
                }
                "f32" | "float" => {
                    let blk = StdInOutBuilder::<f32>::stdout().as_ne().build();
                    Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
                }
                "c32" | "complex" => {
                    let blk = StdInOutBuilder::<Complex32>::stdout().as_ne().build();
                    Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
                }
                _ => todo!("Unhandled StdIn FileSink Type {item_type}"),
            }
        } else {
            match &(item_type[..]) {
                "u8" => {
                    let blk = FileSink::<u8>::new(filename);
                    Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
                }
                "i16" | "short" => {
                    let blk = FileSink::<i16>::new(filename);
                    Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
                }
                "f32" | "float" => {
                    let blk = FileSink::<f32>::new(filename);
                    Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
                }
                "c32" | "complex" => {
                    let blk = FileSink::<Complex32>::new(filename);
                    Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
                }
                _ => todo!("Unhandled FileSink Type {item_type}"),
            }
        };
        Ok(blk)
    }
}
