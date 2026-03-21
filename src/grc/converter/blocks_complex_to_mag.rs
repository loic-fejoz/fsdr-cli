use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use crate::grc::backend::FsdrBackend;
use anyhow::Result;
use futuresdr::blocks::Apply;
use futuresdr::num_complex::Complex32;

pub struct ComplexToMagConverter {}

impl<B: FsdrBackend> BlockConverter<B> for ComplexToMagConverter {
    fn convert(
        &self,
        _blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let blk: Apply<_, Complex32, f32> = Apply::new(|i: &Complex32| -> f32 { i.norm() });
        let blk_ref = backend.add_block_runtime(blk)?;
        Ok(Box::new(DefaultPortAdapter::new(blk_ref)))
    }
}
