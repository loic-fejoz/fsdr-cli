//! Fast SIMD & FMA-optimized mathematical approximations for SDR signal processing.
//!
//! Provides polynomial rational/minimax approximations for transcendental functions:
//! - `fast_atan2`: High-precision (error < 2e-5 rad / < 0.001 degrees) 4-quadrant arctangent using Horner FMA.
//! - `fast_sincos`: High-precision (error < 5e-7) sine/cosine generator using Horner FMA.
//! - Chunked slice processing for batch vectorization.

use futuresdr::num_complex::Complex32;
use std::f32::consts::{FRAC_PI_2, PI};

/// Minimax polynomial coefficients for arctan(z) on [0, 1].
const ATAN_C1: f32 = 0.9999993;
const ATAN_C3: f32 = -0.33329856;
const ATAN_C5: f32 = 0.19946536;
const ATAN_C7: f32 = -0.13908534;
const ATAN_C9: f32 = 0.09642004;
const ATAN_C11: f32 = -0.055909884;
const ATAN_C13: f32 = 0.021861228;
const ATAN_C15: f32 = -0.004054058;

/// Fast 4-quadrant arctangent (atan2) using a Horner polynomial with FMA.
///
/// Max absolute error < 2e-5 radians (< 0.0011 degrees) across the entire domain.
#[inline(always)]
pub fn fast_atan2(y: f32, x: f32) -> f32 {
    let abs_y = y.abs();
    let abs_x = x.abs();

    if abs_x < 1e-20 && abs_y < 1e-20 {
        return 0.0;
    }

    let (n, d, octant_swap) = if abs_y <= abs_x {
        (abs_y, abs_x, false)
    } else {
        (abs_x, abs_y, true)
    };

    let z = n / d;
    let z2 = z * z;

    // Horner's method with FMA
    let poly = z2.mul_add(ATAN_C15, ATAN_C13);
    let poly = z2.mul_add(poly, ATAN_C11);
    let poly = z2.mul_add(poly, ATAN_C9);
    let poly = z2.mul_add(poly, ATAN_C7);
    let poly = z2.mul_add(poly, ATAN_C5);
    let poly = z2.mul_add(poly, ATAN_C3);
    let poly = z2.mul_add(poly, ATAN_C1);
    let mut angle = z * poly;

    if octant_swap {
        angle = FRAC_PI_2 - angle;
    }

    if x < 0.0 {
        angle = PI - angle;
    }

    if y < 0.0 {
        angle = -angle;
    }

    angle
}

#[allow(clippy::excessive_precision)]
/// Minimax coefficients for sin(x) on [-pi/2, pi/2].
const SIN_S1: f32 = -0.16666659;
const SIN_S2: f32 = 0.008333075;
const SIN_S3: f32 = -0.000198107;
const SIN_S4: f32 = 0.0000026083;

#[allow(clippy::excessive_precision)]
/// Minimax coefficients for cos(x) on [-pi/2, pi/2].
const COS_C1: f32 = -0.49999997;
const COS_C2: f32 = 0.04166664;
const COS_C3: f32 = -0.0013888397;
const COS_C4: f32 = 0.0000247609;
const COS_C5: f32 = -0.0000002605;

/// Fast simultaneous sine and cosine (sin_cos) evaluation using Horner FMA.
///
/// Max absolute error < 5e-7 across [-pi, pi].
#[inline(always)]
pub fn fast_sincos(phase: f32) -> (f32, f32) {
    // Fast periodic range reduction to [-PI, PI] using FMA:
    const INV_TWO_PI: f32 = 1.0 / (2.0 * PI);
    const TWO_PI: f32 = 2.0 * PI;
    let k = (phase * INV_TWO_PI).round();
    let x = k.mul_add(-TWO_PI, phase);

    // Range reduction to [-PI/2, PI/2]
    let (reduced_x, cos_sign) = if x > FRAC_PI_2 {
        (PI - x, -1.0f32)
    } else if x < -FRAC_PI_2 {
        (-PI - x, -1.0f32)
    } else {
        (x, 1.0f32)
    };

    let x2 = reduced_x * reduced_x;

    // Sine via Horner's form: x + x^3 * (S1 + x^2 * (S2 + x^2 * (S3 + x^2 * S4)))
    let sin_poly = x2.mul_add(SIN_S4, SIN_S3);
    let sin_poly = x2.mul_add(sin_poly, SIN_S2);
    let sin_poly = x2.mul_add(sin_poly, SIN_S1);
    let sin_val = (reduced_x * x2).mul_add(sin_poly, reduced_x);

    // Cosine via Horner's form: (1 + x^2 * (C1 + x^2 * (C2 + x^2 * (C3 + x^2 * (C4 + x^2 * C5))))) * cos_sign
    let cos_poly = x2.mul_add(COS_C5, COS_C4);
    let cos_poly = x2.mul_add(cos_poly, COS_C3);
    let cos_poly = x2.mul_add(cos_poly, COS_C2);
    let cos_poly = x2.mul_add(cos_poly, COS_C1);
    let cos_val = x2.mul_add(cos_poly, 1.0) * cos_sign;

    (sin_val, cos_val)
}

/// Fast sine approximation using Horner FMA.
#[inline(always)]
pub fn fast_sin(phase: f32) -> f32 {
    fast_sincos(phase).0
}

/// Fast cosine approximation using Horner FMA.
#[inline(always)]
pub fn fast_cos(phase: f32) -> f32 {
    fast_sincos(phase).1
}

/// Batch compute 4-quadrant arctangent over a chunked complex slice.
#[inline]
pub fn fast_atan2_complex_slice(input: &[Complex32], output: &mut [f32]) {
    let m = std::cmp::min(input.len(), output.len());
    for (src, dst) in input[..m].iter().zip(output[..m].iter_mut()) {
        *dst = fast_atan2(src.im, src.re);
    }
}

/// Batch compute quadrature demodulation over complex samples.
///
/// Implements: `arg(x[n] * conj(x[n-1])) * gain` using FMA conjugate multiplication and `fast_atan2`.
#[inline]
pub fn fast_quadri_demod_slice(
    input: &[Complex32],
    output: &mut [f32],
    last_sample: &mut Complex32,
    gain: f32,
) {
    let m = std::cmp::min(input.len(), output.len());
    let mut last = *last_sample;

    for (src, dst) in input[..m].iter().zip(output[..m].iter_mut()) {
        let cur = *src;
        // (cur.re + j*cur.im) * (last.re - j*last.im)
        // Re = cur.re * last.re + cur.im * last.im
        // Im = cur.im * last.re - cur.re * last.im
        let diff_re = cur.re.mul_add(last.re, cur.im.algebraic_mul(last.im));
        let diff_im = cur.im.mul_add(last.re, (-cur.re).algebraic_mul(last.im));

        let arg = fast_atan2(diff_im, diff_re);
        *dst = f32::algebraic_mul(arg, gain);
        last = cur;
    }

    *last_sample = last;
}

/// Batch compute instantaneous phase differentiation FM demodulation.
///
/// Implements: `(arg(x[n]) - arg(x[n-1])) * gain` with `[-PI, PI]` wrapping.
#[inline]
pub fn fast_atan_demod_slice(
    input: &[Complex32],
    output: &mut [f32],
    last_phase: &mut f32,
    gain: f32,
) {
    let m = std::cmp::min(input.len(), output.len());
    let mut prev_phase = *last_phase;

    for (src, dst) in input[..m].iter().zip(output[..m].iter_mut()) {
        let cur_phase = fast_atan2(src.im, src.re);
        let mut diff = f32::algebraic_sub(cur_phase, prev_phase);

        while diff > PI {
            diff -= 2.0 * PI;
        }
        while diff < -PI {
            diff += 2.0 * PI;
        }

        *dst = f32::algebraic_mul(diff, gain);
        prev_phase = cur_phase;
    }

    *last_phase = prev_phase;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_atan2_precision_against_textbook() {
        const MAX_ATAN_ERROR: f32 = 2.0e-5; // < 0.0012 degrees tolerance

        // Test cardinal directions & axes
        assert_eq!(fast_atan2(0.0, 0.0), 0.0);
        assert!((fast_atan2(0.0, 1.0) - 0.0f32.atan2(1.0)).abs() < MAX_ATAN_ERROR);
        assert!((fast_atan2(1.0, 0.0) - 1.0f32.atan2(0.0)).abs() < MAX_ATAN_ERROR);
        assert!((fast_atan2(0.0, -1.0) - 0.0f32.atan2(-1.0)).abs() < MAX_ATAN_ERROR);
        assert!((fast_atan2(-1.0, 0.0) - (-1.0f32).atan2(0.0)).abs() < MAX_ATAN_ERROR);

        // Sweep angles in all 4 quadrants across 10,000 steps and various radii
        let steps = 10000;
        let radii = [0.001, 0.1, 1.0, 10.0, 1000.0];

        for &r in &radii {
            for i in 0..steps {
                let theta = -PI + (2.0 * PI * (i as f32) / (steps as f32));
                let x = r * theta.cos();
                let y = r * theta.sin();

                let expected = y.atan2(x);
                let actual = fast_atan2(y, x);

                let mut diff = (actual - expected).abs();
                // Handle 2*pi boundary wrap if near -pi / +pi
                if diff > PI {
                    diff = (diff - 2.0 * PI).abs();
                }

                assert!(
                    diff < MAX_ATAN_ERROR,
                    "Precision failure at theta={theta}, r={r}: expected={expected}, actual={actual}, diff={diff}"
                );
            }
        }
    }

    #[test]
    fn test_fast_sincos_precision_against_textbook() {
        const MAX_SINCOS_ERROR: f32 = 5e-7;

        let steps = 20000;
        for i in 0..steps {
            let phase = -2.0 * PI + (4.0 * PI * (i as f32) / (steps as f32));
            let (expected_sin, expected_cos) = phase.sin_cos();
            let (actual_sin, actual_cos) = fast_sincos(phase);

            let sin_diff = (actual_sin - expected_sin).abs();
            let cos_diff = (actual_cos - expected_cos).abs();

            assert!(
                sin_diff < MAX_SINCOS_ERROR,
                "Sin precision failure at phase={phase}: expected={expected_sin}, actual={actual_sin}, diff={sin_diff}"
            );
            assert!(
                cos_diff < MAX_SINCOS_ERROR,
                "Cos precision failure at phase={phase}: expected={expected_cos}, actual={actual_cos}, diff={cos_diff}"
            );
        }
    }

    #[test]
    fn test_quadri_demod_slice_against_textbook() {
        let sample_count = 1024;
        let gain = 1.5;
        let mut input = Vec::with_capacity(sample_count);
        let mut phase = 0.0;
        for i in 0..sample_count {
            let freq = 0.1 + 0.05 * (i as f32 * 0.01).sin();
            phase += freq;
            input.push(Complex32::new(phase.cos(), phase.sin()));
        }

        // Textbook reference
        let mut expected = Vec::with_capacity(sample_count);
        let mut last_ref = Complex32::new(0.0, 0.0);
        for &s in &input {
            let arg = (s * last_ref.conj()).arg();
            last_ref = s;
            expected.push(arg * gain);
        }

        // Fast implementation
        let mut actual = vec![0.0; sample_count];
        let mut last_fast = Complex32::new(0.0, 0.0);
        fast_quadri_demod_slice(&input, &mut actual, &mut last_fast, gain);

        for (i, (&exp, &act)) in expected.iter().zip(actual.iter()).enumerate() {
            let diff = (exp - act).abs();
            assert!(
                diff < 1e-4,
                "Quadri demod mismatch at index {i}: expected {exp}, actual {act}, diff {diff}"
            );
        }
    }
}
