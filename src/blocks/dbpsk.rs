use anyhow::Result;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;
use std::f32::consts::PI;

#[derive(Block)]
pub struct DBPskDecoder<
    I: CpuBufferReader<Item = Complex32> = DefaultCpuReader<Complex32>,
    O: CpuBufferWriter<Item = u8> = DefaultCpuWriter<u8>,
> {
    last_phase: f32,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl DBPskDecoder<DefaultCpuReader<Complex32>, DefaultCpuWriter<u8>> {
    pub fn new() -> Self {
        Self {
            last_phase: 0.0,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for DBPskDecoder<DefaultCpuReader<Complex32>, DefaultCpuWriter<u8>> {
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
impl<I, O> Kernel for DBPskDecoder<I, O>
where
    I: CpuBufferReader<Item = Complex32>,
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
            let mut last_phase = self.last_phase;
            for (k, &s) in i.iter().enumerate().take(m) {
                let mut phase = s.arg();
                if phase.is_nan() {
                    phase = 0.0;
                }
                let mut dphase = phase - last_phase;
                while dphase < -PI {
                    dphase += 2.0 * PI;
                }
                while dphase >= PI {
                    dphase -= 2.0 * PI;
                }
                let bit = if dphase > (PI / 2.0) || dphase < (-PI / 2.0) {
                    0u8
                } else {
                    1u8
                };
                o[k] = bit;
                last_phase = phase;
            }
            self.last_phase = last_phase;
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
