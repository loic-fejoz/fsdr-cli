use futuresdr::runtime::dev::prelude::*;

/// AFC (Automatic Frequency Control) block for 1D float (f32) baseband signals.
/// Tracks and removes DC offset caused by carrier frequency offset post-FM demodulation.
#[derive(Block)]
pub struct AfcFf {
    #[input]
    input: DefaultCpuReader<f32>,
    #[output]
    output: DefaultCpuWriter<f32>,

    alpha: f32,     // Smoothing factor for DC offset estimation
    limit: f32,     // Max allowed DC offset correction limit
    dc_offset: f32, // Current estimated DC offset
}

impl AfcFf {
    pub fn new(alpha: f32, limit: f32) -> Self {
        Self {
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
            alpha: alpha.clamp(1e-7, 1.0),
            limit: limit.abs(),
            dc_offset: 0.0,
        }
    }

    pub fn dc_offset(&self) -> f32 {
        self.dc_offset
    }
}

#[doc(hidden)]
impl Kernel for AfcFf {
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
                let sample = *src;
                self.dc_offset = (1.0 - self.alpha) * self.dc_offset + self.alpha * sample;
                if self.limit > 0.0 {
                    self.dc_offset = self.dc_offset.clamp(-self.limit, self.limit);
                }
                *dst = sample - self.dc_offset;
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
