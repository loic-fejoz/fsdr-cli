use anyhow::Result;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;
use std::f32::consts::PI;

/// QPSK / OQPSK Modulator: bytes (u8) to Complex32 symbols
#[derive(Block)]
pub struct QpskModulatorU8C<
    I: CpuBufferReader<Item = u8> = DefaultCpuReader<u8>,
    O: CpuBufferWriter<Item = Complex32> = DefaultCpuWriter<Complex32>,
> {
    oqpsk: bool,
    sps: usize,
    prev_q: f32,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl QpskModulatorU8C<DefaultCpuReader<u8>, DefaultCpuWriter<Complex32>> {
    pub fn new(oqpsk: bool, sps: usize) -> Self {
        Self {
            oqpsk,
            sps: sps.max(1),
            prev_q: 0.0,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for QpskModulatorU8C<I, O>
where
    I: CpuBufferReader<Item = u8>,
    O: CpuBufferWriter<Item = Complex32>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let in_slice = self.input.slice();
        let out_slice = self.output.slice();

        // 1 byte = 8 bits = 4 symbols. Each symbol takes `sps` output samples.
        let samples_per_byte = 4 * self.sps;
        let max_bytes = out_slice.len() / samples_per_byte;
        let bytes_to_process = cmp::min(in_slice.len(), max_bytes);

        if bytes_to_process > 0 {
            let inv_sqrt2 = 1.0 / std::f32::consts::SQRT_2;
            let mut out_idx = 0;

            for &byte in &in_slice[..bytes_to_process] {
                for sym_idx in 0..4 {
                    let b0 = (byte >> (7 - sym_idx * 2)) & 1;
                    let b1 = (byte >> (6 - sym_idx * 2)) & 1;

                    let cur_i = if b0 == 0 { inv_sqrt2 } else { -inv_sqrt2 };
                    let cur_q = if b1 == 0 { inv_sqrt2 } else { -inv_sqrt2 };

                    for s in 0..self.sps {
                        let out_q = if self.oqpsk && s < (self.sps / 2).max(1) {
                            self.prev_q
                        } else {
                            cur_q
                        };
                        out_slice[out_idx] = Complex32::new(cur_i, out_q);
                        out_idx += 1;
                    }
                    self.prev_q = cur_q;
                }
            }

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

/// QPSK / OQPSK Demodulator: Complex32 samples to bytes (u8)
#[derive(Block)]
pub struct QpskDemodCU8<
    I: CpuBufferReader<Item = Complex32> = DefaultCpuReader<Complex32>,
    O: CpuBufferWriter<Item = u8> = DefaultCpuWriter<u8>,
> {
    oqpsk: bool,
    bit_buffer: u8,
    bits_in_buffer: usize,
    prev_q: f32,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl QpskDemodCU8<DefaultCpuReader<Complex32>, DefaultCpuWriter<u8>> {
    pub fn new(oqpsk: bool) -> Self {
        Self {
            oqpsk,
            bit_buffer: 0,
            bits_in_buffer: 0,
            prev_q: 0.0,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for QpskDemodCU8<I, O>
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
        let in_slice = self.input.slice();
        let out_slice = self.output.slice();

        let mut in_consumed = 0;
        let mut out_produced = 0;

        while in_consumed < in_slice.len() && out_produced < out_slice.len() {
            let sample = in_slice[in_consumed];
            in_consumed += 1;

            let b0 = if sample.re >= 0.0 { 0u8 } else { 1u8 };
            let q_val = if self.oqpsk {
                let q = self.prev_q;
                self.prev_q = sample.im;
                q
            } else {
                sample.im
            };
            let b1 = if q_val >= 0.0 { 0u8 } else { 1u8 };

            self.bit_buffer = (self.bit_buffer << 1) | b0;
            self.bits_in_buffer += 1;
            if self.bits_in_buffer == 8 {
                out_slice[out_produced] = self.bit_buffer;
                out_produced += 1;
                self.bit_buffer = 0;
                self.bits_in_buffer = 0;
                if out_produced >= out_slice.len() {
                    break;
                }
            }

            self.bit_buffer = (self.bit_buffer << 1) | b1;
            self.bits_in_buffer += 1;
            if self.bits_in_buffer == 8 {
                out_slice[out_produced] = self.bit_buffer;
                out_produced += 1;
                self.bit_buffer = 0;
                self.bits_in_buffer = 0;
            }
        }

        if in_consumed > 0 {
            self.input.consume(in_consumed);
        }
        if out_produced > 0 {
            self.output.produce(out_produced);
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

/// 4-phase Costas loop for QPSK and OQPSK tracking
#[derive(Block)]
pub struct QpskCostasLoopCc<
    I: CpuBufferReader<Item = Complex32> = DefaultCpuReader<Complex32>,
    O: CpuBufferWriter<Item = Complex32> = DefaultCpuWriter<Complex32>,
> {
    phase: f32,
    freq: f32,
    alpha: f32,
    beta: f32,
    max_freq: f32,
    min_freq: f32,
    oqpsk: bool,
    prev_im: f32,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl QpskCostasLoopCc<DefaultCpuReader<Complex32>, DefaultCpuWriter<Complex32>> {
    pub fn new(loop_bw: f32, damping: f32, oqpsk: bool) -> Self {
        let damping = if damping <= 0.0 { 0.707 } else { damping };
        let loop_bw = if loop_bw <= 0.0 { 0.02 } else { loop_bw };

        let denom = 1.0 + 2.0 * damping * loop_bw + loop_bw * loop_bw;
        let alpha = (4.0 * damping * loop_bw) / denom;
        let beta = (4.0 * loop_bw * loop_bw) / denom;

        Self {
            phase: 0.0,
            freq: 0.0,
            alpha,
            beta,
            max_freq: 1.0,
            min_freq: -1.0,
            oqpsk,
            prev_im: 0.0,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for QpskCostasLoopCc<I, O>
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
        let in_slice = self.input.slice();
        let out_slice = self.output.slice();

        let m = cmp::min(in_slice.len(), out_slice.len());
        if m > 0 {
            let mut phase = self.phase;
            let mut freq = self.freq;
            let alpha = self.alpha;
            let beta = self.beta;
            let max_freq = self.max_freq;
            let min_freq = self.min_freq;

            for (src, dst) in in_slice[..m].iter().zip(out_slice[..m].iter_mut()) {
                let (sin_val, cos_val) = crate::math::fast_sincos(phase);

                // Complex rotation: (src.re + j*src.im) * (cos_val + j*sin_val)
                // rot_re = src.re * cos_val - src.im * (-sin_val) -> src.re.mul_add(cos_val, src.im.algebraic_mul(sin_val))
                let rot_re = src.re.mul_add(cos_val, src.im.algebraic_mul(sin_val));
                let rot_im = (-src.re).mul_add(sin_val, src.im.algebraic_mul(cos_val));

                *dst = Complex32::new(rot_re, rot_im);

                // Decision-directed 4-phase Costas error detector
                // e = sign(re) * im - sign(im) * re
                let sgn_re: f32 = if rot_re >= 0.0 { 1.0 } else { -1.0 };
                let im_comp = if self.oqpsk {
                    let prev = self.prev_im;
                    self.prev_im = rot_im;
                    prev
                } else {
                    rot_im
                };
                let sgn_im: f32 = if im_comp >= 0.0 { 1.0 } else { -1.0 };
                let err = sgn_re.mul_add(im_comp, (-sgn_im).algebraic_mul(rot_re));

                freq = beta.mul_add(err, freq).clamp(min_freq, max_freq);
                phase = phase + freq + alpha.algebraic_mul(err);
                while phase > PI {
                    phase = f32::algebraic_sub(phase, 2.0 * PI);
                }
                while phase < -PI {
                    phase = f32::algebraic_add(phase, 2.0 * PI);
                }
            }

            self.phase = phase;
            self.freq = freq;
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

#[cfg(test)]
mod tests {

    #[test]
    fn test_qpsk_symbols_mapping() {
        let inv_sqrt2 = 1.0 / std::f32::consts::SQRT_2;
        // 0x00 -> 00 00 00 00 -> all (+inv_sqrt2, +inv_sqrt2)
        // 0xFF -> 11 11 11 11 -> all (-inv_sqrt2, -inv_sqrt2)
        let byte = 0x00;
        let b0 = (byte >> 6) & 1;
        let b1 = (byte >> 7) & 1;
        assert_eq!(b0, 0);
        assert_eq!(b1, 0);
        assert!(inv_sqrt2 > 0.7 && inv_sqrt2 < 0.71);
    }
}
