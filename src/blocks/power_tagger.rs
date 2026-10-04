use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;
use futuresdr::runtime::dev::Tag;

/// Helper to compute instantaneous power of various sample types.
pub trait PowerSample: CpuSample {
    fn power(&self) -> f32;
}

impl PowerSample for Complex32 {
    #[inline(always)]
    fn power(&self) -> f32 {
        self.norm_sqr()
    }
}

impl PowerSample for f32 {
    #[inline(always)]
    fn power(&self) -> f32 {
        self * self
    }
}

pub fn tag_matches(tag: &Tag, target: &str) -> bool {
    match tag {
        Tag::String(s) => s == target,
        Tag::Id(id) => id.to_string() == target,
        _ => false,
    }
}

/// PowerTagger monitors the input stream's power level using a moving average window,
/// debounces transitions, and emits stream tags on rising or falling edges.
#[derive(Block)]
pub struct PowerTagger<T: PowerSample> {
    #[input]
    input: DefaultCpuReader<T>,
    #[output]
    output: DefaultCpuWriter<T>,

    off_threshold: Option<f32>,
    off_delay: usize,
    on_threshold: Option<f32>,
    on_delay: usize,
    off_tag: Option<String>,
    on_tag: Option<String>,

    window: Vec<f32>,
    window_pos: usize,
    window_filled: usize,
    moving_sum: f32,

    current_state: bool,
    consecutive_off: usize,
    consecutive_on: usize,

    debug: bool,
    debug_counter: usize,
    debug_interval: usize,

    snr_off_ratio: Option<f32>,
    snr_on_ratio: Option<f32>,
    noise_floor: f32,
    noise_alpha: f32,
}

impl<T: PowerSample> PowerTagger<T> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        off_threshold: Option<f32>,
        off_delay: usize,
        on_threshold: Option<f32>,
        on_delay: usize,
        window_size: usize,
        off_tag: Option<String>,
        on_tag: Option<String>,
    ) -> Self {
        Self::with_full_options(
            off_threshold,
            off_delay,
            on_threshold,
            on_delay,
            window_size,
            off_tag,
            on_tag,
            false,
            None,
            None,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_debug(
        off_threshold: Option<f32>,
        off_delay: usize,
        on_threshold: Option<f32>,
        on_delay: usize,
        window_size: usize,
        off_tag: Option<String>,
        on_tag: Option<String>,
        debug: bool,
    ) -> Self {
        Self::with_full_options(
            off_threshold,
            off_delay,
            on_threshold,
            on_delay,
            window_size,
            off_tag,
            on_tag,
            debug,
            None,
            None,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_full_options(
        off_threshold: Option<f32>,
        off_delay: usize,
        on_threshold: Option<f32>,
        on_delay: usize,
        window_size: usize,
        off_tag: Option<String>,
        on_tag: Option<String>,
        debug: bool,
        snr_off_ratio: Option<f32>,
        snr_on_ratio: Option<f32>,
        noise_alpha: Option<f32>,
    ) -> Self {
        let win_size = window_size.max(1);
        Self {
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
            off_threshold,
            off_delay,
            on_threshold,
            on_delay,
            off_tag,
            on_tag,
            window: vec![0.0; win_size],
            window_pos: 0,
            window_filled: 0,
            moving_sum: 0.0,
            current_state: false,
            consecutive_off: 0,
            consecutive_on: 0,
            debug,
            debug_counter: 0,
            debug_interval: 200_000,
            snr_off_ratio,
            snr_on_ratio,
            noise_floor: 0.0,
            noise_alpha: noise_alpha.unwrap_or(0.001),
        }
    }
}

#[doc(hidden)]
impl<T: PowerSample> Kernel for PowerTagger<T> {
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
            // Forward existing tags
            for tag in in_tags {
                if tag.index < m {
                    out_tags.add_tag(tag.index, tag.tag.clone());
                }
            }

            let win_len = self.window.len();

            for (idx, (src, dst)) in i[..m].iter().zip(o[..m].iter_mut()).enumerate() {
                let p = src.power();

                let old_p = self.window[self.window_pos];
                self.window[self.window_pos] = p;
                self.window_pos = (self.window_pos + 1) % win_len;

                if self.window_filled < win_len {
                    self.window_filled += 1;
                    self.moving_sum += p;
                } else {
                    self.moving_sum += p - old_p;
                }

                let avg_power = self.moving_sum / (self.window_filled as f32);

                // Update noise floor estimation while OFF and not debouncing a signal
                if !self.current_state && self.consecutive_on == 0 {
                    if self.noise_floor == 0.0 {
                        self.noise_floor = avg_power;
                    } else {
                        self.noise_floor = (1.0 - self.noise_alpha) * self.noise_floor
                            + self.noise_alpha * avg_power;
                    }
                }

                let effective_off_th = if let Some(snr_off) = self.snr_off_ratio {
                    if self.noise_floor > 0.0 {
                        Some(self.noise_floor * snr_off)
                    } else {
                        self.off_threshold
                    }
                } else {
                    self.off_threshold
                };

                let effective_on_th = if let Some(snr_on) = self.snr_on_ratio {
                    if self.noise_floor > 0.0 {
                        Some(self.noise_floor * snr_on)
                    } else {
                        self.on_threshold
                    }
                } else {
                    self.on_threshold
                };

                if self.debug {
                    self.debug_counter += 1;
                    if self.debug_counter >= self.debug_interval {
                        self.debug_counter = 0;
                        let db = 10.0 * avg_power.max(1e-12).log10();
                        let noise_db_str =
                            format!("{:.1}dB", 10.0 * self.noise_floor.max(1e-12).log10());
                        let on_db_str = effective_on_th
                            .map(|v| format!("{:.1}dB", 10.0 * v.max(1e-12).log10()))
                            .unwrap_or_else(|| "none".into());
                        let off_db_str = effective_off_th
                            .map(|v| format!("{:.1}dB", 10.0 * v.max(1e-12).log10()))
                            .unwrap_or_else(|| "none".into());
                        eprintln!(
                            "[power_tagger] Power: {:>6.1} dB | Noise: {} | State: {:3} | OnTh: {} | OffTh: {}",
                            db,
                            noise_db_str,
                            if self.current_state { "ON" } else { "OFF" },
                            on_db_str,
                            off_db_str
                        );
                    }
                }

                if self.current_state {
                    // Currently ON: check for loss of power
                    if let Some(off_th) = effective_off_th {
                        if avg_power < off_th {
                            self.consecutive_off += 1;
                            if self.consecutive_off >= self.off_delay {
                                self.current_state = false;
                                self.consecutive_off = 0;
                                self.consecutive_on = 0;
                                if self.debug {
                                    eprintln!(
                                        "[power_tagger] State changed -> OFF (tag: {:?})",
                                        self.off_tag
                                    );
                                }
                                if let Some(ref tag_name) = self.off_tag {
                                    out_tags.add_tag(idx, Tag::String(tag_name.clone()));
                                }
                            }
                        } else {
                            self.consecutive_off = 0;
                        }
                    }
                } else {
                    // Currently OFF: check for appearance of power
                    if let Some(on_th) = effective_on_th {
                        if avg_power >= on_th {
                            self.consecutive_on += 1;
                            if self.consecutive_on >= self.on_delay {
                                self.current_state = true;
                                self.consecutive_on = 0;
                                self.consecutive_off = 0;
                                if self.debug {
                                    eprintln!(
                                        "[power_tagger] State changed -> ON (tag: {:?})",
                                        self.on_tag
                                    );
                                }
                                if let Some(ref tag_name) = self.on_tag {
                                    out_tags.add_tag(idx, Tag::String(tag_name.clone()));
                                }
                            }
                        } else {
                            self.consecutive_on = 0;
                        }
                    } else if let Some(off_th) = effective_off_th {
                        // If only off_threshold is specified, arm immediately when above threshold
                        if avg_power >= off_th {
                            self.current_state = true;
                            self.consecutive_off = 0;
                            if self.debug {
                                eprintln!("[power_tagger] Armed -> ON (power >= off_threshold)");
                            }
                        }
                    }
                }

                *dst = *src;
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
