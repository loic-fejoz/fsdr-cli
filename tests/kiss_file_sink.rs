use anyhow::Result;
use fsdr_cli::blocks::kiss_file_sink::KissFileSink;
use fsdr_cli::blocks::kiss_file_source::KissFileSource;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;
use std::fs;
use std::path::Path;

#[test]
fn test_kiss_file_sink() -> Result<()> {
    // 1. Path to the KISS file in the repository
    let input_filename = "tests/test.kiss";
    let output_filename = "tests/test_output.kiss";

    // Clean up if it exists
    if Path::new(output_filename).exists() {
        fs::remove_file(output_filename)?;
    }

    // 2. Set up FutureSDR flowgraph
    let mut fg = Flowgraph::new();
    let src = KissFileSource::new(input_filename);
    let sink = KissFileSink::new(output_filename);

    let src_id = fg.add_block(src);
    let sink_id = fg.add_block(sink);
    fg.connect_message(src_id, "output", sink_id, "in_port")?;

    // 3. Run flowgraph
    Runtime::new().run(fg)?;

    // 4. Assert
    // The output file should be identical to the input file in terms of framing,
    // though the exact sequence of FENDs might differ slightly if the input had multiple back-to-back FENDs.
    // However, for standard KISS files written by our sink, the content should parse identically.

    let original_bytes = fs::read(input_filename)?;
    let saved_bytes = fs::read(output_filename)?;

    // We can check if the saved bytes contain the expected frames
    // "test.kiss" contains:
    // Frame 1: 00 AA BB
    // Frame 2: 00 CC C0 DD

    // In KISS, with FENDs at start and end:
    // C0 00 AA BB C0
    // C0 00 CC DB DC DD C0

    assert_eq!(original_bytes, saved_bytes);

    // Clean up
    fs::remove_file(output_filename)?;

    Ok(())
}
