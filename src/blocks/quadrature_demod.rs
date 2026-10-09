use crate::math::{fast_atan_demod_slice, fast_quadri_demod_slice};
use anyhow::Result;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;

/// Demodulation algorithm variant for `QuadratureDemodCf`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuadratureDemodAlgo {
    /// Quadrature FM demodulation: arg(x[n] * conj(x[n-1]))
    Quadri,
    /// Instantaneous phase differentiation: arg(x[n]) - arg(x[n-1]) wrapped to [-PI, PI]
    Atan,
}

/// High-performance SIMD/FMA-accelerated Quadrature FM demodulator block.
///
/// Converts FM-modulated `Complex32` IQ baseband into demodulated `f32` audio/subcarrier signals.
#[derive(Block)]
pub struct QuadratureDemodCf<
    I: CpuBufferReader<Item = Complex32> = DefaultCpuReader<Complex32>,
    O: CpuBufferWriter<Item = f32> = DefaultCpuWriter<f32>,
> {
    gain: f32,
    algo: QuadratureDemodAlgo,
    last_sample: Complex32,
    last_phase: f32,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl QuadratureDemodCf<DefaultCpuReader<Complex32>, DefaultCpuWriter<f32>> {
    pub fn new(gain: f32) -> Self {
        Self::with_algo(gain, QuadratureDemodAlgo::Quadri)
    }

    pub fn with_algo(gain: f32, algo: QuadratureDemodAlgo) -> Self {
        Self {
            gain,
            algo,
            last_sample: Complex32::new(0.0, 0.0),
            last_phase: 0.0,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for QuadratureDemodCf<DefaultCpuReader<Complex32>, DefaultCpuWriter<f32>> {
    fn default() -> Self {
        Self::new(1.0)
    }
}

#[doc(hidden)]
impl<I, O> Kernel for QuadratureDemodCf<I, O>
where
    I: CpuBufferReader<Item = Complex32>,
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
            match self.algo {
                QuadratureDemodAlgo::Quadri => {
                    fast_quadri_demod_slice(&i[..m], &mut o[..m], &mut self.last_sample, self.gain);
                }
                QuadratureDemodAlgo::Atan => {
                    fast_atan_demod_slice(&i[..m], &mut o[..m], &mut self.last_phase, self.gain);
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
