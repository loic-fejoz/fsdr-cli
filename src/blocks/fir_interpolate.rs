use anyhow::Result;
use futuresdr::futuredsp::{firdes, windows};
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;

/// Polyphase / Upsampling FIR interpolator for Complex32 streams
#[derive(Block)]
pub struct FirInterpolateCc<
    I: CpuBufferReader<Item = Complex32> = DefaultCpuReader<Complex32>,
    O: CpuBufferWriter<Item = Complex32> = DefaultCpuWriter<Complex32>,
> {
    interp: usize,
    taps: Vec<f32>,
    history: Vec<Complex32>,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl FirInterpolateCc<DefaultCpuReader<Complex32>, DefaultCpuWriter<Complex32>> {
    pub fn new(interp: usize, transition_bw: f32, window: &str) -> Self {
        let interp = interp.max(1);
        let taps_length = (4.0 / transition_bw.max(0.001)) as usize;
        let taps_length = taps_length + if taps_length.is_multiple_of(2) { 1 } else { 0 };

        let rect_win = match window.to_uppercase().as_str() {
            "BLACKMAN" => windows::blackman(taps_length, false),
            "HANN" => windows::hann(taps_length, false),
            _ => windows::hamming(taps_length, false),
        };

        let cutoff = 0.5 / (interp as f32);
        let mut taps = firdes::lowpass::<f32>(cutoff as f64, rect_win.as_slice());
        // Scale taps by interp to maintain gain
        for t in &mut taps {
            *t *= interp as f32;
        }

        let history = vec![Complex32::new(0.0, 0.0); taps.len().saturating_sub(1)];

        Self {
            interp,
            taps,
            history,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }

    pub fn with_taps(interp: usize, taps: Vec<f32>) -> Self {
        let history = vec![Complex32::new(0.0, 0.0); taps.len().saturating_sub(1)];
        Self {
            interp: interp.max(1),
            taps,
            history,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for FirInterpolateCc<I, O>
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

        let max_in = out_slice.len() / self.interp;
        let m = cmp::min(in_slice.len(), max_in);

        if m > 0 {
            let n_taps = self.taps.len();
            let mut out_idx = 0;

            // Pre-combined contiguous slice [history, in_slice[..m]] to vectorize FIR lookup
            let mut buf = Vec::with_capacity(self.history.len() + m);
            buf.extend_from_slice(&self.history);
            buf.extend_from_slice(&in_slice[..m]);
            let hist_len = self.history.len();

            let interp = self.interp;

            for i in 0..m {
                let base_idx = hist_len + i;

                for phase in 0..interp {
                    let mut acc_re = 0.0f32;
                    let mut acc_im = 0.0f32;

                    // Stride by interp through taps for this polyphase branch
                    let mut tap_idx = (interp - (phase % interp)) % interp;
                    while tap_idx < n_taps {
                        let sample_offset = (tap_idx + phase) / interp;
                        let sample = buf[base_idx - sample_offset];
                        let tap = self.taps[tap_idx];

                        acc_re = sample.re.mul_add(tap, acc_re);
                        acc_im = sample.im.mul_add(tap, acc_im);

                        tap_idx += interp;
                    }

                    out_slice[out_idx] = Complex32::new(acc_re, acc_im);
                    out_idx += 1;
                }
            }

            if n_taps > 1 {
                let hist_len = self.history.len();
                if m >= hist_len {
                    self.history.copy_from_slice(&in_slice[m - hist_len..m]);
                } else {
                    self.history.drain(..m);
                    self.history.extend_from_slice(&in_slice[..m]);
                }
            }

            self.input.consume(m);
            self.output.produce(out_idx);
        }

        if !self.input.slice().is_empty() && self.output.slice().len() >= self.interp {
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
    fn test_fir_interpolate_init() {
        let interp = FirInterpolateCc::new(4, 0.05, "HAMMING");
        assert_eq!(interp.interp, 4);
        assert!(!interp.taps.is_empty());
    }
}
