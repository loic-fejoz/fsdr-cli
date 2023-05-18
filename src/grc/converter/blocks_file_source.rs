use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use futuresdr::anyhow::Result;
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
            .expect("item type must be defined");

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
        let blk = match &(item_type[..]) {
            "u8" => FileSource::<u8>::new(filename, repeat),
            "byte" => FileSource::<i8>::new(filename, repeat),
            "f32" | "float" => FileSource::<f32>::new(filename, repeat),
            "c32" | "complex" => FileSource::<Complex32>::new(filename, repeat),
            _ => todo!("Unhandled FileSource Type {item_type}"),
        };
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
