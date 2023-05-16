use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use fsdr_blocks::stdinout::StdInOutBuilder;
use futuresdr::anyhow::Result;
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
        let filename = blk
            .parameters
            .get("file")
            .expect("filename must be defined");
        let item_type = blk
            .parameters
            .get("type")
            .expect("item type must be defined");
        let blk = if "-" == filename {
            match &(item_type[..]) {
                "u8" => StdInOutBuilder::<u8>::stdout().as_ne().build(),
                "i16" | "ishort" | "short" => StdInOutBuilder::<i16>::stdout().as_ne().build(),
                "f32" | "float" => StdInOutBuilder::<f32>::stdout().as_ne().build(),
                "c32" | "complex" => StdInOutBuilder::<Complex32>::stdout().as_ne().build(),
                _ => todo!("Unhandled StdIn FileSink Type {item_type}"),
            }
        } else {
            match &(item_type[..]) {
                "u8" => FileSink::<u8>::new(filename),
                "i16" | "short" => FileSink::<i16>::new(filename),
                "f32" | "float" => FileSink::<f32>::new(filename),
                "c32" | "complex" => FileSink::<Complex32>::new(filename),
                _ => todo!("Unhandled FileSink Type {item_type}"),
            }
        };
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
