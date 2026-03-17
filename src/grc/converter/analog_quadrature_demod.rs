use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use anyhow::{bail, Result, Context};
use futuresdr::blocks::Apply;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;

pub struct AnalogQuadratureDemoConverter {}

impl BlockConverter for AnalogQuadratureDemoConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let gain = Grc2FutureSdr::parameter_as_f64(blk, "gain", "1.0")? as f32;
        let algo = blk.parameter_or("algorithm", "quadri");
        let blk: Box<dyn ConnectorAdapter> = match algo {
            "quadri" => {
                // Quadrature demodulator: phase difference between consecutive samples.
                // arg(x[n] * conj(x[n-1]))
                let mut last = Complex32::new(0.0, 0.0);
                let blk: Apply<_, Complex32, f32> = Apply::new(move |v: &Complex32| -> f32 {
                    let arg = (v * last.conj()).arg();
                    last = *v;
                    arg * gain
                });
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            "atan" => {
                // Atan demodulator: differentiate the instantaneous phase.
                // arg(x[n]) - arg(x[n-1])
                let mut last_phase = 0.0f32;
                let blk: Apply<_, Complex32, f32> = Apply::new(move |v: &Complex32| -> f32 {
                    let phase = v.arg();
                    let mut diff = phase - last_phase;
                    // Wrap phase difference to [-π, π]
                    if diff > std::f32::consts::PI {
                        diff -= 2.0 * std::f32::consts::PI;
                    } else if diff < -std::f32::consts::PI {
                        diff += 2.0 * std::f32::consts::PI;
                    }
                    last_phase = phase;
                    diff * gain
                });
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            _ => bail!("analog_quadrature_demod: Unknown algorithm: {algo}"),
        };
        Ok(blk)
    }
}
