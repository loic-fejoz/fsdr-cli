use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{parameter_as_f32, BlockInstance};
use crate::grc::backend::FsdrBackend;
use anyhow::Result;
use futuresdr::blocks::Apply;
use futuresdr::num_complex::Complex32;

pub struct WeaverSsbConverter {}

impl<B: FsdrBackend> BlockConverter<B> for WeaverSsbConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let audio_rate = parameter_as_f32(blk, "audio_rate", "(1500/48000)")?;

        let mut osc = Complex32::new(1.0, 0.0);
        let shift = Complex32::from_polar(1.0, 2.0 * std::f32::consts::PI * audio_rate);
        let adapter: Box<dyn ConnectorAdapter<B::BlockRef>> = if blk.id == "weaver_lsb_cf" {
            let blk: Apply<_, Complex32, f32> = Apply::new(move |v: &Complex32| -> f32 {
                osc *= shift;
                let term1 = v.re * osc.re;
                let term2 = v.im * osc.im;
                term1 - term2 // substraction for LSB
            });
            let blk_ref = backend.add_block_runtime(blk)?;
            Box::new(DefaultPortAdapter::new(blk_ref))
        } else {
            let blk: Apply<_, Complex32, f32> = Apply::new(move |v: &Complex32| -> f32 {
                osc *= shift;
                let term1 = v.re * osc.re;
                let term2 = v.im * osc.im;
                term1 + term2 // addition for USB
            });
            let blk_ref = backend.add_block_runtime(blk)?;
            Box::new(DefaultPortAdapter::new(blk_ref))
        };
        Ok(adapter)
    }
}
