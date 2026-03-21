use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{parameter_as_f64, BlockInstance};
use crate::grc::backend::{CodegenTaps, FsdrBackend};
use anyhow::{bail, Context, Result};
use futuresdr::futuredsp::{firdes, windows};

pub struct FirFilterXxConverter {}

impl<B: FsdrBackend> BlockConverter<B> for FirFilterXxConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let item_type = blk
            .parameters
            .get("type")
            .context("fir_filter_xxx: item type must be defined")?;
        let taps_str = blk
            .parameters
            .get("taps")
            .context("fir_filter_xxx: taps must be defined")?;
        let decimation = parameter_as_f64(blk, "decim", "1")? as usize;
        let taps: Vec<f32> = if taps_str.is_empty() {
            // This block definition was from csdr
            // so use dedicated parameters
            let transition_bw = parameter_as_f64(blk, "transition_bw", "1")?;
            let window = blk
                .parameters
                .get("window")
                .context("fir_filter_xxx: window must be defined")?;
            let taps_length: usize = (4.0 / transition_bw) as usize;
            let taps_length = if taps_length % 2 == 0 {
                taps_length + 1
            } else {
                taps_length
            };

            let rect_win = match &window[..] {
                "HAMMING" => windows::hamming(taps_length, false),
                "BLACKMAN" => windows::blackman(taps_length, false),
                "HANN" => windows::hann(taps_length, false),
                _ => bail!("fir_filter_xxx: Unknown window: {window}"),
            };
            let taps = firdes::lowpass::<f32>(transition_bw, rect_win.as_slice());
            taps
        } else {
            bail!("fir_filter_xxx: Unhandled taps definition")
        };

        let adapter: Box<dyn ConnectorAdapter<B::BlockRef>> = match &(item_type[..]) {
            "ccc" => {
                let blk_ref = backend.add_fir_filter_ccc(decimation, CodegenTaps(taps))?;
                Box::new(DefaultPortAdapter::new(blk_ref))
            }
            _ => bail!("fir_filter_xxx: Unhandled type {item_type}"),
        };
        Ok(adapter)
    }
}
