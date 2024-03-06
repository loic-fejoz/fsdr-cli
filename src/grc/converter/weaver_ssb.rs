use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use fsdr_blocks::futuresdr::anyhow::Result;
use fsdr_blocks::futuresdr::blocks::Apply;
use fsdr_blocks::futuresdr::num_complex::Complex32;
use fsdr_blocks::futuresdr::runtime::Flowgraph;

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
        let blk = if blk.id == "weaver_lsb_cf" {
            Apply::new(move |v: &Complex32| {
                osc *= shift;
                let term1 = v.re * osc.re;
                let term2 = v.im * osc.im;
                term1 - term2 // substraction for LSB, addition for USB
            })
        } else {
            Apply::new(move |v: &Complex32| {
                osc *= shift;
                let term1 = v.re * osc.re;
                let term2 = v.im * osc.im;
                term1 + term2 // substraction for LSB, addition for USB
            })
        };

        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
