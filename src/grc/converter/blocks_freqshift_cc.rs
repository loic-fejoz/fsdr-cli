use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{parameter_as_f64, BlockInstance};
use crate::grc::backend::FsdrBackend;
use anyhow::Result;

pub struct FreqShiftCcConverter {}

impl<B: FsdrBackend> BlockConverter<B> for FreqShiftCcConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let sample_rate = parameter_as_f64(blk, "sample_rate", "48000")? as f32;
        let freq = parameter_as_f64(blk, "freq", "1.0")? as f32;

        let blk_ref = backend.add_freq_shift_cc(freq, sample_rate)?;
        Ok(Box::new(DefaultPortAdapter::new(blk_ref)))
    }
}
