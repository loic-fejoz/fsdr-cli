use anyhow::Result;
use fsdr_cli::blocks::CtcssDetect;
use futuresdr::blocks::VectorSource;
use futuresdr::prelude::connect;
use futuresdr::runtime::dev::prelude::*;
use futuresdr::runtime::dev::Tag;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;
use std::f32::consts::PI;

#[derive(Block)]
struct TagCollectorSink<T: CpuSample> {
    #[input]
    input: DefaultCpuReader<T>,
    items: Vec<T>,
    tags: Vec<(usize, Tag)>,
    total_consumed: usize,
}

impl<T: CpuSample> TagCollectorSink<T> {
    fn new() -> Self {
        Self {
            input: DefaultCpuReader::default(),
            items: Vec::new(),
            tags: Vec::new(),
            total_consumed: 0,
        }
    }
    fn tags(&self) -> &[(usize, Tag)] {
        &self.tags
    }
}

#[doc(hidden)]
impl<T: CpuSample> Kernel for TagCollectorSink<T> {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mo: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let (i, in_tags) = self.input.slice_with_tags();
        let m = i.len();
        if m > 0 {
            for tag in in_tags {
                if tag.index < m {
                    self.tags
                        .push((self.total_consumed + tag.index, tag.tag.clone()));
                }
            }
            self.items.extend_from_slice(i);
            self.total_consumed += m;
            self.input.consume(m);
        }
        if self.input.finished() {
            io.finished = true;
        }
        Ok(())
    }
}

#[test]
fn test_ctcss_detect_positive_and_negative_tones() -> Result<()> {
    let samp_rate = 8000.0f32;
    let target_tone = 88.5f32;
    let n_samples = 4000; // 0.5s

    // 1. Positive test: Signal with target tone 88.5 Hz
    let mut positive_signal = Vec::new();
    for i in 0..n_samples {
        let t = i as f32 / samp_rate;
        let sample = (2.0 * PI * target_tone * t).sin() * 0.5;
        positive_signal.push(sample);
    }

    let mut fg = Flowgraph::new();
    let src = VectorSource::<f32>::new(positive_signal);
    let detect = CtcssDetect::new(
        samp_rate,
        target_tone,
        0.005,                                // threshold
        (0.050 * samp_rate).round() as usize, // 50ms duration
        "msgstart".to_string(),
    );
    let snk = TagCollectorSink::<f32>::new();

    connect!(fg, src > detect > snk;);

    let rt = Runtime::new();
    let fg = rt.run(fg)?;

    let snk_blk = fg.block(&snk)?;
    let tags = snk_blk.tags();

    let has_msgstart = tags.iter().any(|(_idx, t)| match t {
        Tag::String(s) => s == "msgstart",
        _ => false,
    });
    assert!(has_msgstart, "CtcssDetect should detect 88.5 Hz tone");

    // 2. Negative test: Signal with different tone (131.8 Hz)
    let different_tone = 131.8f32;
    let mut negative_signal = Vec::new();
    for i in 0..n_samples {
        let t = i as f32 / samp_rate;
        let sample = (2.0 * PI * different_tone * t).sin() * 0.5;
        negative_signal.push(sample);
    }

    let mut fg2 = Flowgraph::new();
    let src2 = VectorSource::<f32>::new(negative_signal);
    let detect2 = CtcssDetect::new(
        samp_rate,
        target_tone,
        0.005,
        (0.050 * samp_rate).round() as usize,
        "msgstart".to_string(),
    );
    let snk2 = TagCollectorSink::<f32>::new();

    connect!(fg2, src2 > detect2 > snk2;);

    let rt2 = Runtime::new();
    let fg2 = rt2.run(fg2)?;

    let snk_blk2 = fg2.block(&snk2)?;
    let tags2 = snk_blk2.tags();

    let has_false_msgstart = tags2.iter().any(|(_idx, t)| match t {
        Tag::String(s) => s == "msgstart",
        _ => false,
    });
    assert!(
        !has_false_msgstart,
        "CtcssDetect should reject 131.8 Hz tone when tuned to 88.5 Hz"
    );

    Ok(())
}
