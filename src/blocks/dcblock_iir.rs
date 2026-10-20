use anyhow::Result;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;

const DEFAULT_R: f32 = 0.999;

#[derive(Block)]
pub struct DcBlockFf<
    I: CpuBufferReader<Item = f32> = DefaultCpuReader<f32>,
    O: CpuBufferWriter<Item = f32> = DefaultCpuWriter<f32>,
> {
    r: f32,
    xm1: f32,
    ym1: f32,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl DcBlockFf<DefaultCpuReader<f32>, DefaultCpuWriter<f32>> {
    pub fn new(r: f32) -> Self {
        let r = if r <= 0.0 || r >= 1.0 { DEFAULT_R } else { r };
        Self {
            r,
            xm1: 0.0,
            ym1: 0.0,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for DcBlockFf<DefaultCpuReader<f32>, DefaultCpuWriter<f32>> {
    fn default() -> Self {
        Self::new(DEFAULT_R)
    }
}

#[doc(hidden)]
impl<I, O> Kernel for DcBlockFf<I, O>
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
            let r = self.r;
            let mut xm1 = self.xm1;
            let mut ym1 = self.ym1;

            for (src, dst) in i[..m].iter().zip(o[..m].iter_mut()) {
                let x = if src.is_nan() { 0.0 } else { *src };
                // y[n] = (x[n] - x[n-1]) + r * y[n-1]
                let diff = f32::algebraic_sub(x, xm1);
                let y = r.mul_add(ym1, diff);

                xm1 = x;
                ym1 = y;
                *dst = y;
            }

            self.xm1 = xm1;
            self.ym1 = ym1;

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
