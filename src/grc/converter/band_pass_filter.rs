use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use futuresdr::anyhow::Result;
use futuresdr::blocks::FirBuilder;
use futuresdr::futuredsp::{firdes, windows, TapsAccessor};
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;

struct Complex32VecTaps {
    taps: Vec<Complex32>
}

impl Complex32VecTaps {
    fn new(taps: Vec<Complex32>) -> Complex32VecTaps {
        Complex32VecTaps {
            taps
        }
    }
}

impl TapsAccessor for Complex32VecTaps
{
    type TapType = Complex32;

    fn num_taps(&self) -> usize {
        self.taps.len()
    }

    unsafe fn get(&self, index: usize) -> Complex32 {
        debug_assert!(index < self.num_taps());
        *self.taps.get_unchecked(index)
    }
}

pub struct BandPassFilterConverter {}

impl BlockConverter for BandPassFilterConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let low_cutoff_freq = Grc2FutureSdr::parameter_as_f64(blk, "low_cutoff_freq", "0.0")?; // Low Cutoff frequency in Hz
        let high_cutoff_freq = Grc2FutureSdr::parameter_as_f64(blk, "high_cutoff_freq", "1.0")?; // Low Cutoff frequency in Hz
        let decimation = Grc2FutureSdr::parameter_as_f64(blk, "decim", "1")? as usize; // Decimation rate of filter
        let _gain = Grc2FutureSdr::parameter_as_f64(blk, "gain", "1.0")?;
        let interp = Grc2FutureSdr::parameter_as_f64(blk, "interp", "1.0")? as usize;
        let sample_rate = Grc2FutureSdr::parameter_as_f64(blk, "samp_rate", "1.0")?;
        let item_type = blk.parameters.get("type").expect("type must be defined");
        let transition_bw = Grc2FutureSdr::parameter_as_f64(blk, "width", "1.0")?; // Transition width between stop-band and pass-band in Hz
        let window = blk.parameters.get("win").expect("win must be defined");

        let low_cutoff_freq = low_cutoff_freq / sample_rate;
        let high_cutoff_freq = high_cutoff_freq / sample_rate;

        let taps_length: usize = (4.0 / transition_bw) as usize;
        let taps_length = taps_length + if taps_length % 2 == 0 { 1 } else { 0 };
        assert!(taps_length % 2 == 1); //number of symmetric FIR filter taps should be odd
        let rect_win = match &window[..] {
            "window.WIN_HAMMING" => windows::hamming(taps_length, false),
            "window.WIN_BLACKMAN" => windows::blackman(taps_length, false),
            "window.WIN_KAISER" => {
                let beta = Grc2FutureSdr::parameter_as_f64(blk, "beta", "1.0")?;
                windows::kaiser(taps_length, beta)
            }
            "window.WIN_HANN" => windows::hann(taps_length, false),
            "window.WIN_GAUSSIAN" => {
                // NB: Mismatch between name and key is no a bug
                let alpha = Grc2FutureSdr::parameter_as_f64(blk, "beta", "1.0")?;
                windows::gaussian(taps_length, alpha)
            }
            _ => todo!("Unknown band_pass_filter window: {window}"),
        };
        let blk = match &(item_type[..]) {
            "fir_filter_ccf" => {
                let taps = firdes::bandpass::<f32>(low_cutoff_freq, high_cutoff_freq, &rect_win);
                FirBuilder::new_resampling_with_taps::<Complex32, Complex32, f32, _>(
                    interp, decimation, taps,
                )
            },
            "fir_filter_ccc" => {
                let taps = firdes::bandpass::<Complex32>(low_cutoff_freq, high_cutoff_freq, &rect_win);
                let taps = Complex32VecTaps::new(taps);
                FirBuilder::new_resampling_with_taps::<Complex32, Complex32, Complex32, Complex32VecTaps>(
                    interp, decimation, taps,
                )
            }
            _ => todo!("Unhandled band_pass_filter Type {item_type}"),
        };
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
