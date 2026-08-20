use anyhow::Result;
use futuresdr::num_complex::Complex32;
use futuresdr::prelude::*;
use futuresdr::runtime::Pmt;
use std::f32::consts::PI;

#[derive(Block)]
#[message_inputs(set_frequency)]
pub struct FrequencyShifter<
    I: CpuBufferReader<Item = Complex32> = DefaultCpuReader<Complex32>,
    O: CpuBufferWriter<Item = Complex32> = DefaultCpuWriter<Complex32>,
> {
    freq: f32,
    sample_rate: f32,
    phase: f32,
    phase_inc: f32,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl<I, O> FrequencyShifter<I, O>
where
    I: CpuBufferReader<Item = Complex32>,
    O: CpuBufferWriter<Item = Complex32>,
{
    pub fn new(freq: f32, sample_rate: f32) -> Self {
        Self {
            freq,
            sample_rate,
            phase: 0.0,
            phase_inc: 2.0 * PI * freq / sample_rate,
            input: I::default(),
            output: O::default(),
        }
    }

    async fn set_frequency(
        &mut self,
        _io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
        p: Pmt,
    ) -> Result<Pmt> {
        let freq = match p {
            Pmt::F32(f) => f,
            Pmt::F64(f) => f as f32,
            Pmt::U32(f) => f as f32,
            Pmt::U64(f) => f as f32,
            _ => return Ok(Pmt::InvalidValue),
        };
        self.freq = freq;
        self.phase_inc = 2.0 * PI * freq / self.sample_rate;
        Ok(Pmt::Ok)
    }
}

#[doc(hidden)]
impl Kernel for FrequencyShifter {
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

            if m > 0 {
                for (in_sample, out_sample) in i.iter().zip(o.iter_mut()).take(m) {
                    let osc = Complex32::new(self.phase.cos(), self.phase.sin());
                    *out_sample = *in_sample * osc;
                    self.phase += self.phase_inc;
                    if self.phase > 2.0 * PI {
                        self.phase -= 2.0 * PI;
                    } else if self.phase < -2.0 * PI {
                        self.phase += 2.0 * PI;
                    }
                }
            }
        }

        if m > 0 {
            self.input.consume(m);
            self.output.produce(m);
        }

        if self.input.finished() && m == ilen {
            io.finished = true;
        }

        Ok(())
    }
}
