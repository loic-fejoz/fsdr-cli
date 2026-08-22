use anyhow::Result;
use fsdr_cli::blocks::PowerTagger;
use futuresdr::blocks::VectorSource;
use futuresdr::num_complex::Complex32;
use futuresdr::prelude::connect;
use futuresdr::runtime::dev::prelude::*;
use futuresdr::runtime::dev::Tag;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;

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
fn test_power_tagger_falling_and_rising_edges() -> Result<()> {
    let mut fg = Flowgraph::new();

    // Generate samples: 100 high power, 100 low power, 100 high power
    let mut samples = Vec::new();
    for _ in 0..100 {
        samples.push(Complex32::new(1.0, 1.0)); // Power = 2.0
    }
    for _ in 0..100 {
        samples.push(Complex32::new(0.01, 0.01)); // Power = 0.0002
    }
    for _ in 0..100 {
        samples.push(Complex32::new(1.0, 1.0)); // Power = 2.0
    }

    let src = VectorSource::<Complex32>::new(samples);
    let tagger = PowerTagger::<Complex32>::new(
        Some(0.1), // off_threshold
        10,        // off_delay (10 samples)
        Some(0.5), // on_threshold
        10,        // on_delay (10 samples)
        10,        // window_size (10 samples)
        Some("msgend".to_string()),
        Some("msgstart".to_string()),
    );
    let snk = TagCollectorSink::<Complex32>::new();

    connect!(fg, src > tagger > snk;);

    let rt = Runtime::new();
    let fg = rt.run(fg)?;

    let snk_blk = fg.block(&snk)?;
    let tags = snk_blk.tags();

    // Check that we got tags
    let tag_names: Vec<String> = tags
        .iter()
        .filter_map(|(_idx, t)| match t {
            Tag::String(s) => Some(s.clone()),
            _ => None,
        })
        .collect();

    assert!(tag_names.contains(&"msgend".to_string()));
    assert!(tag_names.contains(&"msgstart".to_string()));

    Ok(())
}

#[test]
fn test_power_tagger_debounce_avoids_false_off() -> Result<()> {
    let mut fg = Flowgraph::new();

    // Generate high power with a brief 3-sample dip (< 10 sample delay)
    let mut samples = vec![Complex32::new(1.0, 1.0); 50];
    samples.extend(vec![Complex32::new(0.0, 0.0); 3]);
    samples.extend(vec![Complex32::new(1.0, 1.0); 50]);

    let src = VectorSource::<Complex32>::new(samples);
    let tagger = PowerTagger::<Complex32>::new(
        Some(0.1),
        10, // requires 10 consecutive samples below threshold
        Some(0.5),
        10,
        5,
        Some("msgend".to_string()),
        Some("msgstart".to_string()),
    );
    let snk = TagCollectorSink::<Complex32>::new();

    connect!(fg, src > tagger > snk;);

    let rt = Runtime::new();
    let fg = rt.run(fg)?;

    let snk_blk = fg.block(&snk)?;
    let tags = snk_blk.tags();

    let has_msgend = tags.iter().any(|(_idx, t)| match t {
        Tag::String(s) => s == "msgend",
        _ => false,
    });

    assert!(
        !has_msgend,
        "Debounce should prevent brief dip from emitting msgend"
    );

    Ok(())
}
