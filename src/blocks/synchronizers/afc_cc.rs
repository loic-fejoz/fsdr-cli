use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;

/// AFC (Automatic Frequency Control) block for Complex32 IQ signals.
/// Tracks carrier frequency error using a Frequency-Locked Loop (FLL) with an internal NCO.
#[derive(Block)]
pub struct AfcCc {
    #[input]
    input: DefaultCpuReader<Complex32>,
    #[output]
    output: DefaultCpuWriter<Complex32>,

    alpha: f32,        // Loop gain / tracking speed
    max_freq_rad: f32, // Maximum frequency offset limit in rad/sample
    freq_est: f32,     // Current estimated frequency offset (rad/sample)
    phase: f32,        // NCO phase accumulator
    prev_sample: Complex32,
}

impl AfcCc {
    pub fn new(alpha: f32, max_freq_rad: f32) -> Self {
        Self {
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
            alpha: alpha.clamp(1e-7, 1.0),
            max_freq_rad: max_freq_rad.abs(),
            freq_est: 0.0,
            phase: 0.0,
            prev_sample: Complex32::new(1.0, 0.0),
        }
    }

    pub fn freq_est(&self) -> f32 {
        self.freq_est
    }
}

#[doc(hidden)]
impl Kernel for AfcCc {
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
            for tag in in_tags {
                if tag.index < m {
                    out_tags.add_tag(tag.index, tag.tag.clone());
                }
            }

            for (src, dst) in i[..m].iter().zip(o[..m].iter_mut()) {
                let input_sample = *src;

                // NCO rotation: y[n] = x[n] * exp(-j * phase)
                let nco_rot = Complex32::new((-self.phase).cos(), (-self.phase).sin());
                let rot_sample = input_sample * nco_rot;

                // Frequency Error Detector (FED): Im(rot_sample * prev_sample*)
                let conj_prev = Complex32::new(self.prev_sample.re, -self.prev_sample.im);
                let diff = rot_sample * conj_prev;
                let norm = (rot_sample.norm_sqr() * self.prev_sample.norm_sqr())
                    .sqrt()
                    .max(1e-12);
                let freq_err = (diff.im / norm).clamp(-1.0, 1.0);

                // Update frequency estimate (PI / integrator)
                self.freq_est += self.alpha * freq_err;
                if self.max_freq_rad > 0.0 {
                    self.freq_est = self.freq_est.clamp(-self.max_freq_rad, self.max_freq_rad);
                }

                // Update NCO phase
                self.phase = (self.phase + self.freq_est) % (2.0 * std::f32::consts::PI);
                self.prev_sample = rot_sample;

                *dst = rot_sample;
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
