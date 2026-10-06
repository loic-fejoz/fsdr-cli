use anyhow::Result;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;
use std::f32::consts::PI;

#[derive(Block)]
pub struct CostasLoopCc<
    I: CpuBufferReader<Item = Complex32> = DefaultCpuReader<Complex32>,
    O: CpuBufferWriter<Item = Complex32> = DefaultCpuWriter<Complex32>,
> {
    phase: f32,
    freq: f32,
    alpha: f32,
    beta: f32,
    max_freq: f32,
    min_freq: f32,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl CostasLoopCc<DefaultCpuReader<Complex32>, DefaultCpuWriter<Complex32>> {
    pub fn new(loop_bw: f32, damping: f32) -> Self {
        let damping = if damping <= 0.0 { 0.707 } else { damping };
        let loop_bw = if loop_bw <= 0.0 { 0.05 } else { loop_bw };

        let denom = 1.0 + 2.0 * damping * loop_bw + loop_bw * loop_bw;
        let alpha = (4.0 * damping * loop_bw) / denom;
        let beta = (4.0 * loop_bw * loop_bw) / denom;

        Self {
            phase: 0.0,
            freq: 0.0,
            alpha,
            beta,
            max_freq: 1.0,
            min_freq: -1.0,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for CostasLoopCc<DefaultCpuReader<Complex32>, DefaultCpuWriter<Complex32>> {
    fn default() -> Self {
        Self::new(0.05, 0.707)
    }
}

#[doc(hidden)]
impl<I, O> Kernel for CostasLoopCc<I, O>
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

        let m = cmp::min(in_slice.len(), out_slice.len());
        if m > 0 {
            let mut phase = self.phase;
            let mut freq = self.freq;
            let alpha = self.alpha;
            let beta = self.beta;
            let max_freq = self.max_freq;
            let min_freq = self.min_freq;

            for (k, &s) in in_slice.iter().enumerate().take(m) {
                let (sin_val, cos_val) = phase.sin_cos();

                // Rotate sample by -phase
                // (s.re + j*s.im) * (cos - j*sin)
                let rot_re = s.re * cos_val + s.im * sin_val;
                let rot_im = -s.re * sin_val + s.im * cos_val;

                out_slice[k] = Complex32::new(rot_re, rot_im);

                // Phase error detector for BPSK: e = sign(rot_re) * rot_im
                let sgn = if rot_re >= 0.0 { 1.0 } else { -1.0 };
                let err = sgn * rot_im;

                // Update frequency and phase
                freq += beta * err;
                if freq > max_freq {
                    freq = max_freq;
                } else if freq < min_freq {
                    freq = min_freq;
                }

                phase += freq + alpha * err;
                while phase > PI {
                    phase -= 2.0 * PI;
                }
                while phase < -PI {
                    phase += 2.0 * PI;
                }
            }

            self.phase = phase;
            self.freq = freq;
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
