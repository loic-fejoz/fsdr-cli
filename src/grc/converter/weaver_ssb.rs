use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use anyhow::{Result, Context};
use futuresdr::blocks::Apply;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;

pub struct WeaverSsbConverter {}

impl BlockConverter for WeaverSsbConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let audio_rate = Grc2FutureSdr::parameter_as_f32(blk, "audio_rate", "(1500/48000)")?;

        let mut osc = Complex32::new(1.0, 0.0);
        let shift = Complex32::from_polar(1.0, 2.0 * std::f32::consts::PI * audio_rate);
        let blk: Box<dyn ConnectorAdapter> = if blk.id == "weaver_lsb_cf" {
            let blk: Apply<_, Complex32, f32> = Apply::new(move |v: &Complex32| -> f32 {
                osc *= shift;
                let term1 = v.re * osc.re;
                let term2 = v.im * osc.im;
                term1 - term2 // substraction for LSB
            });
            let blk = fg.add_block(blk);
            Box::new(DefaultPortAdapter::new(blk.into()))
        } else {
            let blk: Apply<_, Complex32, f32> = Apply::new(move |v: &Complex32| -> f32 {
                osc *= shift;
                let term1 = v.re * osc.re;
                let term2 = v.im * osc.im;
                term1 + term2 // addition for USB
            });
            let blk = fg.add_block(blk);
            Box::new(DefaultPortAdapter::new(blk.into()))
        };
        Ok(blk)
    }
}
