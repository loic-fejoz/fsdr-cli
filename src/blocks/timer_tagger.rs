use super::power_tagger::tag_matches;
use futuresdr::runtime::dev::prelude::*;

/// TimerTagger arms a timer on `start_tag` and injects `end_tag` after `duration_samples`
/// unless a natural `end_tag` arrives earlier.
#[derive(Block)]
pub struct TimerTagger<T: CpuSample> {
    #[input]
    input: DefaultCpuReader<T>,
    #[output]
    output: DefaultCpuWriter<T>,

    start_tag: String,
    end_tag: String,
    duration_samples: usize,

    total_samples_processed: usize,
    timeout_target_sample: Option<usize>,
    debug: bool,
}

impl<T: CpuSample> TimerTagger<T> {
    pub fn new(start_tag: String, end_tag: String, duration_samples: usize) -> Self {
        Self::with_debug(start_tag, end_tag, duration_samples, false)
    }

    pub fn with_debug(
        start_tag: String,
        end_tag: String,
        duration_samples: usize,
        debug: bool,
    ) -> Self {
        Self {
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
            start_tag,
            end_tag,
            duration_samples: duration_samples.max(1),
            total_samples_processed: 0,
            timeout_target_sample: None,
            debug,
        }
    }
}

#[doc(hidden)]
impl<T: CpuSample> Kernel for TimerTagger<T> {
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
            // Process incoming tags
            for tag in in_tags {
                if tag.index < m {
                    let abs_index = self.total_samples_processed + tag.index;

                    if tag_matches(&tag.tag, &self.start_tag) {
                        if self.timeout_target_sample.is_none() {
                            self.timeout_target_sample = Some(abs_index + self.duration_samples);
                            if self.debug {
                                eprintln!(
                                    "[timer_tagger] Armed timer (duration: {} samples) on '{}'",
                                    self.duration_samples, self.start_tag
                                );
                            }
                        }
                    } else if tag_matches(&tag.tag, &self.end_tag) {
                        if self.timeout_target_sample.is_some() && self.debug {
                            eprintln!(
                                "[timer_tagger] Canceled timer on natural '{}'",
                                self.end_tag
                            );
                        }
                        self.timeout_target_sample = None;
                    }
                    out_tags.add_tag(tag.index, tag.tag.clone());
                }
            }

            // Check if timeout expires within this chunk
            if let Some(target) = self.timeout_target_sample {
                let chunk_start = self.total_samples_processed;
                let chunk_end = self.total_samples_processed + m;

                if target >= chunk_start && target < chunk_end {
                    let rel_index = target - chunk_start;
                    if self.debug {
                        eprintln!(
                            "[timer_tagger] Timeout reached! Injected tag '{}'",
                            self.end_tag
                        );
                    }
                    out_tags.add_tag(rel_index, Tag::String(self.end_tag.clone()));
                    self.timeout_target_sample = None;
                } else if target < chunk_start {
                    if self.debug {
                        eprintln!(
                            "[timer_tagger] Timeout reached! Injected tag '{}'",
                            self.end_tag
                        );
                    }
                    out_tags.add_tag(0, Tag::String(self.end_tag.clone()));
                    self.timeout_target_sample = None;
                }
            }

            o[..m].copy_from_slice(&i[..m]);
            self.total_samples_processed += m;
            self.input.consume(m);
            self.output.produce(m);
        }

        if self.input.finished() && m == i_len {
            io.finished = true;
        }

        Ok(())
    }
}
