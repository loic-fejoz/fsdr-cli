use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use crate::grc::converter::bail;
use fsdr_blocks::futuresdr::anyhow::Result;
use fsdr_blocks::futuresdr::blocks::Apply;
use fsdr_blocks::futuresdr::num_complex::Complex32;
use fsdr_blocks::futuresdr::runtime::Flowgraph;

pub struct AnalogQuadratureDemoConverter {}

const fmdemod_quadri_K: f32 = 0.340447550238101026565118445432744920253753662109375f32;

impl BlockConverter for AnalogQuadratureDemoConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let gain = Grc2FutureSdr::parameter_as_f64(blk, "gain", "1.0")? as f32;
        let algo = blk.parameter_or("algorithm", "quadri");
        let mut last = Complex32::new(0.0, 0.0); // store sample x[n-1]

        let blk = match algo {
            "atan" => Apply::new(move |v: &Complex32| -> f32 {
                let arg = (v * last.conj()).arg(); // Obtain phase of x[n] * conj(x[n-1])
                last = *v;
                arg * gain
            }),
            "quadri" => Apply::new(move |v: &Complex32| -> f32 {
                // gain * fmdemod_quadri_K * x[n] . (x[n] - x[n-1]) / x[n]²
                let t = v - last;
                let o = v.re * t.im - v.im * t.re;
                let t = v.re * v.re + v.im * v.im;
                last = *v;
                if t > 0.0f32 {
                    gain * fmdemod_quadri_K * o / t
                } else {
                    0.0f32
                }
            }),
            _ => bail!("Unknown FM demodulation algorithm: {algo}"),
        };
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
