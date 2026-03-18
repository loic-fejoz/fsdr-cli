use anyhow::Result;
use futuresdr::prelude::*;
use futuresdr::runtime::Pmt;

#[derive(Block)]
#[message_outputs(pdus)]
pub struct FixedlenToPdu<I: CpuBufferReader<Item = u8> = DefaultCpuReader<u8>> {
    packet_len: usize,
    buffer: Vec<u8>,
    #[input]
    input: I,
}

impl<I: CpuBufferReader<Item = u8>> FixedlenToPdu<I> {
    pub fn new(packet_len: usize) -> Self {
        Self {
            packet_len,
            buffer: Vec::with_capacity(packet_len),
            input: I::default(),
        }
    }
}

#[doc(hidden)]
impl<I: CpuBufferReader<Item = u8>> Kernel for FixedlenToPdu<I> {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i = self.input.slice();
        if i.is_empty() {
            if self.input.finished() {
                io.finished = true;
            }
            return Ok(());
        }

        let mut consumed = 0;
        for &byte in i {
            self.buffer.push(byte);
            consumed += 1;
            if self.buffer.len() == self.packet_len {
                mio.post("pdus", Pmt::Blob(self.buffer.clone())).await?;
                self.buffer.clear();
            }
        }

        self.input.consume(consumed);
        if self.input.finished() {
            io.finished = true;
        }

        Ok(())
    }
}
