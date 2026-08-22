use anyhow::Result;
use fsdr_cli::blocks::TimerTagger;
use futuresdr::prelude::connect;
use futuresdr::runtime::dev::prelude::*;
use futuresdr::runtime::dev::Tag;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;

// Custom block that outputs samples and attaches a tag at a given sample index
#[derive(Block)]
struct TaggedSource {
    #[output]
    output: DefaultCpuWriter<f32>,
    count: usize,
    total: usize,
    tag_at: usize,
    tag_name: String,
    second_tag_at: Option<usize>,
    second_tag_name: Option<String>,
}

#[doc(hidden)]
impl Kernel for TaggedSource {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mo: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let (o, mut out_tags) = self.output.slice_with_tags();
        let m = std::cmp::min(self.total - self.count, o.len());
        for idx in 0..m {
            let cur = self.count + idx;
            if cur == self.tag_at {
                out_tags.add_tag(idx, Tag::String(self.tag_name.clone()));
            }
            if let (Some(s_at), Some(s_name)) = (self.second_tag_at, &self.second_tag_name) {
                if cur == s_at {
                    out_tags.add_tag(idx, Tag::String(s_name.clone()));
                }
            }
            o[idx] = 1.0;
        }
        self.count += m;
        self.output.produce(m);
        if self.count >= self.total {
            io.finished = true;
        }
        Ok(())
    }
}

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
fn test_timer_tagger_timeout_and_cancel() -> Result<()> {
    // 1. Timeout scenario: msgstart at sample 10, timeout 30 samples -> msgend expected at sample 40
    let mut fg = Flowgraph::new();
    let src = TaggedSource {
        output: DefaultCpuWriter::default(),
        count: 0,
        total: 100,
        tag_at: 10,
        tag_name: "msgstart".to_string(),
        second_tag_at: None,
        second_tag_name: None,
    };
    let timer = TimerTagger::<f32>::new("msgstart".to_string(), "msgend".to_string(), 30);
    let snk = TagCollectorSink::<f32>::new();

    connect!(fg, src > timer > snk;);

    let rt = Runtime::new();
    let fg = rt.run(fg)?;

    let snk_blk = fg.block(&snk)?;
    let tags = snk_blk.tags();

    let tag_names: Vec<(usize, String)> = tags
        .iter()
        .filter_map(|(idx, t)| match t {
            Tag::String(s) => Some((*idx, s.clone())),
            _ => None,
        })
        .collect();

    assert!(tag_names.contains(&(10, "msgstart".to_string())));
    assert!(tag_names.contains(&(40, "msgend".to_string())));

    // 2. Cancellation scenario: natural msgend at sample 25 cancels the 30-sample timeout (sample 40)
    let mut fg2 = Flowgraph::new();
    let src2 = TaggedSource {
        output: DefaultCpuWriter::default(),
        count: 0,
        total: 100,
        tag_at: 10,
        tag_name: "msgstart".to_string(),
        second_tag_at: Some(25),
        second_tag_name: Some("msgend".to_string()),
    };
    let timer2 = TimerTagger::<f32>::new("msgstart".to_string(), "msgend".to_string(), 30);
    let snk2 = TagCollectorSink::<f32>::new();

    connect!(fg2, src2 > timer2 > snk2;);

    let rt2 = Runtime::new();
    let fg2 = rt2.run(fg2)?;

    let snk_blk2 = fg2.block(&snk2)?;
    let tags2 = snk_blk2.tags();

    let msgends: Vec<usize> = tags2
        .iter()
        .filter_map(|(idx, t)| match t {
            Tag::String(s) if s == "msgend" => Some(*idx),
            _ => None,
        })
        .collect();

    assert_eq!(
        msgends,
        vec![25],
        "Timeout should be canceled; only the natural msgend at 25 should exist"
    );

    Ok(())
}
