use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{parameter_as_f64, BlockInstance};
use crate::grc::backend::FsdrBackend;
use anyhow::{bail, Result};
use futuresdr::blocks::Apply;
use futuresdr::num_complex::Complex32;

pub struct AnalogQuadratureDemoConverter {}

impl<B: FsdrBackend> BlockConverter<B> for AnalogQuadratureDemoConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let gain = parameter_as_f64(blk, "gain", "1.0")? as f32;
        let algo = blk.parameter_or("algorithm", "quadri");
        let adapter: Box<dyn ConnectorAdapter<B::BlockRef>> = match algo {
            "quadri" => {
                let blk_ref = backend.add_quadrature_demod_cf(gain)?;
                Box::new(DefaultPortAdapter::new(blk_ref))
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
                let blk_ref = backend.add_block_runtime(blk)?;
                Box::new(DefaultPortAdapter::new(blk_ref))
            }
            _ => bail!("analog_quadrature_demod: Unknown algorithm: {algo}"),
        };
        Ok(adapter)
    }
}
