use anyhow::Result;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;

#[derive(Block)]
pub struct Mono2StereoS16<
    I: CpuBufferReader<Item = i16> = DefaultCpuReader<i16>,
    O: CpuBufferWriter<Item = i16> = DefaultCpuWriter<i16>,
> {
    #[input]
    input: I,
    #[output]
    output: O,
}

impl Mono2StereoS16<DefaultCpuReader<i16>, DefaultCpuWriter<i16>> {
    pub fn new() -> Self {
        Self {
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for Mono2StereoS16<DefaultCpuReader<i16>, DefaultCpuWriter<i16>> {
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
impl<I, O> Kernel for Mono2StereoS16<I, O>
where
    I: CpuBufferReader<Item = i16>,
    O: CpuBufferWriter<Item = i16>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let i = self.input.slice();
        let o = self.output.slice();

        let max_in_by_out = o.len() / 2;
        let n = cmp::min(i.len(), max_in_by_out);

        if n > 0 {
            for (k, &val) in i.iter().enumerate().take(n) {
                let out_idx = k * 2;
                o[out_idx] = val;
                o[out_idx + 1] = val;
            }
            self.input.consume(n);
            self.output.produce(n * 2);
        }

        if !self.input.slice().is_empty() && self.output.slice().len() >= 2 {
            io.call_again = true;
        }

        if self.input.slice().is_empty() && self.input.finished() {
            io.finished = true;
        }

        Ok(())
    }
}
