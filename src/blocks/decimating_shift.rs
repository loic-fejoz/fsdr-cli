use anyhow::Result;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;
use std::f32::consts::PI;

#[derive(Block)]
pub struct DecimatingShiftAdditionCc<
    I: CpuBufferReader<Item = Complex32> = DefaultCpuReader<Complex32>,
    O: CpuBufferWriter<Item = Complex32> = DefaultCpuWriter<Complex32>,
> {
    phase: f32,
    phase_increment: f32,
    decimation: usize,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl DecimatingShiftAdditionCc<DefaultCpuReader<Complex32>, DefaultCpuWriter<Complex32>> {
    pub fn new(rate: f32, decimation: usize) -> Self {
        assert!(decimation > 0, "decimation must be greater than 0");
        let phase_increment = 2.0 * PI * rate * (decimation as f32);
        Self {
            phase: 0.0,
            phase_increment,
            decimation,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for DecimatingShiftAdditionCc<I, O>
where
    I: CpuBufferReader<Item = Complex32>,
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

        let max_out_by_in = in_slice.len() / self.decimation;
        let n_out = cmp::min(max_out_by_in, out_slice.len());

        if n_out > 0 {
            let mut phase = self.phase;
            let phase_inc = self.phase_increment;
            let dec = self.decimation;

            for k in 0..n_out {
                let s = in_slice[k * dec];
                let (sin_val, cos_val) = phase.sin_cos();

                // (s.re + j*s.im) * (cos_val + j*sin_val)
                // re = s.re * cos_val - s.im * sin_val
                // im = s.re * sin_val + s.im * cos_val
                let re_cos = unsafe { core::intrinsics::fmul_fast(s.re, cos_val) };
                let im_sin = unsafe { core::intrinsics::fmul_fast(s.im, sin_val) };
                let re_sin = unsafe { core::intrinsics::fmul_fast(s.re, sin_val) };
                let im_cos = unsafe { core::intrinsics::fmul_fast(s.im, cos_val) };

                let out_re = unsafe { core::intrinsics::fsub_fast(re_cos, im_sin) };
                let out_im = unsafe { core::intrinsics::fadd_fast(re_sin, im_cos) };

                out_slice[k] = Complex32::new(out_re, out_im);

                phase = unsafe { core::intrinsics::fadd_fast(phase, phase_inc) };
                if phase > 2.0 * PI {
                    phase = unsafe { core::intrinsics::fsub_fast(phase, 2.0 * PI) };
                } else if phase < -2.0 * PI {
                    phase = unsafe { core::intrinsics::fadd_fast(phase, 2.0 * PI) };
                }
            }

            self.phase = phase;
            self.input.consume(n_out * dec);
            self.output.produce(n_out);
        }

        if self.input.slice().len() >= self.decimation && !self.output.slice().is_empty() {
            io.call_again = true;
        }

        if self.input.finished() && self.input.slice().len() < self.decimation {
            io.finished = true;
        }

        Ok(())
    }
}
