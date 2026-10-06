use anyhow::Result;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;

#[derive(Block)]
pub struct AddConstCc<
    I: CpuBufferReader<Item = Complex32> = DefaultCpuReader<Complex32>,
    O: CpuBufferWriter<Item = Complex32> = DefaultCpuWriter<Complex32>,
> {
    constant: Complex32,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl AddConstCc<DefaultCpuReader<Complex32>, DefaultCpuWriter<Complex32>> {
    pub fn new(constant: Complex32) -> Self {
        Self {
            constant,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for AddConstCc<DefaultCpuReader<Complex32>, DefaultCpuWriter<Complex32>> {
    fn default() -> Self {
        Self::new(Complex32::new(0.0, 0.0))
    }
}

#[doc(hidden)]
impl<I, O> Kernel for AddConstCc<I, O>
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
            let c = self.constant;
            for (src, dst) in i[..m].iter().zip(o[..m].iter_mut()) {
                *dst = Complex32::new(
                    f32::algebraic_add(src.re, c.re),
                    f32::algebraic_add(src.im, c.im),
                );
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
