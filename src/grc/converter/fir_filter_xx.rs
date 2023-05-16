use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use futuresdr::anyhow::Result;
use futuresdr::blocks::FirBuilder;
use futuresdr::futuredsp::{firdes, windows};
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;

pub struct FirFilterXxConverter {}

impl BlockConverter for FirFilterXxConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let item_type = blk
            .parameters
            .get("type")
            .expect("item type must be defined");
        let taps = blk.parameters.get("taps").expect("taps must be defined");
        let decimation = Grc2FutureSdr::parameter_as_f64(blk, "decim", "1")? as usize;
        let taps: Vec<f32> = if taps.is_empty() {
            // This block definition was from csdr
            // so use dedicated parameters
            let transition_bw = Grc2FutureSdr::parameter_as_f64(blk, "transition_bw", "1")?;
            let window = blk
                .parameters
                .get("window")
                .expect("window must be defined");
            let taps_length: usize = (4.0 / transition_bw) as usize;
            let taps_length = taps_length + if taps_length % 2 == 0 { 1 } else { 0 };
            assert!(taps_length % 2 == 1); //number of symmetric FIR filter taps should be odd

            // Building firdes_lowpass_f(taps,taps_length,0.5/(float)factor,window);
            let rect_win = match &window[..] {
                "HAMMING" => windows::hamming(taps_length, false),
                "BLACKMAN" => windows::blackman(taps_length, false),
                //"KAISER" => windows::kaiser(taps_length, beta),
                "HANN" => windows::hann(taps_length, false),
                //"GAUSSIAN" => windows::gaussian(taps_length, alpha),
                _ => todo!("Unknown fir_filter_xx window: {window}"),
            };
            let taps = firdes::lowpass::<f32>(transition_bw, rect_win.as_slice());
            taps
        } else {
            todo!("Unhandled fir_filter_xx taps definition")
        };
        let blk = match &(item_type[..]) {
            "ccc" => FirBuilder::new_resampling_with_taps::<Complex32, Complex32, f32, _>(
                1, decimation, taps,
            ),
            _ => todo!("Unhandled fir_filter_xx Type {item_type}"),
        };
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
