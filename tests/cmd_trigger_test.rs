use anyhow::Result;
use fsdr_cli::blocks::CmdTrigger;
use futuresdr::prelude::connect;
use futuresdr::runtime::dev::prelude::*;
use futuresdr::runtime::dev::Tag;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;

#[derive(Block)]
struct TaggedMessageSource {
    #[output]
    output: DefaultCpuWriter<f32>,
    count: usize,
    total: usize,
    events: Vec<(usize, String)>,
}

#[doc(hidden)]
impl Kernel for TaggedMessageSource {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mo: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let (o, mut out_tags) = self.output.slice_with_tags();
        let m = std::cmp::min(self.total - self.count, o.len());

        #[allow(clippy::needless_range_loop)]
        for idx in 0..m {
            let cur = self.count + idx;
            for (ev_idx, ev_name) in &self.events {
                if cur == *ev_idx {
                    out_tags.add_tag(idx, Tag::String(ev_name.clone()));
                }
            }
            o[idx] = 0.42;
        }

        self.count += m;
        self.output.produce(m);

        if self.count >= self.total {
            io.finished = true;
        }

        Ok(())
    }
}

#[test]
fn test_cmd_trigger_execution_and_sigmf() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let flag_path = temp_dir.path().join("output_flag.txt");
    let flag_path_str = flag_path.to_str().unwrap().to_string();

    // Script to execute: appends input_file path into flag_path file
    let script_cmd = format!("echo $input_file >> {}", flag_path_str);

    let mut fg = Flowgraph::new();
    let src = TaggedMessageSource {
        output: DefaultCpuWriter::default(),
        count: 0,
        total: 200,
        events: vec![
            (20, "msgstart".to_string()),
            (30, "msgstart".to_string()), // duplicate start tag, should be ignored
            (80, "msgend".to_string()),
            (110, "msgstart".to_string()),
            (160, "msgend".to_string()),
        ],
    };

    let sink = CmdTrigger::<f32>::new(
        "msgstart".to_string(),
        "msgend".to_string(),
        script_cmd,
        48000.0,
    );

    connect!(fg, src > sink;);

    let rt = Runtime::new();
    let _ = rt.run(fg)?;

    // Give background threads a short moment to finish command execution
    std::thread::sleep(std::time::Duration::from_millis(500));

    let content = std::fs::read_to_string(&flag_path)?;
    let lines: Vec<&str> = content.lines().collect();

    // Exactly 2 commands should have been executed
    assert_eq!(
        lines.len(),
        2,
        "Expected 2 triggered messages, found: {:?}",
        lines
    );
    assert!(lines[0].ends_with("message.sigmf-meta"));
    assert!(lines[1].ends_with("message.sigmf-meta"));

    Ok(())
}
