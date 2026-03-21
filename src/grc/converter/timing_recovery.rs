use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{parameter_as_f64, BlockInstance};
use crate::grc::backend::FsdrBackend;
use anyhow::{Context, Result};

pub struct TimingRecoveryConverter {}

impl<B: FsdrBackend> BlockConverter<B> for TimingRecoveryConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let decim = parameter_as_f64(blk, "decimation", "1")? as usize;

        if decim <= 4 || !decim.is_multiple_of(4) {
            panic!("timing_recovery: decim must be a multiple of 4 and > 4 (decim={decim})");
        }

        let item_type = blk
            .parameters
            .get("type")
            .context("timing_recovery: item type must be defined")?;

        let adapter: Box<dyn ConnectorAdapter<B::BlockRef>> = match &(item_type[..]) {
            "cc" | "complex" => {
                let blk_ref = backend.add_timing_recovery_cc(decim)?;
                Box::new(DefaultPortAdapter::new(blk_ref))
            }
            _ => todo!("Unhandled timing_recovery Type {item_type}"),
        };
        Ok(adapter)
    }
}
