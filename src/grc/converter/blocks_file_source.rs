use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use crate::grc::builder::GrcItemType;
use fsdr_blocks::futuresdr::anyhow::Result;
use fsdr_blocks::futuresdr::blocks::FileSource;
use fsdr_blocks::futuresdr::num_complex::Complex32;
use fsdr_blocks::futuresdr::runtime::Flowgraph;

pub struct FileSourceConverter {}

impl BlockConverter for FileSourceConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let item_type: GrcItemType = blk
            .parameters
            .get("type")
            .expect("item type must be defined")
            .into();

        let filename = blk
            .parameters
            .get("file")
            .expect("filename must be defined");
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
        let blk = match item_type {
            GrcItemType::U8 => FileSource::<u8>::new(filename, repeat),
            GrcItemType::S8 => FileSource::<i8>::new(filename, repeat),
            GrcItemType::F32 => FileSource::<f32>::new(filename, repeat),
            GrcItemType::C32 => FileSource::<Complex32>::new(filename, repeat),
            _ => todo!("Unhandled FileSource Type {:?}", item_type),
        };
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
