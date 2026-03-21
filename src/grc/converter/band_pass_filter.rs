use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{parameter_as_f64, BlockInstance};
use crate::grc::backend::FsdrBackend;
use anyhow::{bail, Context, Result};
use futuresdr::blocks::FirBuilder;
use futuresdr::futuredsp::{firdes, windows};
use futuresdr::num_complex::Complex32;

pub struct BandPassFilterConverter {}

impl<B: FsdrBackend> BlockConverter<B> for BandPassFilterConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let low_cutoff_freq = parameter_as_f64(blk, "low_cutoff_freq", "0.0")?; // Low Cutoff frequency in Hz
        let high_cutoff_freq = parameter_as_f64(blk, "high_cutoff_freq", "1.0")?; // Low Cutoff frequency in Hz
        let decimation = parameter_as_f64(blk, "decim", "1")? as usize; // Decimation rate of filter
        let _gain = parameter_as_f64(blk, "gain", "1.0")?;
        let interp = parameter_as_f64(blk, "interp", "1.0")? as usize;
        let sample_rate = parameter_as_f64(blk, "samp_rate", "1.0")?;
        let item_type = blk
            .parameters
            .get("type")
            .context("band_pass_filter: type must be defined")?;
        let transition_bw = parameter_as_f64(blk, "width", "1.0")?; // Transition width between stop-band and pass-band in Hz
        let window = blk
            .parameters
            .get("win")
            .context("band_pass_filter: win must be defined")?;

        let low_cutoff_freq = low_cutoff_freq / sample_rate;
        let high_cutoff_freq = high_cutoff_freq / sample_rate;

        let taps_length: usize = (4.0 / transition_bw) as usize;
        let taps_length = taps_length + if taps_length.is_multiple_of(2) { 1 } else { 0 };
        assert!(taps_length % 2 == 1); //number of symmetric FIR filter taps should be odd
        let rect_win = match &window[..] {
            "window.WIN_HAMMING" => windows::hamming(taps_length, false),
            "window.WIN_BLACKMAN" => windows::blackman(taps_length, false),
            "window.WIN_KAISER" => {
                let beta = parameter_as_f64(blk, "beta", "1.0")?;
                windows::kaiser(taps_length, beta)
            }
            "window.WIN_HANN" => windows::hann(taps_length, false),
            "window.WIN_GAUSSIAN" => {
                // NB: Mismatch between name and key is no a bug
                let alpha = parameter_as_f64(blk, "beta", "1.0")?;
                windows::gaussian(taps_length, alpha)
            }
            _ => bail!("band_pass_filter: Unknown window: {window}"),
        };
        let adapter: Box<dyn ConnectorAdapter<B::BlockRef>> = match &(item_type[..]) {
            "fir_filter_ccf" => {
                let taps = firdes::bandpass::<f32>(low_cutoff_freq, high_cutoff_freq, &rect_win);
                let blk = FirBuilder::resampling_with_taps::<Complex32, Complex32, Vec<f32>>(
                    interp, decimation, taps,
                );
                let blk_ref = backend.add_block_runtime(blk)?;
                Box::new(DefaultPortAdapter::new(blk_ref))
            }
            "fir_filter_ccc" => {
                let taps = firdes::bandpass::<f32>(low_cutoff_freq, high_cutoff_freq, &rect_win);
                let blk = FirBuilder::resampling_with_taps::<Complex32, Complex32, Vec<f32>>(
                    interp, decimation, taps,
                );
                let blk_ref = backend.add_block_runtime(blk)?;
                Box::new(DefaultPortAdapter::new(blk_ref))
            }
            _ => bail!("band_pass_filter: Unhandled type {item_type}"),
        };
        Ok(adapter)
    }
}
