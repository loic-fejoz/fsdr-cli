use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use anyhow::Result;
use futuresdr::blocks::NullSink;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;

pub struct NullSinkConverter {}

impl BlockConverter for NullSinkConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let item_type = blk
            .parameters
            .get("type")
            .expect("item type must be defined");
        let blk: Box<dyn ConnectorAdapter> = match &(item_type[..]) {
            "byte" | "char" | "u8" | "uchar" => {
                let blk = NullSink::<u8>::new();
                Box::new(DefaultPortAdapter::new(fg.add(blk)?.id()))
            }
            "short" | "ishort" | "s16" | "i16" => {
                let blk = NullSink::<i16>::new();
                Box::new(DefaultPortAdapter::new(fg.add(blk)?.id()))
            }
            "float" | "f32" | "f" => {
                let blk = NullSink::<f32>::new();
                Box::new(DefaultPortAdapter::new(fg.add(blk)?.id()))
            }
            "complex" | "c32" | "c" => {
                let blk = NullSink::<Complex32>::new();
                Box::new(DefaultPortAdapter::new(fg.add(blk)?.id()))
            }
            _ => todo!("Unhandled blocks_null_sink Type {item_type}"),
        };
        Ok(blk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_sink_all_types() -> Result<()> {
        let mut fg = Flowgraph::new();
        let converter = NullSinkConverter {};

        for type_name in &[
            "byte", "char", "u8", "uchar", "short", "ishort", "s16", "i16", "float", "f32", "f",
            "complex", "c32", "c",
        ] {
            let blk_instance =
                BlockInstance::new("null_snk", "blocks_null_sink").with("type", type_name);
            let adapter = converter.convert(&blk_instance, &mut fg)?;
            let (_, in_port) = adapter.adapt_input_port("0")?;
            assert_eq!(in_port, "input");
        }

        Ok(())
    }
}
