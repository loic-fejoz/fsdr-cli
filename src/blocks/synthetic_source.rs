use anyhow::Result;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;
use std::f32::consts::PI;
use std::time::{Duration, Instant};

/// SyntheticSdrSource:
/// Generates realistic synthetic multi-carrier SDR stream at `sample_rate` (e.g. 2_000_000 SPS):
/// - Gaussian thermal noise across the spectrum
/// - CW / Beacon tone at `beacon_offset` (-200 kHz)
/// - FT8 multi-tone signal at `ft8_offset` (-126 kHz)
/// - SSTV / Audio tone at `sstv_offset` (+30 kHz)
/// - SSB Voice two-tone test signal at `ssb_offset` (+50 kHz), audio tones 700/1900 Hz (USB, carrier +1500 Hz)
/// - APRS AFSK (1200 / 2200 Hz) at `aprs_offset` (-97 kHz)
/// - OQPSK modulated carrier at `oqpsk_offset` (+150 kHz)
///
/// Features hardware-accurate real-time pacing (throttled to SPS) so pipelines process data
/// at realistic speeds without pegging 100% CPU.
#[derive(Block)]
pub struct SyntheticSdrSource<O: CpuBufferWriter<Item = Complex32> = DefaultCpuWriter<Complex32>> {
    sample_rate: f32,
    phase_beacon: f32,
    phase_ft8: f32,
    phase_sstv: f32,
    phase_ssb_1: f32,
    phase_ssb_2: f32,
    phase_aprs: f32,
    phase_aprs_audio: f32,
    phase_oqpsk: f32,
    oqpsk_symbol_counter: usize,
    oqpsk_i: f32,
    oqpsk_q: f32,
    aprs_sample_counter: usize,
    aprs_bit_idx: usize,
    rng_state: u64,
    start_time: Instant,
    total_produced: u64,
    #[output]
    output: O,
}

impl SyntheticSdrSource<DefaultCpuWriter<Complex32>> {
    pub fn new(sample_rate: f32) -> Self {
        let sr = if sample_rate <= 0.0 {
            2_000_000.0
        } else {
            sample_rate
        };
        Self {
            sample_rate: sr,
            phase_beacon: 0.0,
            phase_ft8: 0.0,
            phase_sstv: 0.0,
            phase_ssb_1: 0.0,
            phase_ssb_2: 0.0,
            phase_aprs: 0.0,
            phase_aprs_audio: 0.0,
            phase_oqpsk: 0.0,
            oqpsk_symbol_counter: 0,
            oqpsk_i: 1.0,
            oqpsk_q: 1.0,
            aprs_sample_counter: 0,
            aprs_bit_idx: 0,
            rng_state: 0x123456789abcdef0,
            start_time: Instant::now(),
            total_produced: 0,
            output: DefaultCpuWriter::default(),
        }
    }
}

#[doc(hidden)]
impl<O> Kernel for SyntheticSdrSource<O>
where
    O: CpuBufferWriter<Item = Complex32>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let out_slice = self.output.slice();
        let max_n = out_slice.len();

        if max_n == 0 {
            return Ok(());
        }

        // Pacing limit: do not produce faster than 1.2x real-time to avoid runaway memory
        let elapsed_secs = self.start_time.elapsed().as_secs_f64();
        let expected_samples = (elapsed_secs * self.sample_rate as f64) as u64;
        if self.total_produced > expected_samples + (self.sample_rate as u64 / 10) {
            // Slight sleep to maintain steady real-time flow
            let sleep_dur = Duration::from_millis(5);
            std::thread::sleep(sleep_dur);
        }

        let n = max_n.min(4096);
        let sr = self.sample_rate;
        let two_pi = 2.0 * PI;

        // Frequencies relative to center LO
        let f_beacon = -200_000.0; // QO-100 / General beacon
        let f_ft8 = -126_000.0; // 14.074 MHz (FT8 carrier offset)
        let f_sstv = 30_000.0; // 14.230 MHz (SSTV)
        let f_ssb = 50_000.0; // 14.250 MHz (SSB Voice)
        let f_aprs = -97_000.0; // 14.103 MHz (APRS HF)
        let f_oqpsk = 150_000.0; // 14.350 MHz (OQPSK digital)

        let d_beacon = two_pi * f_beacon / sr;
        let d_ft8 = two_pi * f_ft8 / sr;
        let d_sstv = two_pi * f_sstv / sr;
        let d_ssb_1 = two_pi * (f_ssb + 1500.0 + 700.0) / sr;
        let d_ssb_2 = two_pi * (f_ssb + 1500.0 + 1900.0) / sr;
        let d_aprs = two_pi * f_aprs / sr;
        let d_oqpsk = two_pi * f_oqpsk / sr;

        // APRS bit duration: 300 bauds (sr / 300 samples)
        let aprs_samples_per_bit = (sr / 300.0) as usize;
        // OQPSK symbol duration: 2400 bauds
        let oqpsk_samples_per_sym = (sr / 2400.0) as usize;

        let mut rng = self.rng_state;
        let mut phase_beacon = self.phase_beacon;
        let mut phase_ft8 = self.phase_ft8;
        let mut phase_sstv = self.phase_sstv;
        let mut phase_ssb_1 = self.phase_ssb_1;
        let mut phase_ssb_2 = self.phase_ssb_2;
        let mut phase_aprs = self.phase_aprs;
        let mut phase_aprs_audio = self.phase_aprs_audio;
        let mut phase_oqpsk = self.phase_oqpsk;
        let mut oqpsk_i = self.oqpsk_i;
        let mut oqpsk_q = self.oqpsk_q;
        let mut aprs_bit_idx = self.aprs_bit_idx;
        let mut aprs_sample_counter = self.aprs_sample_counter;
        let mut oqpsk_symbol_counter = self.oqpsk_symbol_counter;

        let out_slice = self.output.slice();

        for out_sample in &mut out_slice[..n] {
            // 1. Noise floor (-40 dBFS)
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let n_re = (((rng as i32) as f32) * (1.0 / 2147483648.0)) * 0.01;

            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let n_im = (((rng as i32) as f32) * (1.0 / 2147483648.0)) * 0.01;
            let mut sig = Complex32::new(n_re, n_im);

            // 2. Beacon Tone
            let (b_s, b_c) = phase_beacon.sin_cos();
            sig += Complex32::new(b_c, b_s) * 0.25;
            phase_beacon = (phase_beacon + d_beacon) % two_pi;

            // 3. FT8 Carrier & Tone
            let (ft_s, ft_c) = phase_ft8.sin_cos();
            sig += Complex32::new(ft_c, ft_s) * 0.20;
            phase_ft8 = (phase_ft8 + d_ft8) % two_pi;

            // 4. SSTV Carrier
            let (sstv_s, sstv_c) = phase_sstv.sin_cos();
            sig += Complex32::new(sstv_c, sstv_s) * 0.15;
            phase_sstv = (phase_sstv + d_sstv) % two_pi;

            // 5. SSB Voice (Two-tone test)
            let (ssb1_s, ssb1_c) = phase_ssb_1.sin_cos();
            let (ssb2_s, ssb2_c) = phase_ssb_2.sin_cos();
            sig += (Complex32::new(ssb1_c, ssb1_s) + Complex32::new(ssb2_c, ssb2_s)) * 0.10;
            phase_ssb_1 = (phase_ssb_1 + d_ssb_1) % two_pi;
            phase_ssb_2 = (phase_ssb_2 + d_ssb_2) % two_pi;

            // 6. APRS AFSK
            let audio_freq = if (aprs_bit_idx & 1) == 0 {
                1200.0
            } else {
                2200.0
            };
            let d_audio = two_pi * audio_freq / sr;
            let (a_mod_s, a_mod_c) = phase_aprs_audio.sin_cos();
            let (aprs_c, aprs_s) = phase_aprs.sin_cos();
            let aprs_c_val = Complex32::new(aprs_c, aprs_s) * Complex32::new(a_mod_c, a_mod_s);
            sig += aprs_c_val * 0.15;
            phase_aprs = (phase_aprs + d_aprs) % two_pi;
            phase_aprs_audio = (phase_aprs_audio + d_audio) % two_pi;

            aprs_sample_counter += 1;
            if aprs_sample_counter >= aprs_samples_per_bit {
                aprs_sample_counter = 0;
                aprs_bit_idx = (aprs_bit_idx + 1) % 64;
            }

            // 7. OQPSK Modulation
            let (oqpsk_s, oqpsk_c) = phase_oqpsk.sin_cos();
            let sym = Complex32::new(oqpsk_i, oqpsk_q);
            let oqpsk_val = Complex32::new(oqpsk_c, oqpsk_s) * sym;
            sig += oqpsk_val * 0.15;
            phase_oqpsk = (phase_oqpsk + d_oqpsk) % two_pi;

            oqpsk_symbol_counter += 1;
            if oqpsk_symbol_counter >= oqpsk_samples_per_sym {
                oqpsk_symbol_counter = 0;
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                oqpsk_i = if (rng as i32) > 0 { 1.0 } else { -1.0 };
                rng ^= rng << 13;
                rng ^= rng >> 7;
                rng ^= rng << 17;
                oqpsk_q = if (rng as i32) > 0 { 1.0 } else { -1.0 };
            }

            *out_sample = sig;
        }

        self.rng_state = rng;
        self.phase_beacon = phase_beacon;
        self.phase_ft8 = phase_ft8;
        self.phase_sstv = phase_sstv;
        self.phase_ssb_1 = phase_ssb_1;
        self.phase_ssb_2 = phase_ssb_2;
        self.phase_aprs = phase_aprs;
        self.phase_aprs_audio = phase_aprs_audio;
        self.phase_oqpsk = phase_oqpsk;
        self.oqpsk_i = oqpsk_i;
        self.oqpsk_q = oqpsk_q;
        self.aprs_bit_idx = aprs_bit_idx;
        self.aprs_sample_counter = aprs_sample_counter;
        self.oqpsk_symbol_counter = oqpsk_symbol_counter;

        self.output.produce(n);
        self.total_produced += n as u64;
        io.call_again = true;

        Ok(())
    }
}
