use anyhow::Result;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;

#[derive(Block)]
pub struct RepeatU8<
    I: CpuBufferReader<Item = u8> = DefaultCpuReader<u8>,
    O: CpuBufferWriter<Item = u8> = DefaultCpuWriter<u8>,
> {
    repeat: usize,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl RepeatU8<DefaultCpuReader<u8>, DefaultCpuWriter<u8>> {
    pub fn new(repeat: usize) -> Self {
        assert!(repeat > 0, "repeat must be greater than 0");
        Self {
            repeat,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for RepeatU8<I, O>
where
    I: CpuBufferReader<Item = u8>,
    O: CpuBufferWriter<Item = u8>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let i = self.input.slice();
        let o = self.output.slice();

        let rep = self.repeat;
        let max_in_by_out = o.len() / rep;
        let n = cmp::min(i.len(), max_in_by_out);

        if n > 0 {
            for (k, &val) in i.iter().enumerate().take(n) {
                let out_offset = k * rep;
                o[out_offset..out_offset + rep].fill(val);
            }
            self.input.consume(n);
            self.output.produce(n * rep);
        }

        if !self.input.slice().is_empty() && self.output.slice().len() >= rep {
            io.call_again = true;
        }

        if self.input.slice().is_empty() && self.input.finished() {
            io.finished = true;
        }

        Ok(())
    }
}
