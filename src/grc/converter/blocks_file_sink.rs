use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use crate::grc::builder::GrcItemType;
use fsdr_blocks::stdinout::StdInOutBuilder;
use fsdr_blocks::futuresdr::anyhow::Result;
use fsdr_blocks::futuresdr::blocks::FileSink;
use fsdr_blocks::futuresdr::num_complex::Complex32;
use fsdr_blocks::futuresdr::runtime::Flowgraph;

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
        let item_type: GrcItemType = blk
            .parameters
            .get("type")
            .expect("item type must be defined")
            .into();
        let blk = if "-" == filename {
            match item_type {
                GrcItemType::U8 => StdInOutBuilder::<u8>::stdout().as_ne().build(),
                GrcItemType::S16 => StdInOutBuilder::<i16>::stdout().as_ne().build(),
                GrcItemType::F32 => StdInOutBuilder::<f32>::stdout().as_ne().build(),
                GrcItemType::C32 => StdInOutBuilder::<Complex32>::stdout().as_ne().build(),
                _ => todo!("Unhandled StdOut FileSink Type {:?}", item_type),
            }
        } else {
            match item_type {
                GrcItemType::U8 => FileSink::<u8>::new(filename),
                GrcItemType::S16 => FileSink::<i16>::new(filename),
                GrcItemType::F32 => FileSink::<f32>::new(filename),
                GrcItemType::C32 => FileSink::<Complex32>::new(filename),
                _ => todo!("Unhandled FileSink Type {:?}", item_type),
            }
        };
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
