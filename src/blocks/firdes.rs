use crate::blocks::fft_block::create_window;
use futuresdr::num_complex::Complex32;
use std::f32::consts::PI;

pub fn firdes_filter_length(transition_bw: f32, window_type: &str) -> usize {
    let win_str = window_type.to_uppercase();
    let factor = match win_str.as_str() {
        "BLACKMAN" | "WIN_BLACKMAN" | "WINDOW.WIN_BLACKMAN" => 4.0,
        "HANN" | "HANNING" | "WIN_HANN" | "WINDOW.WIN_HANN" => 3.1,
        "BOXCAR" | "RECTANGULAR" | "NONE" | "WIN_RECTANGULAR" | "WINDOW.WIN_RECTANGULAR" => 0.9,
        _ => 3.3, // Hamming default
    };

    let len = (factor / transition_bw).ceil() as usize;
    if len.is_multiple_of(2) {
        len + 1
    } else {
        len
    }
}

pub fn firdes_lowpass_f(cutoff_rate: f32, transition_bw: f32, window_type: &str) -> Vec<f32> {
    let length = firdes_filter_length(transition_bw, window_type);
    let window = create_window(window_type, length);
    let mid = (length - 1) as f32 / 2.0;

    let mut taps = Vec::with_capacity(length);
    let mut sum = 0.0;

    for (i, &w) in window.iter().enumerate() {
        let x = (i as f32) - mid;
        let sinc_val = if x.abs() < 1e-7 {
            2.0 * cutoff_rate
        } else {
            (2.0 * PI * cutoff_rate * x).sin() / (PI * x)
        };
        let val = sinc_val * w;
        taps.push(val);
        sum += val;
    }

    if sum.abs() > 1e-9 {
        for tap in &mut taps {
            *tap /= sum;
        }
    }

    taps
}

pub fn firdes_bandpass_c(
    low_cutoff: f32,
    high_cutoff: f32,
    transition_bw: f32,
    window_type: &str,
) -> Vec<Complex32> {
    let cutoff_rate = (high_cutoff - low_cutoff) / 2.0;
    let center_freq = (high_cutoff + low_cutoff) / 2.0;

    let lp_taps = firdes_lowpass_f(cutoff_rate, transition_bw, window_type);
    let length = lp_taps.len();
    let mid = (length - 1) as f32 / 2.0;

    let mut taps = Vec::with_capacity(length);
    for (i, &lp_tap) in lp_taps.iter().enumerate() {
        let x = (i as f32) - mid;
        let phase = 2.0 * PI * center_freq * x;
        let shift = Complex32::new(phase.cos(), phase.sin());
        taps.push(shift * lp_tap);
    }

    taps
}
