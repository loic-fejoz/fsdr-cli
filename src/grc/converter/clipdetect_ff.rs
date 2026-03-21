use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use crate::grc::backend::FsdrBackend;
use anyhow::Result;
use futuresdr::blocks::Apply;

pub struct ClipDetectFfConverter {}

impl<B: FsdrBackend> BlockConverter<B> for ClipDetectFfConverter {
    fn convert(
        &self,
        _blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let blk: Apply<_, f32, f32> = Apply::new(|i: &f32| -> f32 {
            if *i < -1.0 {
                eprintln!("csdr clipdetect_ff: Signal value below -1.0!")
            } else if *i > 1.0 {
                eprintln!("csdr clipdetect_ff: Signal value above 1.0!")
            };
            *i
        });
        let blk_ref = backend.add_block_runtime(blk)?;
        Ok(Box::new(DefaultPortAdapter::new(blk_ref)))
    }
}
