use anyhow::Result;
use futuresdr::prelude::*;

#[derive(Block)]
pub struct DCBlocker<
    A: Copy + Send + 'static,
    I: CpuBufferReader<Item = A> = DefaultCpuReader<A>,
    O: CpuBufferWriter<Item = A> = DefaultCpuWriter<A>,
> {
    last_dc_level: f32,
    min_bufsize: usize,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl<A, I, O> DCBlocker<A, I, O>
where
    A: Copy + Send + 'static,
    I: CpuBufferReader<Item = A>,
    O: CpuBufferWriter<Item = A>,
{
    pub fn new(min_bufsize: usize) -> Self {
        Self {
            last_dc_level: 0.0,
            min_bufsize,
            input: I::default(),
            output: O::default(),
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for DCBlocker<f32, I, O>
where
    I: CpuBufferReader<Item = f32>,
    O: CpuBufferWriter<Item = f32>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let m;
        let ilen;
        {
            let i = self.input.slice();
            let o = self.output.slice();

            ilen = i.len();
            m = std::cmp::min(ilen, o.len());

            if m > self.min_bufsize {
                let sum: f32 = i.iter().sum();
                let avg = sum / (ilen as f32);
                let avgdiff = avg - self.last_dc_level;

                let input_size = m as f32;
                for (index, (v, r)) in i.iter().zip(o.iter_mut()).enumerate() {
                    let linear_dc_level_change = avgdiff * ((index as f32) / input_size);
                    let dc_removal_level = self.last_dc_level + linear_dc_level_change;
                    *r = *v - dc_removal_level;
                }

                self.last_dc_level = avg;
            }
        }

        if m > self.min_bufsize {
            self.input.consume(m);
            self.output.produce(m);
        }

        if self.input.finished() && m == ilen {
            io.finished = true;
        }

        Ok(())
    }
}
