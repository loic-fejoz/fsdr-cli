use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{parameter_as_f32, BlockInstance};
use crate::grc::backend::{CodegenPattern, FsdrBackend};
use anyhow::Result;

pub struct PatternSearchConverter {}

impl<B: FsdrBackend> BlockConverter<B> for PatternSearchConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let values_after = parameter_as_f32(blk, "values_after", "8")? as usize;
        let pattern_values = blk.parameter_or("pattern_values", "0,1");
        let pattern_values: Vec<u8> = pattern_values
            .split(",")
            .map(|x| x.parse::<u8>().unwrap())
            .collect();
        let blk_ref =
            backend.add_pattern_search_u8(values_after, CodegenPattern(pattern_values))?;
        Ok(Box::new(DefaultPortAdapter::new(blk_ref)))
    }
}
