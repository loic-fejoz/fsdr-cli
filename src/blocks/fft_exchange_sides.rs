use anyhow::Result;
use futuresdr::runtime::dev::prelude::*;

#[derive(Block)]
pub struct FftExchangeSidesFf<
    I: CpuBufferReader<Item = f32> = DefaultCpuReader<f32>,
    O: CpuBufferWriter<Item = f32> = DefaultCpuWriter<f32>,
> {
    fft_size: usize,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl FftExchangeSidesFf<DefaultCpuReader<f32>, DefaultCpuWriter<f32>> {
    pub fn new(fft_size: usize) -> Self {
        assert!(
            fft_size > 0 && fft_size.is_multiple_of(2),
            "fft_size must be non-zero and even for fft_exchange_sides_ff"
        );
        Self {
            fft_size,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for FftExchangeSidesFf<I, O>
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
        let in_slice = self.input.slice();
        let out_slice = self.output.slice();

        let half = self.fft_size / 2;
        let mut in_offset = 0;
        let mut out_offset = 0;

        while in_slice.len() - in_offset >= self.fft_size
            && out_slice.len() - out_offset >= self.fft_size
        {
            out_slice[out_offset..out_offset + half]
                .copy_from_slice(&in_slice[in_offset + half..in_offset + self.fft_size]);
            out_slice[out_offset + half..out_offset + self.fft_size]
                .copy_from_slice(&in_slice[in_offset..in_offset + half]);

            in_offset += self.fft_size;
            out_offset += self.fft_size;
        }

        if in_offset > 0 {
            self.input.consume(in_offset);
        }
        if out_offset > 0 {
            self.output.produce(out_offset);
        }

        if self.input.slice().len() >= self.fft_size && self.output.slice().len() >= self.fft_size {
            io.call_again = true;
        }

        if self.input.finished() && self.input.slice().len() < self.fft_size {
            io.finished = true;
        }

        Ok(())
    }
}
