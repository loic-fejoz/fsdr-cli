pub struct ClockTrackingLoop {
    // Estimate of the average clock period, T_avg, in units of
    // input sample clocks (so this is the average number of
    // input samples per output symbol, aka samples/symbol).
    // To convert to seconds, divide by the input sample rate: F_s_input.
    d_avg_period: f32,

    // Limits on how far the average clock period estimate can wander,
    // and the nominal average clock period, in units of input sample clocks.
    // To convert to seconds, divide by the input sample rate: F_s_input.
    d_max_avg_period: f32,
    d_min_avg_period: f32,
    d_nom_avg_period: f32,

    // Instantaneous clock period estimate, T_inst, in units of
    // input sample clocks (so this is the intantaneous number of
    // input samples per output symbol, aka instantaneous samples/symbol).
    // To convert to seconds, divide by the input sample rate: F_s_input.
    d_inst_period: f32,

    // Instantaneous clock phase estimate, tau, in units of
    // input sample clocks.
    // To convert to seconds, divide by the input sample rate: F_s_input.
    // To wrap, add or subtract a multiple of the estimate of the
    // average clock period, T_avg.
    // To convert to a normalized (but not wrapped) clock phase estimate,
    // divide by the estimate of the average clock period, T_avg.
    // To further convert the normalized clock phase estimate to radians,
    // multiply the normalized clock phase estimate by 2*pi.
    d_phase: f32,

    // Damping factor of the 2nd order loop transfer function.
    // Zeta in the range (0.0, 1.0) yields an under-damped loop.
    // Zeta in the range (1.0, Inf) yields an over-damped loop.
    // Zeta equal to 1.0 yields a crtically-damped loop.
    d_zeta: f32,

    // Normalized natural radian frequency of the 2nd order loop transfer
    // function.  It should be a small positive number, corresponding to
    // the normalized natural radian frequency of the loop as digital
    // low-pass filter that is filtering the clock phase/timing error signal.
    // omega_n_norm = omega_n*T  = 2*pi*f_n*T = 2*pi*f_n_norm
    d_omega_n_norm: f32,

    // Expected gain of the timing error detector in use, given the
    // TED estimator expression, the expected input amplitude, the
    // input pulse shape, and the expected input Es/No.  (This value is the
    // slope of the TED's S-curve plot at a timing offset of tau = 0, and
    // must be determined by analysis and/or simulation by the user.)
    d_ted_gain: f32,

    // Proportional gain of the PI loop filter (aka gain_mu)
    // (aka gain_mu in some clock recovery blocks)
    d_alpha: f32,

    // Integral gain of the PI loop filter
    // (aka gain_omega in some clock recovery blocks)
    d_beta: f32,

    // For reverting the loop state one iteration (only)
    d_prev_avg_period: f32,
    d_prev_inst_period: f32,
    d_prev_phase: f32,
}

impl ClockTrackingLoop {
    fn new(
        d_avg_period: f32,
        d_max_avg_period: f32,
        d_min_avg_period: f32,
        d_nom_avg_period: f32,
        d_inst_period: f32,
        d_phase: f32,
        d_zeta: f32,
        d_omega_n_norm: f32,
        d_ted_gain: f32,
        d_alpha: f32,
        d_beta: f32,
        d_prev_avg_period: f32,
        d_prev_inst_period: f32,
        d_prev_phase: f32,
    ) -> Self {
        Self {
            d_avg_period,
            d_max_avg_period,
            d_min_avg_period,
            d_nom_avg_period,
            d_inst_period,
            d_phase,
            d_zeta,
            d_omega_n_norm,
            d_ted_gain,
            d_alpha,
            d_beta,
            d_prev_avg_period,
            d_prev_inst_period,
            d_prev_phase,
        }
    }

    pub fn advance_loop(&mut self, error: f32) {
        // So the loop can be reverted one step, if needed.
        self.d_prev_avg_period = self.d_avg_period;
        self.d_prev_inst_period = self.d_inst_period;
        self.d_prev_phase = self.d_phase;

        // Integral arm of PI filter
        self.d_avg_period = self.d_avg_period + self.d_beta * error;
        // Limit the integral arm output here, as a large negative
        // error input can lead to a negative d_avg_period, which
        // will cause an infitine loop in the phase wrap method.
        self.period_limit();

        // Proportional arm of PI filter and final sum of PI filter arms
        self.d_inst_period = self.d_avg_period + self.d_alpha * error;
        // Limit the filter output here, for the errant case of a large
        // negative error input, that can lead to a negative d_inst_period,
        // which results in an incorrect phase increment, as it is assumed
        // to be moving forward to the next symbol.
        if self.d_inst_period <= 0.0 {
            self.d_inst_period = self.d_avg_period;
        }
        // Compute the new, unwrapped clock phase
        self.d_phase = self.d_phase + self.d_inst_period;
    }

    pub fn revert_loop(&mut self) {
        self.d_avg_period = self.d_prev_avg_period;
        self.d_inst_period = self.d_prev_inst_period;
        self.d_phase = self.d_prev_phase;
    }

    fn phase_wrap(&mut self) {
        let period = self.d_avg_period; // One could argue d_inst_period instead
        let limit = period / 2.0;

        while self.d_phase > limit {
            self.d_phase -= period;
        }

        while self.d_phase <= -limit {
            self.d_phase += period;
        }
    }

    fn period_limit(&mut self) {
        if self.d_avg_period > self.d_max_avg_period {
            self.d_avg_period = self.d_max_avg_period;
        } else if self.d_avg_period < self.d_min_avg_period {
            self.d_avg_period = self.d_min_avg_period;
        }
    }

    fn update_gains(&mut self) {
        let omega_n_T = self.d_omega_n_norm;
        let zeta_omega_n_T = self.d_zeta * omega_n_T;
        let k0 = 2.0 / self.d_ted_gain;
        let k1 = (-zeta_omega_n_T).exp();
        let sinh_zeta_omega_n_T = zeta_omega_n_T.sinh();

        let cosx_omega_d_T = if self.d_zeta > 1.0 {
            // Over-damped (or critically-damped too)

            let omega_d_T = omega_n_T * (self.d_zeta * self.d_zeta - 1.0).sqrt();
            omega_d_T.cosh()
        } else if self.d_zeta == 1.0 {
            // Critically-damped
            1.0
            // cosh(omega_d_T) & cos(omega_d_T) are both 1 for omega_d_T == 0
        } else {
            // Under-damped (or critically-damped too)

            let omega_d_T = omega_n_T * (1.0 - self.d_zeta * self.d_zeta).sqrt();
            omega_d_T.cos()
        };

        let alpha = k0 * k1 * sinh_zeta_omega_n_T;
        let beta = k0 * (1.0 - k1 * (sinh_zeta_omega_n_T + cosx_omega_d_T));

        self.d_alpha = alpha;
        self.d_beta = beta;
    }

    fn set_loop_bandwidth(&mut self, bw: f32) {
        assert!(
            bw >= 0.0,
            "clock_tracking_loop: loop bandwidth must be greater than 0.0"
        );
        self.d_omega_n_norm = bw;
        self.update_gains();
    }

    fn set_damping_factor(&mut self, df: f32) {
        assert!(
            df >= 0.0,
            "clock_tracking_loop: damping factor must be > 0.0"
        );
        self.d_zeta = df;
        self.update_gains();
    }

    fn set_ted_gain(&mut self, ted_gain: f32) {
        assert!(
            ted_gain > 0.0,
            "clock_tracking_loop: expected ted gain must be > 0.0"
        );
        self.d_ted_gain = ted_gain;
        self.update_gains();
    }

    pub fn set_avg_period(&mut self, period: f32) {
        self.d_avg_period = period;
        self.d_prev_avg_period = period;
    }

    pub fn set_inst_period(&mut self, period: f32) {
        self.d_inst_period = period;
        self.d_prev_inst_period = period;
    }

    fn set_nom_avg_period(&mut self, period: f32) {
        if period < self.d_min_avg_period || period > self.d_max_avg_period {
            self.d_nom_avg_period = (self.d_max_avg_period + self.d_min_avg_period) / 2.0;
        } else {
            self.d_nom_avg_period = period;
        }
    }

    fn loop_bandwidth(&self) -> f32 {
        return self.d_omega_n_norm;
    }

    fn damping_factor(&self) -> f32 {
        return self.d_zeta;
    }

    pub fn set_phase(&mut self, phase: f32) {
        // This previous phase is likely inconsistent with the tracking,
        // but if the caller is setting the phase, the odds of
        // revert_loop() being called are slim.
        self.d_prev_phase = phase;

        self.d_phase = phase;
    }
}
