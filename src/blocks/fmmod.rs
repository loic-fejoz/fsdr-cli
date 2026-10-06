use anyhow::Result;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;
use std::f32::consts::PI;

#[derive(Block)]
pub struct FmModFc<
    I: CpuBufferReader<Item = f32> = DefaultCpuReader<f32>,
    O: CpuBufferWriter<Item = Complex32> = DefaultCpuWriter<Complex32>,
> {
    phase: f32,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl FmModFc<DefaultCpuReader<f32>, DefaultCpuWriter<Complex32>> {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for FmModFc<DefaultCpuReader<f32>, DefaultCpuWriter<Complex32>> {
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
impl<I, O> Kernel for FmModFc<I, O>
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
        let i = self.input.slice();
        let o = self.output.slice();

        let m = cmp::min(i.len(), o.len());
        if m > 0 {
            let mut phase = self.phase;
            for (src, dst) in i[..m].iter().zip(o[..m].iter_mut()) {
                phase = f32::algebraic_add(phase, *src);
                while phase > PI {
                    phase -= 2.0 * PI;
                }
                while phase < -PI {
                    phase += 2.0 * PI;
                }
                let (sin_val, cos_val) = phase.sin_cos();
                *dst = Complex32::new(cos_val, sin_val);
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
