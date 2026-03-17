use futuresdr::num_complex::Complex32;
use std::{collections::VecDeque, marker::PhantomData};

use crate::blocks::modulators::constellation::Constellation;

pub enum ErrorPolicy {
    Preserve,
    Reset,
}

pub struct NeedsDerivativeType;
pub struct NoDerivativeType;

pub trait DerivativeType {}
impl DerivativeType for NeedsDerivativeType {}
impl DerivativeType for NoDerivativeType {}

pub struct WithLookAhead;
pub struct WithoutLookAhead;

pub trait LookAheadType {}
impl LookAheadType for WithLookAhead {}
impl LookAheadType for WithoutLookAhead {}

pub trait TimingErrorDetectorAlgorithm {
    type Derivative: DerivativeType;
    type LookAhead: LookAheadType;
}

pub trait TimingErrorDetectorAlgorithmNeedsLookahead<D>:
    TimingErrorDetectorAlgorithm<LookAhead = WithLookAhead, Derivative = D>
where
    D: DerivativeType,
{
}

pub trait TimingErrorDetectorAlgorithmNoLookahead<D>:
    TimingErrorDetectorAlgorithm<LookAhead = WithoutLookAhead, Derivative = D>
where
    D: DerivativeType,
{
}

pub trait TimingErrorDetectorAlgorithmNeedsDerivatives<L>:
    TimingErrorDetectorAlgorithm<Derivative = NeedsDerivativeType, LookAhead = L>
where
    L: LookAheadType,
{
    fn compute_error_cf(
        d_decision: &VecDeque<Complex32>,
        d_input: &VecDeque<Complex32>,
        d_input_derivative: &VecDeque<Complex32>,
    ) -> f32;
    // fn compute_error_ff(
    //     d_decision: [Complex32],
    //     d_input: [Complex32],
    //     d_input_derivative: [Complex32],
    // ) -> f32;
}

pub trait TimingErrorDetectorAlgorithmNoDerivatives<L>:
    TimingErrorDetectorAlgorithm<Derivative = NoDerivativeType, LookAhead = L>
where
    L: LookAheadType,
{
    fn compute_error_cf(d_decision: &VecDeque<Complex32>, d_input: &VecDeque<Complex32>) -> f32;
    // fn compute_error_ff(d_decision: [Complex32], d_input: [Complex32]) -> f32;
}

pub struct TimingErrorDetector<'a, A, C, L, D>
where
    A: TimingErrorDetectorAlgorithm<LookAhead = L, Derivative = D>,
    C: Constellation,
    D: DerivativeType,
    L: LookAheadType,
{
    d_constellation: Option<&'a C>,
    d_error: f32,
    d_prev_error: f32,
    d_inputs_per_symbol: usize,
    d_input_clock: usize,
    d_error_depth: usize,
    d_input: VecDeque<Complex32>,
    d_decision: VecDeque<Complex32>,
    d_input_derivative: VecDeque<Complex32>,
    _phantom_l: PhantomData<L>,
    _phantom_d: PhantomData<D>,
    _phantom_a: PhantomData<A>,
    // _phantom_c: PhantomData<C>,
}



impl<'a, A, C, L, D> TimingErrorDetector<'a, A, C, L, D>
where
    D: DerivativeType,
    L: LookAheadType,
    A: TimingErrorDetectorAlgorithm<LookAhead = L, Derivative = D>,
    C: Constellation,
{
    pub fn new(
        inputs_per_symbol: usize,
        error_computation_depth: usize,
        constellation: Option<&'a C>,
    ) -> TimingErrorDetector<'a, A, C, L, D> {
        let mut ted = TimingErrorDetector {
            d_constellation: constellation,
            d_error: 0.0,
            d_prev_error: 0.0,
            d_inputs_per_symbol: inputs_per_symbol,
            d_input_clock: 0,
            d_error_depth: error_computation_depth,
            d_input: VecDeque::new(),
            d_decision: VecDeque::new(),
            d_input_derivative: VecDeque::new(),
            _phantom_l: PhantomData,
            _phantom_d: PhantomData,
            _phantom_a: PhantomData,
        };
        ted.sync_reset();
        ted
    }

    pub fn inputs_per_symbol(&self) -> usize {
        self.d_inputs_per_symbol
    }
    pub fn error(&self) -> f32 {
        self.d_error
    }

    pub fn advance_input_clock(&mut self) {
        self.d_input_clock = (self.d_input_clock + 1) % self.d_inputs_per_symbol;
    }

    pub fn revert_input_clock(&mut self) {
        if self.d_input_clock == 0 {
            self.d_input_clock = self.d_inputs_per_symbol - 1;
        } else {
            self.d_input_clock -= 1;
        }
    }

    pub fn sync_reset_input_clock(&mut self) {
        self.d_input_clock = self.d_inputs_per_symbol - 1;
    }

    pub fn sync_reset(&mut self) {
        self.d_error = 0.0;
        self.d_prev_error = 0.0;

        self.d_input = std::iter::repeat_n(Complex32::default(), self.d_error_depth).collect();

        self.d_input_derivative = std::iter::repeat_n(Complex32::default(), self.d_error_depth).collect();

        if self.d_constellation.is_some() {
            self.d_decision = std::iter::repeat_n(Complex32::default(), self.d_input.len()).collect();
        }

        self.sync_reset_input_clock();
    }

    pub fn slice(&self, x: Complex32) -> Complex32 {
        /* Passing a length 1 array is OK since we only accept 1D constellations */
        let mut z = [Complex32 { re: 0.0, im: 0.0 }];
        if let Some(constellation) = self.d_constellation.as_ref() {
            let index = constellation.decision_maker(x);
            constellation.map_to_points(index, &mut z);
        }
        z[0]
    }
}

#[inline]
pub fn dup_back_same_size(v: &mut VecDeque<Complex32>) {
    let elt = v.back();
    match elt {
        Some(elt) => {
            v.push_back(*elt);
            v.pop_front();
        }
        _ => {
            todo!("to be understood")
        }
    }
}

impl<'a, A, C, L> TimingErrorDetector<'a, A, C, L, NeedsDerivativeType>
where
    A: TimingErrorDetectorAlgorithmNeedsDerivatives<L, Derivative = NeedsDerivativeType>,
    C: Constellation,
    L: LookAheadType,
{
    pub fn compute_error_cf(&mut self) -> f32 {
        A::compute_error_cf(&self.d_decision, &self.d_input, &self.d_input_derivative)
    }

    // fn compute_error_ff(&mut self) -> f32 {
    //     A::compute_error_ff(self.d_decision, self.d_input, self.d_input_derivative)
    // }

    pub fn revert(&mut self, err_policy: ErrorPolicy) {
        if self.d_input_clock == 0 {
            if let ErrorPolicy::Preserve = err_policy {
                self.d_error = self.d_prev_error;
            }
        }
        self.revert_input_clock();

        dup_back_same_size(&mut self.d_input_derivative);

        if self.d_constellation.is_some() {
            dup_back_same_size(&mut self.d_decision);
        }

        dup_back_same_size(&mut self.d_input);
    }
}

impl<'a, A, C, L> TimingErrorDetector<'a, A, C, L, NoDerivativeType>
where
    A: TimingErrorDetectorAlgorithmNoDerivatives<L, Derivative = NoDerivativeType>,
    C: Constellation,
    L: LookAheadType,
{
    pub fn compute_error_cf(&mut self) -> f32 {
        A::compute_error_cf(&self.d_decision, &self.d_input)
    }

    // fn compute_error_ff(&mut self) -> f32 {
    //     A::compute_error_ff(self.d_decision, self.d_input)
    // }

    pub fn revert(&mut self, err_policy: ErrorPolicy) {
        if self.d_input_clock == 0 {
            if let ErrorPolicy::Preserve = err_policy {
                self.d_error = self.d_prev_error;
            }
        }
        self.revert_input_clock();

        if self.d_constellation.is_some() {
            dup_back_same_size(&mut self.d_decision)
        }

        dup_back_same_size(&mut self.d_input);
    }
}

impl<'a, A, C> TimingErrorDetector<'a, A, C, WithLookAhead, NeedsDerivativeType>
where
    A: TimingErrorDetectorAlgorithmNeedsLookahead<NeedsDerivativeType, LookAhead = WithLookAhead>
        + TimingErrorDetectorAlgorithmNeedsDerivatives<
            WithLookAhead,
            Derivative = NeedsDerivativeType,
        >,
    C: Constellation,
{
    pub fn input_lookahead(&mut self, x: Complex32, d_x: Complex32) {
        if self.d_input_clock != 0 {
            return;
        }
        self.d_input.push_front(x);
        if self.d_constellation.is_some() {
            self.d_decision.push_front(self.slice(self.d_input[0]));
        }
        self.d_input_derivative.push_front(d_x);

        self.d_prev_error = self.d_error;
        self.d_error = self.compute_error_cf();

        self.d_input_derivative.pop_front();
        if self.d_constellation.is_some() {
            self.d_decision.pop_front();
        }
        self.d_input.pop_front();
    }

    pub fn input(&mut self, x: Complex32, d_x: Complex32) {
        self.d_input.push_front(x);
        self.d_input.pop_back();

        if self.d_constellation.is_some() {
            self.d_decision.push_front(self.slice(self.d_input[0]));
            self.d_decision.pop_back();
        }

        self.d_input_derivative.push_front(d_x);
        self.d_input_derivative.pop_back();

        self.advance_input_clock();
    }
}

impl<'a, A, C> TimingErrorDetector<'a, A, C, WithLookAhead, NoDerivativeType>
where
    A: TimingErrorDetectorAlgorithmNeedsLookahead<NoDerivativeType, LookAhead = WithLookAhead>
        + TimingErrorDetectorAlgorithmNoDerivatives<WithLookAhead, Derivative = NoDerivativeType>,
    C: Constellation,
{
    pub fn input_lookahead(&mut self, x: Complex32) {
        if self.d_input_clock != 0 {
            return;
        }
        self.d_input.push_front(x);
        if self.d_constellation.is_some() {
            self.d_decision.push_front(self.slice(self.d_input[0]));
        }

        self.d_prev_error = self.d_error;
        self.d_error = self.compute_error_cf();

        if self.d_constellation.is_some() {
            self.d_decision.pop_front();
        }
        self.d_input.pop_front();
    }

    pub fn input(&mut self, x: Complex32) {
        self.d_input.push_front(x);
        self.d_input.pop_back();

        if self.d_constellation.is_some() {
            self.d_decision.push_front(self.slice(self.d_input[0]));
            self.d_decision.pop_back();
        }

        self.advance_input_clock();
    }
}

impl<'a, A, C, D> TimingErrorDetector<'a, A, C, WithoutLookAhead, D>
where
    D: DerivativeType,
    A: TimingErrorDetectorAlgorithmNoLookahead<D, LookAhead = WithoutLookAhead>,
    C: Constellation,
{
    pub fn input_lookahead(&mut self, _x: Complex32, _d_x: Option<Complex32>) {
        // Nothing to do
    }
}

impl<'a, A, C> TimingErrorDetector<'a, A, C, WithoutLookAhead, NeedsDerivativeType>
where
    A: TimingErrorDetectorAlgorithmNoLookahead<NeedsDerivativeType, LookAhead = WithoutLookAhead>
        + TimingErrorDetectorAlgorithmNeedsDerivatives<
            WithoutLookAhead,
            Derivative = NeedsDerivativeType,
        >,
    C: Constellation,
{
    pub fn input(&mut self, x: Complex32, d_x: Complex32) {
        self.d_input.push_front(x);
        self.d_input.pop_back();

        if self.d_constellation.is_some() {
            self.d_decision.push_front(self.slice(self.d_input[0]));
            self.d_decision.pop_back();
        }

        self.d_input_derivative.push_front(d_x);
        self.d_input_derivative.pop_back();

        self.advance_input_clock();
        if self.d_input_clock == 0 {
            self.d_prev_error = self.d_error;
            self.d_error = self.compute_error_cf();
        }
    }
}

impl<'a, A, C> TimingErrorDetector<'a, A, C, WithoutLookAhead, NoDerivativeType>
where
    A: TimingErrorDetectorAlgorithmNoLookahead<NoDerivativeType, LookAhead = WithoutLookAhead>
        + TimingErrorDetectorAlgorithmNoDerivatives<WithoutLookAhead, Derivative = NoDerivativeType>,
    C: Constellation,
{
    pub fn input(&mut self, x: Complex32) {
        self.d_input.push_front(x);
        self.d_input.pop_back();

        if self.d_constellation.is_some() {
            self.d_decision.push_front(self.slice(self.d_input[0]));
            self.d_decision.pop_back();
        }
        self.advance_input_clock();
        if self.d_input_clock == 0 {
            self.d_prev_error = self.d_error;
            self.d_error = self.compute_error_cf();
        }
    }
}
