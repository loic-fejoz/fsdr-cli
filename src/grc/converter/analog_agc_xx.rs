use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{parameter_as_f32, BlockInstance};
use crate::grc::backend::FsdrBackend;
use anyhow::{bail, Context, Result};
use fsdr_blocks::Agc;
use futuresdr::num_complex::Complex32;

pub struct AnalogAgcXxConverter {}

impl<B: FsdrBackend> BlockConverter<B> for AnalogAgcXxConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let item_type = blk
            .parameters
            .get("type")
            .context("analog_agc_xx: item type must be defined")?;
        let reference = parameter_as_f32(blk, "reference", "1.0")?;
        let gain = parameter_as_f32(blk, "gain", "1.0")?;
        let max_gain = parameter_as_f32(blk, "max_gain", "65536.0")?;

        let adapter: Box<dyn ConnectorAdapter<B::BlockRef>> = match &(item_type[..]) {
            "complex" => {
                // Correct order: squelch, max_gain, gain, adjustment_rate, reference_power, gain_lock, auto_lock
                let blk =
                    Agc::<Complex32>::new(0.0, max_gain, gain, 0.0001, reference, false, false);
                let blk_ref = backend.add_block_runtime(blk)?;
                Box::new(DefaultPortAdapter::new(blk_ref))
            }
            "float" => {
                let blk = Agc::<f32>::new(0.0, max_gain, gain, 0.0001, reference, false, false);
                let blk_ref = backend.add_block_runtime(blk)?;
                Box::new(DefaultPortAdapter::new(blk_ref))
            }
            _ => bail!("analog_agc_xx: Unhandled type {item_type}"),
        };
        Ok(adapter)
    }
}
