use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use futuresdr::anyhow::{Context, Result};
use pest::iterators::Pair;

pub trait ConvertCmd<'i> {
    fn types(&self) -> Result<(GrcItemType, GrcItemType)>;

    fn build_convert(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc.clone();
        let (src_type, tgt_type) = self.types()?;
        let blk_name = match (src_type, tgt_type) {
            (GrcItemType::U8, GrcItemType::F32) => "blocks_uchar_to_float",
            (GrcItemType::S8, GrcItemType::F32) => "blocks_char_to_float",
            (GrcItemType::S16, GrcItemType::F32) => "blocks_short_to_float",
            (GrcItemType::S16, GrcItemType::S8) => "blocks_short_to_char",
            (GrcItemType::F32, GrcItemType::U8) => "blocks_float_to_uchar",
            (GrcItemType::F32, GrcItemType::S8) => "blocks_float_to_char",
            (GrcItemType::F32, GrcItemType::S16) => "blocks_float_to_short",
            (GrcItemType::InterleavedF32, GrcItemType::C32) => "convert_ff_c",
            _ => "convert",
        };
        grc = grc
            .ensure_source(src_type)
            .create_block_instance(blk_name)
            .with_parameter("source_type", src_type.as_csdr())
            .with_parameter("target_type", tgt_type.as_csdr())
            .assert_output(tgt_type)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> ConvertCmd<'i> for Pair<'i, Rule> {
    fn types(&self) -> Result<(GrcItemType, GrcItemType)> {
        if let Some(types) = self.clone().into_inner().next() {
            let types = types.as_str();
            let mut types_iter = types.split("_").map(|t| GrcItemType::from(t));
            Ok((
                types_iter.next().context("source converion type")?,
                types_iter.next().context("conversion target type")?,
            ))
        } else {
            Ok((GrcItemType::U8, GrcItemType::F32))
        }
    }
}
