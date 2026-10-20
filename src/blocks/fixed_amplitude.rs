use anyhow::Result;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;

#[derive(Block)]
pub struct FixedAmplitudeCc<
    I: CpuBufferReader<Item = Complex32> = DefaultCpuReader<Complex32>,
    O: CpuBufferWriter<Item = Complex32> = DefaultCpuWriter<Complex32>,
> {
    amplitude: f32,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl FixedAmplitudeCc<DefaultCpuReader<Complex32>, DefaultCpuWriter<Complex32>> {
    pub fn new(amplitude: f32) -> Self {
        Self {
            amplitude: if amplitude <= 0.0 { 1.0 } else { amplitude },
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for FixedAmplitudeCc<DefaultCpuReader<Complex32>, DefaultCpuWriter<Complex32>> {
    fn default() -> Self {
        Self::new(1.0)
    }
}

#[doc(hidden)]
impl<I, O> Kernel for FixedAmplitudeCc<I, O>
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
        let i = self.input.slice();
        let o = self.output.slice();

        let m = cmp::min(i.len(), o.len());
        if m > 0 {
            let amp = self.amplitude;
            for (s, dst) in i[..m].iter().zip(o[..m].iter_mut()) {
                let norm_sqr = s.re.mul_add(s.re, s.im.algebraic_mul(s.im));
                if norm_sqr > 1e-24 {
                    let factor = amp / norm_sqr.sqrt();
                    *dst = Complex32::new(
                        f32::algebraic_mul(s.re, factor),
                        f32::algebraic_mul(s.im, factor),
                    );
                } else {
                    *dst = Complex32::new(amp, 0.0);
                }
            }
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
