use anyhow::Result;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;

#[derive(Block)]
pub struct DifferentialEncoderU8<
    I: CpuBufferReader<Item = u8> = DefaultCpuReader<u8>,
    O: CpuBufferWriter<Item = u8> = DefaultCpuWriter<u8>,
> {
    last: u8,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl DifferentialEncoderU8<DefaultCpuReader<u8>, DefaultCpuWriter<u8>> {
    pub fn new() -> Self {
        Self {
            last: 0,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for DifferentialEncoderU8<DefaultCpuReader<u8>, DefaultCpuWriter<u8>> {
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
impl<I, O> Kernel for DifferentialEncoderU8<I, O>
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

        let m = cmp::min(i.len(), o.len());
        if m > 0 {
            let mut last = self.last;
            for (k, &s) in i.iter().enumerate().take(m) {
                last ^= s & 1;
                o[k] = last;
            }
            self.last = last;
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

#[derive(Block)]
pub struct DifferentialDecoderU8<
    I: CpuBufferReader<Item = u8> = DefaultCpuReader<u8>,
    O: CpuBufferWriter<Item = u8> = DefaultCpuWriter<u8>,
> {
    last: u8,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl DifferentialDecoderU8<DefaultCpuReader<u8>, DefaultCpuWriter<u8>> {
    pub fn new() -> Self {
        Self {
            last: 0,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for DifferentialDecoderU8<DefaultCpuReader<u8>, DefaultCpuWriter<u8>> {
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
impl<I, O> Kernel for DifferentialDecoderU8<I, O>
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

        let m = cmp::min(i.len(), o.len());
        if m > 0 {
            let mut last = self.last;
            for (k, &s) in i.iter().enumerate().take(m) {
                let bit = s & 1;
                o[k] = bit ^ last;
                last = bit;
            }
            self.last = last;
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

#[derive(Block)]
pub struct InvertU8<
    I: CpuBufferReader<Item = u8> = DefaultCpuReader<u8>,
    O: CpuBufferWriter<Item = u8> = DefaultCpuWriter<u8>,
> {
    #[input]
    input: I,
    #[output]
    output: O,
}

impl InvertU8<DefaultCpuReader<u8>, DefaultCpuWriter<u8>> {
    pub fn new() -> Self {
        Self {
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for InvertU8<DefaultCpuReader<u8>, DefaultCpuWriter<u8>> {
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
impl<I, O> Kernel for InvertU8<I, O>
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

        let m = cmp::min(i.len(), o.len());
        if m > 0 {
            for (k, &s) in i.iter().enumerate().take(m) {
                o[k] = 1 - (s & 1);
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
