use anyhow::Result;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;

#[derive(Block)]
pub struct AddDcOffsetCc<
    I: CpuBufferReader<Item = Complex32> = DefaultCpuReader<Complex32>,
    O: CpuBufferWriter<Item = Complex32> = DefaultCpuWriter<Complex32>,
> {
    offset: Complex32,
    scale: f32,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl AddDcOffsetCc<DefaultCpuReader<Complex32>, DefaultCpuWriter<Complex32>> {
    pub fn new(offset: Complex32) -> Self {
        Self {
            offset,
            scale: 1.0,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }

    pub fn new_am() -> Self {
        Self {
            offset: Complex32::new(0.5, 0.0),
            scale: 0.5,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for AddDcOffsetCc<DefaultCpuReader<Complex32>, DefaultCpuWriter<Complex32>> {
    fn default() -> Self {
        Self::new_am()
    }
}

#[doc(hidden)]
impl<I, O> Kernel for AddDcOffsetCc<I, O>
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
            let off = self.offset;
            let scale = self.scale;

            if (scale - 1.0).abs() < 1e-6 {
                for k in 0..m {
                    let s = i[k];
                    let re = unsafe { core::intrinsics::fadd_fast(s.re, off.re) };
                    let im = unsafe { core::intrinsics::fadd_fast(s.im, off.im) };
                    o[k] = Complex32::new(re, im);
                }
            } else {
                for k in 0..m {
                    let s = i[k];
                    let re_scaled = unsafe { core::intrinsics::fmul_fast(s.re, scale) };
                    let im_scaled = unsafe { core::intrinsics::fmul_fast(s.im, scale) };
                    let re = unsafe { core::intrinsics::fadd_fast(re_scaled, off.re) };
                    let im = unsafe { core::intrinsics::fadd_fast(im_scaled, off.im) };
                    o[k] = Complex32::new(re, im);
                }
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
