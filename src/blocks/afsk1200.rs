use anyhow::Result;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;
use std::f32::consts::PI;

/// AFSK 1200 Bell 202 Modulator: bytes to audio f32 at 48 kSPS
/// Mark: 1200 Hz (bit 1)
/// Space: 2200 Hz (bit 0)
#[derive(Block)]
pub struct Afsk1200ModU8F<
    I: CpuBufferReader<Item = u8> = DefaultCpuReader<u8>,
    O: CpuBufferWriter<Item = f32> = DefaultCpuWriter<f32>,
> {
    phase: f32,
    sample_rate: f32,
    samples_per_bit: usize,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl Afsk1200ModU8F<DefaultCpuReader<u8>, DefaultCpuWriter<f32>> {
    pub fn new(sample_rate: f32) -> Self {
        let sample_rate = if sample_rate <= 0.0 {
            48000.0
        } else {
            sample_rate
        };
        let baud_rate = 1200.0;
        let samples_per_bit = (sample_rate / baud_rate).round() as usize;

        Self {
            phase: 0.0,
            sample_rate,
            samples_per_bit: samples_per_bit.max(1),
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for Afsk1200ModU8F<I, O>
where
    I: CpuBufferReader<Item = u8>,
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

        // 1 byte = 8 bits -> 8 * samples_per_bit samples
        let samples_per_byte = 8 * self.samples_per_bit;
        let max_bytes = out_slice.len() / samples_per_byte;
        let bytes_to_process = cmp::min(in_slice.len(), max_bytes);

        if bytes_to_process > 0 {
            let mut out_idx = 0;
            let mut phase = self.phase;

            let freq_mark = 1200.0;
            let freq_space = 2200.0;
            let inc_mark = 2.0 * PI * freq_mark / self.sample_rate;
            let inc_space = 2.0 * PI * freq_space / self.sample_rate;

            for &byte in &in_slice[..bytes_to_process] {
                for bit_idx in 0..8 {
                    let bit = (byte >> (7 - bit_idx)) & 1;
                    let phase_inc = if bit == 1 { inc_mark } else { inc_space };

                    for _ in 0..self.samples_per_bit {
                        let (sin_val, _) = crate::math::fast_sincos(phase);
                        out_slice[out_idx] = sin_val;
                        out_idx += 1;
                        phase = f32::algebraic_add(phase, phase_inc);
                        if phase > 2.0 * PI {
                            phase = f32::algebraic_sub(phase, 2.0 * PI);
                        }
                    }
                }
            }

            self.phase = phase;
            self.input.consume(bytes_to_process);
            self.output.produce(out_idx);
        }

        if !self.input.slice().is_empty() && self.output.slice().len() >= samples_per_byte {
            io.call_again = true;
        }

        if self.input.slice().is_empty() && self.input.finished() {
            io.finished = true;
        }

        Ok(())
    }
}

/// AFSK 1200 Bell 202 Demodulator: audio f32 at 48 kSPS to bytes (u8)
#[derive(Block)]
pub struct Afsk1200DemodFU8<
    I: CpuBufferReader<Item = f32> = DefaultCpuReader<f32>,
    O: CpuBufferWriter<Item = u8> = DefaultCpuWriter<u8>,
> {
    sample_rate: f32,
    samples_per_bit: usize,
    sample_accum: usize,
    i_mark: f32,
    q_mark: f32,
    i_space: f32,
    q_space: f32,
    bit_buffer: u8,
    bits_in_buffer: usize,
    phase_mark: f32,
    phase_space: f32,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl Afsk1200DemodFU8<DefaultCpuReader<f32>, DefaultCpuWriter<u8>> {
    pub fn new(sample_rate: f32) -> Self {
        let sample_rate = if sample_rate <= 0.0 {
            48000.0
        } else {
            sample_rate
        };
        let samples_per_bit = (sample_rate / 1200.0).round() as usize;

        Self {
            sample_rate,
            samples_per_bit: samples_per_bit.max(1),
            sample_accum: 0,
            i_mark: 0.0,
            q_mark: 0.0,
            i_space: 0.0,
            q_space: 0.0,
            bit_buffer: 0,
            bits_in_buffer: 0,
            phase_mark: 0.0,
            phase_space: 0.0,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for Afsk1200DemodFU8<I, O>
where
    I: CpuBufferReader<Item = f32>,
    O: CpuBufferWriter<Item = u8>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let in_slice = self.input.slice();
        let out_slice = self.output.slice();
        let max_out = out_slice.len();

        let inc_mark = 2.0 * PI * 1200.0 / self.sample_rate;
        let inc_space = 2.0 * PI * 2200.0 / self.sample_rate;

        let mut in_consumed = 0;
        let mut out_produced = 0;

        while in_consumed < in_slice.len() && out_produced < out_slice.len() {
            let s = in_slice[in_consumed];
            in_consumed += 1;

            let (sin_m, cos_m) = crate::math::fast_sincos(self.phase_mark);
            let (sin_s, cos_s) = crate::math::fast_sincos(self.phase_space);

            self.i_mark = f32::algebraic_add(self.i_mark, f32::algebraic_mul(s, cos_m));
            self.q_mark = f32::algebraic_add(self.q_mark, f32::algebraic_mul(s, sin_m));

            self.i_space = f32::algebraic_add(self.i_space, f32::algebraic_mul(s, cos_s));
            self.q_space = f32::algebraic_add(self.q_space, f32::algebraic_mul(s, sin_s));

            self.phase_mark = f32::algebraic_add(self.phase_mark, inc_mark);
            if self.phase_mark > 2.0 * PI {
                self.phase_mark = f32::algebraic_sub(self.phase_mark, 2.0 * PI);
            }

            self.phase_space = f32::algebraic_add(self.phase_space, inc_space);
            if self.phase_space > 2.0 * PI {
                self.phase_space = f32::algebraic_sub(self.phase_space, 2.0 * PI);
            }
            self.sample_accum += 1;

            if self.sample_accum >= self.samples_per_bit {
                let energy_mark = f32::algebraic_add(
                    f32::algebraic_mul(self.i_mark, self.i_mark),
                    f32::algebraic_mul(self.q_mark, self.q_mark),
                );
                let energy_space = f32::algebraic_add(
                    f32::algebraic_mul(self.i_space, self.i_space),
                    f32::algebraic_mul(self.q_space, self.q_space),
                );
                let bit = if energy_mark >= energy_space {
                    1u8
                } else {
                    0u8
                };
                self.i_mark = 0.0;
                self.q_mark = 0.0;
                self.i_space = 0.0;
                self.q_space = 0.0;
                self.sample_accum = 0;

                self.bit_buffer = (self.bit_buffer << 1) | bit;
                self.bits_in_buffer += 1;

                if self.bits_in_buffer == 8 {
                    out_slice[out_produced] = self.bit_buffer;
                    out_produced += 1;
                    self.bit_buffer = 0;
                    self.bits_in_buffer = 0;
                }
            }
        }

        if in_consumed > 0 {
            self.input.consume(in_consumed);
        }
        if out_produced > 0 {
            self.output.produce(out_produced);
        }

        if !self.input.slice().is_empty() && out_produced < max_out {
            io.call_again = true;
        }

        if self.input.slice().is_empty() && self.input.finished() {
            io.finished = true;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_afsk1200_parameters() {
        let mod_blk = Afsk1200ModU8F::new(48000.0);
        assert_eq!(mod_blk.samples_per_bit, 40);

        let demod_blk = Afsk1200DemodFU8::new(48000.0);
        assert_eq!(demod_blk.samples_per_bit, 40);
    }

    #[test]
    fn test_afsk1200_mod_demod_roundtrip() -> Result<()> {
        use futuresdr::blocks::{VectorSink, VectorSource};
        use futuresdr::runtime::Flowgraph;
        use futuresdr::runtime::Runtime;

        let input_bytes = vec![0x55, 0xAA, 0x12, 0x34, 0xFE, 0xCA];
        let mut fg = Flowgraph::new();

        let src = fg.add(VectorSource::<u8>::new(input_bytes.clone()))?;
        let modulator = fg.add(Afsk1200ModU8F::new(48000.0))?;
        let demodulator = fg.add(Afsk1200DemodFU8::new(48000.0))?;
        let snk = fg.add(VectorSink::<u8>::new(input_bytes.len()))?;

        fg.stream_dyn(src.id(), "output", modulator.id(), "input")?;
        fg.stream_dyn(modulator.id(), "output", demodulator.id(), "input")?;
        fg.stream_dyn(demodulator.id(), "output", snk.id(), "input")?;

        let term_fg = Runtime::new().run(fg)?;
        let snk_block = term_fg.block(&snk)?;
        let received = snk_block.items();

        assert_eq!(received, &input_bytes);
        Ok(())
    }

    #[test]
    fn test_afsk1200_demod_backpressure() -> Result<()> {
        use futuresdr::blocks::{VectorSink, VectorSource};
        use futuresdr::runtime::Flowgraph;
        use futuresdr::runtime::Runtime;

        let input_bytes: Vec<u8> = (0..50).map(|i| (i * 37 + 13) as u8).collect();
        let mut fg = Flowgraph::new();

        let src = fg.add(VectorSource::<u8>::new(input_bytes.clone()))?;
        let modulator = fg.add(Afsk1200ModU8F::new(48000.0))?;
        let demodulator = fg.add(Afsk1200DemodFU8::new(48000.0))?;
        let snk = fg.add(VectorSink::<u8>::new(10))?; // Small capacity forcing backpressure & multi-cycle consumption

        fg.stream_dyn(src.id(), "output", modulator.id(), "input")?;
        fg.stream_dyn(modulator.id(), "output", demodulator.id(), "input")?;
        fg.stream_dyn(demodulator.id(), "output", snk.id(), "input")?;

        let term_fg = Runtime::new().run(fg)?;
        let snk_block = term_fg.block(&snk)?;
        let received = snk_block.items();

        assert_eq!(received, &input_bytes);
        Ok(())
    }
}
