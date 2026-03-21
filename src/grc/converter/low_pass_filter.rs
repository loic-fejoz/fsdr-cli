use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{parameter_as_f64, BlockInstance};
use crate::grc::backend::FsdrBackend;
use anyhow::{bail, Context, Result};
use futuresdr::futuredsp::{firdes, windows};
use futuresdr::num_complex::Complex32;

pub struct LowPassFilterConverter {}

impl<B: FsdrBackend> BlockConverter<B> for LowPassFilterConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let item_type = blk
            .parameters
            .get("type")
            .context("low_pass_filter: item type must be defined")?;
        let decimation = parameter_as_f64(blk, "decim", "1")? as usize;
        let gain = parameter_as_f64(blk, "gain", "1.0")? as f32;
        let samp_rate = parameter_as_f64(blk, "samp_rate", "1.0")?;
        let cutoff_freq = parameter_as_f64(blk, "cutoff_freq", "1.0")?;
        let width = parameter_as_f64(blk, "width", "1.0")?;
        let window = blk
            .parameters
            .get("window")
            .context("low_pass_filter: window must be defined")?;

        let taps_length: usize = (4.0 * samp_rate / width) as usize;
        let taps_length = taps_length + if taps_length.is_multiple_of(2) { 1 } else { 0 };

        let rect_win = match &window[..] {
            "window.WIN_HAMMING" | "HAMMING" => windows::hamming(taps_length, false),
            "window.WIN_BLACKMAN" | "BLACKMAN" => windows::blackman(taps_length, false),
            "window.WIN_HANN" | "HANN" => windows::hann(taps_length, false),
            _ => bail!("low_pass_filter: Unknown window: {window}"),
        };
        let taps = firdes::lowpass::<f32>(cutoff_freq / samp_rate, rect_win.as_slice());
        let taps: Vec<f32> = taps.into_iter().map(|v| v * gain).collect();

        let adapter: Box<dyn ConnectorAdapter<B::BlockRef>> = match &(item_type[..]) {
            "ccf" | "complex" => {
                let blk = futuresdr::blocks::FirBuilder::resampling_with_taps::<
                    Complex32,
                    Complex32,
                    Vec<f32>,
                >(1, decimation, taps);
                let blk_ref = backend.add_block_runtime(blk)?;
                Box::new(DefaultPortAdapter::new(blk_ref))
            }
            "fff" | "float" => {
                let blk = futuresdr::blocks::FirBuilder::resampling_with_taps::<f32, f32, Vec<f32>>(
                    1, decimation, taps,
                );
                let blk_ref = backend.add_block_runtime(blk)?;
                Box::new(DefaultPortAdapter::new(blk_ref))
            }
            _ => bail!("low_pass_filter: Unhandled type {item_type}"),
        };
        Ok(adapter)
    }
}
