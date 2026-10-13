use anyhow::Result;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;
use std::f32::consts::PI;

/// Weaver USB Modulator: converts real audio f32 stream to Complex32 baseband stream
/// centered at audio subcarrier rate (e.g. 1500 Hz / 48000 Hz).
#[derive(Block)]
pub struct WeaverUsbModFc<
    I: CpuBufferReader<Item = f32> = DefaultCpuReader<f32>,
    O: CpuBufferWriter<Item = Complex32> = DefaultCpuWriter<Complex32>,
> {
    phase: f32,
    phase_inc: f32,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl WeaverUsbModFc<DefaultCpuReader<f32>, DefaultCpuWriter<Complex32>> {
    pub fn new(audio_rate: f32) -> Self {
        let phase_inc = 2.0 * PI * audio_rate;
        Self {
            phase: 0.0,
            phase_inc,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for WeaverUsbModFc<I, O>
where
    I: CpuBufferReader<Item = f32>,
    O: CpuBufferWriter<Item = Complex32>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let in_slice = self.input.slice();
        let out_slice = self.output.slice();

        let m = cmp::min(in_slice.len(), out_slice.len());
        if m > 0 {
            let mut phase = self.phase;
            let phase_inc = self.phase_inc;

            for (src, dst) in in_slice[..m].iter().zip(out_slice[..m].iter_mut()) {
                let (sin_val, cos_val) = crate::math::fast_sincos(phase);
                *dst = Complex32::new(
                    f32::algebraic_mul(*src, cos_val),
                    f32::algebraic_mul(*src, sin_val),
                );

                phase = f32::algebraic_add(phase, phase_inc);
                if phase > 2.0 * PI {
                    phase = f32::algebraic_sub(phase, 2.0 * PI);
                }
            }

            self.phase = phase;
            self.input.consume(m);
            self.output.produce(m);
        }

        if !self.input.slice().is_empty() && !self.output.slice().is_empty() {
            io.call_again = true;
        }

        if self.input.slice().is_empty() && self.input.finished() {
            io.finished = true;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weaver_mod_oscillation() {
        let audio_rate = 1500.0 / 48000.0;
        let mod_fc = WeaverUsbModFc::new(audio_rate);
        assert!((mod_fc.phase_inc - 2.0 * PI * audio_rate).abs() < 1e-6);
    }
}
