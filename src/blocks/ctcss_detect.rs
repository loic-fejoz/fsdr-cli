use futuresdr::runtime::dev::prelude::*;
use std::f32::consts::PI;

/// CtcssDetect runs a Goertzel filter on demodulated audio stream
/// to detect a target CTCSS sub-audible tone and insert `tag_name` (e.g. "msgstart").
#[derive(Block)]
pub struct CtcssDetect {
    #[input]
    input: DefaultCpuReader<f32>,
    #[output]
    output: DefaultCpuWriter<f32>,

    threshold: f32,
    tag_name: String,
    tone_freq: f32,
    coeff: f32,
    block_size: usize,
    norm_factor: f32,

    s1: f32,
    s2: f32,
    block_energy: f32,
    block_count: usize,
    tone_power_avg: f32,
    alpha_smooth: f32,
    is_detected: bool,

    debug: bool,
    debug_counter: usize,
    debug_interval: usize,
}

impl CtcssDetect {
    pub fn new(
        samp_rate: f32,
        tone_freq: f32,
        threshold: f32,
        duration_samples: usize,
        tag_name: String,
    ) -> Self {
        Self::with_debug(
            samp_rate,
            tone_freq,
            threshold,
            duration_samples,
            tag_name,
            false,
        )
    }

    pub fn with_debug(
        samp_rate: f32,
        tone_freq: f32,
        threshold: f32,
        duration_samples: usize,
        tag_name: String,
        debug: bool,
    ) -> Self {
        let omega = 2.0 * PI * (tone_freq / samp_rate);
        let coeff = 2.0 * omega.cos();

        let block_size = duration_samples.max(32);
        let norm_factor = (block_size * block_size) as f32 / 4.0;

        let alpha_smooth = 0.5f32;

        Self {
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
            threshold,
            tag_name,
            tone_freq,
            coeff,
            block_size,
            norm_factor,
            s1: 0.0,
            s2: 0.0,
            block_energy: 0.0,
            block_count: 0,
            tone_power_avg: 0.0,
            alpha_smooth,
            is_detected: false,
            debug,
            debug_counter: 0,
            debug_interval: (samp_rate * 0.5).round().max(1000.0) as usize,
        }
    }
}

#[doc(hidden)]
impl Kernel for CtcssDetect {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mo: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let (i, in_tags) = self.input.slice_with_tags();
        let (o, mut out_tags) = self.output.slice_with_tags();
        let i_len = i.len();
        let m = std::cmp::min(i_len, o.len());

        if m > 0 {
            // Forward incoming tags
            for tag in in_tags {
                if tag.index < m {
                    out_tags.add_tag(tag.index, tag.tag.clone());
                }
            }

            for (idx, (src, dst)) in i[..m].iter().zip(o[..m].iter_mut()).enumerate() {
                let x = *src;

                // Goertzel recurrence
                let s0 = x + self.coeff * self.s1 - self.s2;
                self.s2 = self.s1;
                self.s1 = s0;
                self.block_energy += x * x;
                self.block_count += 1;

                if self.block_count >= self.block_size {
                    let raw_power = (self.s1 * self.s1 + self.s2 * self.s2
                        - self.coeff * self.s1 * self.s2)
                        / self.norm_factor;

                    let total_block_power = self.block_energy / self.block_size as f32;
                    let snr = raw_power / total_block_power.max(1e-9);

                    self.tone_power_avg += self.alpha_smooth * (raw_power - self.tone_power_avg);
                    self.s1 = 0.0;
                    self.s2 = 0.0;
                    self.block_energy = 0.0;
                    self.block_count = 0;

                    if self.debug {
                        self.debug_counter += self.block_size;
                        if self.debug_counter >= self.debug_interval {
                            self.debug_counter = 0;
                            eprintln!(
                                "[ctcss_detect] Tone: {:.1} Hz | Power: {:.6} | Thresh: {:.6} | SNR: {:.5} | Detected: {}",
                                self.tone_freq,
                                self.tone_power_avg,
                                self.threshold,
                                snr,
                                self.is_detected
                            );
                        }
                    }

                    // To be a valid CTCSS tone:
                    // 1. Absolute tone power must exceed threshold.
                    // 2. Must not be open-discriminator broadband noise (total_block_power < 0.05 or SNR >= 0.001).
                    let valid_carrier_and_tone = self.tone_power_avg >= self.threshold
                        && (total_block_power < 0.05 || snr >= 0.001);

                    if valid_carrier_and_tone {
                        if !self.is_detected {
                            self.is_detected = true;
                            if self.debug {
                                eprintln!(
                                    "[ctcss_detect] Tone detected -> emitted tag: '{}'",
                                    self.tag_name
                                );
                            }
                            out_tags.add_tag(idx, Tag::String(self.tag_name.clone()));
                        }
                    } else if self.is_detected {
                        self.is_detected = false;
                        if self.debug {
                            eprintln!("[ctcss_detect] Tone lost");
                        }
                    }
                }

                *dst = x;
            }

            self.input.consume(m);
            self.output.produce(m);
        }

        if self.input.finished() && m == i_len {
            io.finished = true;
        }

        Ok(())
    }
}
