use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{parameter_as_f64, BlockInstance};
use crate::grc::backend::FsdrBackend;
use anyhow::{bail, Result};
use futuresdr::blocks::Throttle;
use futuresdr::num_complex::Complex32;

pub struct ThrottleConverter {}

impl<B: FsdrBackend> BlockConverter<B> for ThrottleConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let item_type = blk.parameter_or("type", "float");
        let rate = parameter_as_f64(blk, "samples_per_second", "48000")?;

        let adapter: Box<dyn ConnectorAdapter<B::BlockRef>> = match &item_type[..] {
            "float" => {
                let blk = Throttle::<f32>::new(rate as f64);
                let blk_ref = backend.add_block_runtime(blk)?;
                Box::new(DefaultPortAdapter::new(blk_ref))
            }
            "complex" => {
                let blk = Throttle::<Complex32>::new(rate as f64);
                let blk_ref = backend.add_block_runtime(blk)?;
                Box::new(DefaultPortAdapter::new(blk_ref))
            }
            "short" => {
                let blk = Throttle::<i16>::new(rate as f64);
                let blk_ref = backend.add_block_runtime(blk)?;
                Box::new(DefaultPortAdapter::new(blk_ref))
            }
            "byte" => {
                let blk = Throttle::<u8>::new(rate as f64);
                let blk_ref = backend.add_block_runtime(blk)?;
                Box::new(DefaultPortAdapter::new(blk_ref))
            }
            _ => bail!("blocks_throttle: Unhandled type {item_type}"),
        };
        Ok(adapter)
    }
}
