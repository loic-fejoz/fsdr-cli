use std::collections::VecDeque;

use futuresdr::num_complex::Complex32;

use crate::blocks::modulators::constellation::Constellation;

use super::timing_error_detector::*;

pub struct TedMuellerAndMuller {}

impl TimingErrorDetectorAlgorithm for TedMuellerAndMuller {
    type Derivative = NoDerivativeType;
    type LookAhead = WithoutLookAhead;
}

impl TimingErrorDetectorAlgorithmNoDerivatives<WithoutLookAhead> for TedMuellerAndMuller {
    fn compute_error_cf(d_decision: &VecDeque<Complex32>, d_input: &VecDeque<Complex32>) -> f32 {
        return (d_decision[1].re * d_input[0].re - d_decision[0].re * d_input[1].re)
            + (d_decision[1].im * d_input[0].im - d_decision[0].im * d_input[1].im);
    }

    // fn compute_error_ff(d_decision: [Complex32], d_input: [Complex32]) {
    //     return (d_decision[1].real() * d_input[0].real()
    //         - d_decision[0].real() * d_input[1].real());
    // }
}

impl TimingErrorDetectorAlgorithmNoLookahead<NoDerivativeType> for TedMuellerAndMuller {}

impl TedMuellerAndMuller {
    pub fn build<C>(
        inputs_per_symbol: usize,
        error_computation_depth: usize,
        constellation: &C,
    ) -> Result<TimingErrorDetector<TedMuellerAndMuller, C, WithoutLookAhead, NoDerivativeType>, &'static str>
    where
        C: Constellation,
    {
        if constellation.dimensionality() != 1 {
            return Err("timing_error_detector: constellation dimensionality (ie complex numbers per symbol) must be 1.")
        }
        return Ok(TimingErrorDetector::new(inputs_per_symbol, error_computation_depth, Some(constellation)));
    }
}
