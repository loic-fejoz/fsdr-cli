use anyhow::Result;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;

#[derive(Block)]
pub struct BfskDemodCf<
    I: CpuBufferReader<Item = Complex32> = DefaultCpuReader<Complex32>,
    O: CpuBufferWriter<Item = f32> = DefaultCpuWriter<f32>,
> {
    last_sample: Complex32,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl BfskDemodCf<DefaultCpuReader<Complex32>, DefaultCpuWriter<f32>> {
    pub fn new() -> Self {
        Self {
            last_sample: Complex32::new(1.0, 0.0),
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for BfskDemodCf<DefaultCpuReader<Complex32>, DefaultCpuWriter<f32>> {
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
impl<I, O> Kernel for BfskDemodCf<I, O>
where
    I: CpuBufferReader<Item = Complex32>,
    O: CpuBufferWriter<Item = f32>,
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
            let mut last = self.last_sample;
            for (k, &s) in i.iter().enumerate().take(m) {
                // Quad demod: angle of s * last.conj()
                let prod = s * last.conj();
                let dphi = prod.arg();
                o[k] = if dphi > 0.0 { 1.0 } else { 0.0 };
                last = s;
            }
            self.last_sample = last;
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
