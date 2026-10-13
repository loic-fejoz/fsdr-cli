use anyhow::{bail, Result};
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;
use std::f32::consts::PI;

pub fn firdes_rrc_taps(alpha: f32, sps: usize, span: usize) -> Vec<f32> {
    let ntaps = span * sps + 1;
    let mut taps = Vec::with_capacity(ntaps);
    let center = (ntaps as f32 - 1.0) / 2.0;
    let sps_f = sps as f32;

    for i in 0..ntaps {
        let t = (i as f32 - center) / sps_f;
        let val = if t.abs() < 1e-7 {
            1.0 - alpha + 4.0 * alpha / PI
        } else if alpha > 1e-6 && (t.abs() - 1.0 / (4.0 * alpha)).abs() < 1e-5 {
            (alpha / std::f32::consts::SQRT_2)
                * ((1.0 + 2.0 / PI) * (PI / (4.0 * alpha)).sin()
                    + (1.0 - 2.0 / PI) * (PI / (4.0 * alpha)).cos())
        } else {
            let num =
                (PI * t * (1.0 - alpha)).sin() + 4.0 * alpha * t * (PI * t * (1.0 + alpha)).cos();
            let denom = PI * t * (1.0 - (4.0 * alpha * t).powi(2));
            num / denom
        };
        taps.push(val);
    }

    // Energy normalize
    let energy: f32 = taps.iter().map(|&x| x * x).sum();
    if energy > 0.0 {
        let scale = 1.0 / energy.sqrt();
        for tap in &mut taps {
            *tap *= scale;
        }
    }
    taps
}

pub fn firdes_gaussian_taps(bt: f32, sps: usize, span: usize) -> Vec<f32> {
    let ntaps = span * sps + 1;
    let mut taps = Vec::with_capacity(ntaps);
    let center = (ntaps as f32 - 1.0) / 2.0;
    let alpha = (2.0 * PI / (2.0_f32.ln()).sqrt()) * bt;

    for i in 0..ntaps {
        let t = (i as f32 - center) / (sps as f32);
        let val = (-PI * (alpha * t).powi(2)).exp();
        taps.push(val);
    }

    let sum: f32 = taps.iter().sum();
    if sum > 0.0 {
        for tap in &mut taps {
            *tap /= sum;
        }
    }
    taps
}

/// Pulse shaping FIR filter for Complex32 streams
#[derive(Block)]
pub struct PulseShapingFilterCc<
    I: CpuBufferReader<Item = Complex32> = DefaultCpuReader<Complex32>,
    O: CpuBufferWriter<Item = Complex32> = DefaultCpuWriter<Complex32>,
> {
    taps: Vec<f32>,
    history: Vec<Complex32>,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl PulseShapingFilterCc<DefaultCpuReader<Complex32>, DefaultCpuWriter<Complex32>> {
    pub fn new(filter_type: &str, alpha: f32, sps: usize, span: usize) -> Result<Self> {
        let taps = match filter_type.to_uppercase().as_str() {
            "RRC" | "ROOT_RAISED_COSINE" => firdes_rrc_taps(alpha, sps, span),
            "GAUSSIAN" | "GAUSS" => firdes_gaussian_taps(alpha, sps, span),
            _ => bail!("Unsupported pulse shaping filter type: {}", filter_type),
        };
        let history = vec![Complex32::new(0.0, 0.0); taps.len().saturating_sub(1)];
        Ok(Self {
            taps,
            history,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        })
    }

    pub fn with_taps(taps: Vec<f32>) -> Self {
        let history = vec![Complex32::new(0.0, 0.0); taps.len().saturating_sub(1)];
        Self {
            taps,
            history,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for PulseShapingFilterCc<I, O>
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
            let n_taps = self.taps.len();
            // Build contiguous combined buffer [history, in_slice[..m]] once to eliminate branch inside convolution
            let mut buf = Vec::with_capacity(self.history.len() + m);
            buf.extend_from_slice(&self.history);
            buf.extend_from_slice(&in_slice[..m]);

            let taps = &self.taps;

            for i in 0..m {
                let mut acc_re = 0.0f32;
                let mut acc_im = 0.0f32;
                let window = &buf[i..i + n_taps];

                // Reversible dot-product with FMA
                for (tap, sample) in taps.iter().zip(window.iter().rev()) {
                    acc_re = sample.re.mul_add(*tap, acc_re);
                    acc_im = sample.im.mul_add(*tap, acc_im);
                }
                out_slice[i] = Complex32::new(acc_re, acc_im);
            }

            // Update history buffer
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
    use super::*;

    #[test]
    fn test_rrc_taps_symmetry_and_energy() {
        let taps = firdes_rrc_taps(0.35, 4, 8);
        assert_eq!(taps.len(), 8 * 4 + 1);

        for i in 0..taps.len() / 2 {
            let diff = (taps[i] - taps[taps.len() - 1 - i]).abs();
            assert!(diff < 1e-6);
        }

        let energy: f32 = taps.iter().map(|&x| x * x).sum();
        assert!((energy - 1.0).abs() < 1e-5);
    }
}
