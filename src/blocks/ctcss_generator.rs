use anyhow::Result;
use futuresdr::prelude::*;
use futuresdr::runtime::Pmt;
use std::f32::consts::PI;

#[derive(Block)]
#[message_inputs(set_tone)]
pub struct CtcssGenerator {
    tone: f32,
    sample_rate: f32,
    phase: f32,
    amplitude: f32,
    #[input]
    input: DefaultCpuReader<f32>,
    #[output]
    output: DefaultCpuWriter<f32>,
}

impl CtcssGenerator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            tone: 0.0,
            sample_rate,
            phase: 0.0,
            amplitude: 0.1,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }

    async fn set_tone(
        &mut self,
        _io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
        p: Pmt,
    ) -> Result<Pmt> {
        if let Pmt::F32(tone) = p {
            self.tone = tone;
            Ok(Pmt::Ok)
        } else {
            Ok(Pmt::InvalidValue)
        }
    }
}

#[doc(hidden)]
impl Kernel for CtcssGenerator {
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
                if self.tone > 0.0 {
                    let phase_inc = 2.0 * PI * self.tone / self.sample_rate;
                    for (in_sample, out_sample) in i.iter().zip(o.iter_mut()).take(m) {
                        *out_sample = *in_sample + self.amplitude * self.phase.sin();
                        self.phase += phase_inc;
                        if self.phase > 2.0 * PI {
                            self.phase -= 2.0 * PI;
                        }
                    }
                } else {
                    o[..m].copy_from_slice(&i[..m]);
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
