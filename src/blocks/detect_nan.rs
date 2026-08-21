use anyhow::Result;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;

#[derive(Block)]
pub struct DetectNanFf<
    I: CpuBufferReader<Item = f32> = DefaultCpuReader<f32>,
    O: CpuBufferWriter<Item = f32> = DefaultCpuWriter<f32>,
> {
    #[input]
    input: I,
    #[output]
    output: O,
}

impl DetectNanFf<DefaultCpuReader<f32>, DefaultCpuWriter<f32>> {
    pub fn new() -> Self {
        Self {
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for DetectNanFf<DefaultCpuReader<f32>, DefaultCpuWriter<f32>> {
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
impl<I, O> Kernel for DetectNanFf<I, O>
where
    I: CpuBufferReader<Item = f32>,
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
            for (k, &s) in i.iter().enumerate().take(m) {
                o[k] = if s.is_nan() || s.is_infinite() {
                    0.0
                } else {
                    s
                };
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
