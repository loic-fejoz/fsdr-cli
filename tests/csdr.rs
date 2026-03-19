use anyhow::Result;
use fsdr_cli::csdr_cmd::CsdrParser;
use fsdr_cli::grc::converter::Grc2FutureSdr;
use futuresdr::blocks::VectorSink;
use futuresdr::blocks::VectorSource;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;

#[test]
pub fn parse_load_kiss() {
    let cmds = "load_kiss tests/test.kiss";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(1, grc.blocks.len());
    assert_eq!("satellites_kiss_file_source", grc.blocks[0].id);
    assert_eq!("tests/test.kiss", grc.blocks[0].parameters["file"]);
}

#[test]
pub fn parse_save_kiss() {
    let cmds = "csdr load_kiss tests/test.kiss | save_kiss tests/test_loaded_saved.kiss";
    let result = CsdrParser::parse_multiple_commands(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(2, grc.blocks.len());
    assert_eq!("satellites_kiss_file_source", grc.blocks[0].id);
    assert_eq!("satellites_kiss_file_sink", grc.blocks[1].id);
    assert_eq!(
        "tests/test_loaded_saved.kiss",
        grc.blocks[1].parameters["file"]
    );
    assert_eq!(1, grc.connections.len());
}

#[test]
pub fn parse_convert_u8_f() {
    let cmds = "convert_u8_f";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("blocks_uchar_to_float", grc.blocks[1].id);
    println!("{grc:?}");
    assert_eq!(2, grc.connections.len());
}

#[test]
pub fn parse_clipdetect_f() {
    let cmds = "clipdetect_ff";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("clipdetect_ff", grc.blocks[1].id);
    println!("{grc:?}");
    assert_eq!(2, grc.connections.len());
}

#[test]
pub fn parse_dump_f() {
    let cmds = "dump_f";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(2, grc.blocks.len());
    assert_eq!("dump_f", grc.blocks[1].id);
    println!("{grc:?}");
    assert_eq!(1, grc.connections.len());
}

#[test]
pub fn parse_dump_u8() {
    let cmds = "dump_u8";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(2, grc.blocks.len());
    assert_eq!("dump_u8", grc.blocks[1].id);
    println!("{grc:?}");
    assert_eq!(1, grc.connections.len());
}

/// Regression test for the bug where the "float" and "u8" match arms in
/// `src/grc/converter/dump.rs` were swapped, causing a
/// "Validation error dyn BufferReader has wrong type" error when connecting
/// the dump_u8 block downstream of any block producing u8 output.
#[test]
pub fn convert_dump_u8_connects_to_u8_source() -> Result<()> {
    let cmds = "dump_u8";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();

    let mut fg = Flowgraph::new();
    let orig: Vec<u8> = vec![0x00, 0x01, 0x02, 0x03, 0xFF];
    let src = fg.add_block(VectorSource::<u8>::new(orig));

    let block_under_test = Grc2FutureSdr::new().convert_block(&mut fg, &grc.blocks[1])?;
    let (but_in, in_name) = block_under_test.adapt_input_port("in")?;

    // Before the fix this line failed with:
    // "Validation error dyn BufferReader has wrong type"
    fg.connect_dyn(&src, "output", but_in, in_name)?;

    Runtime::new().run(fg)?;
    Ok(())
}

#[test]
pub fn convert_dump_f_connects_to_f32_source() -> Result<()> {
    let cmds = "dump_f";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();

    let mut fg = Flowgraph::new();
    let orig: Vec<f32> = vec![0.0, 1.0, -1.0, 0.5];
    let src = fg.add_block(VectorSource::<f32>::new(orig));

    let block_under_test = Grc2FutureSdr::new().convert_block(&mut fg, &grc.blocks[1])?;
    let (but_in, in_name) = block_under_test.adapt_input_port("in")?;

    fg.connect_dyn(&src, "output", but_in, in_name)?;

    Runtime::new().run(fg)?;
    Ok(())
}

#[test]
pub fn parse_realpart_cf() {
    let cmds = "realpart_cf";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("blocks_complex_to_real", grc.blocks[1].id);
    println!("{grc:?}");
    assert_eq!(2, grc.connections.len());
}

#[test]
pub fn parse_throttle_ff() {
    let cmds = "throttle_ff (48000*6.0)";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("blocks_throttle", grc.blocks[1].id);
    println!("{grc:?}");
    assert_eq!(2, grc.connections.len());
}

#[test]
pub fn parse_octave_complex_c() {
    let cmds = "octave_complex_c 512 1024";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(2, grc.blocks.len());
    assert_eq!("octave_complex_c", grc.blocks[1].id);
    println!("{grc:?}");
    assert_eq!(1, grc.connections.len());
}

#[test]
pub fn parse_multiple_commands_retrocompatibility() {
    let cmds = "csdr convert_u8_f | csdr fmdemod_quadri_cf | csdr fractional_decimator_ff 5 | csdr deemphasis_wfm_ff 48000 50e-6 | csdr convert_f_s16";
    let result = CsdrParser::parse_multiple_commands(cmds);
    let grc = result.expect("").unwrap();
    println!("{grc:?}");
    assert_eq!(8, grc.blocks.len());
    assert_eq!(7, grc.connections.len());
}

#[test]
pub fn parse_multiple_commands() {
    let cmds = "csdr convert_u8_f | fmdemod_quadri_cf | fractional_decimator_ff 5 | deemphasis_wfm_ff 48000 50e-6 | convert_f_s16";
    let result = CsdrParser::parse_multiple_commands(cmds);
    let grc = result.expect("").unwrap();
    println!("{grc:?}");
    assert_eq!(8, grc.blocks.len());
    assert_eq!(7, grc.connections.len());
}

#[test]
pub fn parse_shift_addition_cc_1256() {
    let cmds = "shift_addition_cc 1256";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("blocks_freqshift_cc", grc.blocks[1].id);
    println!("{grc:?}");
    assert_eq!(2, grc.connections.len());
}

#[test]
pub fn parse_limit_ff() {
    let cmds = "limit_ff";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_rail_ff", grc.blocks[1].id);
    let low_threshold = grc.blocks[1]
        .parameters
        .get("lo")
        .expect("low threshold must be defined");
    assert_eq!("-1.0*(1.0)", low_threshold);
    let high_threshold = grc.blocks[1]
        .parameters
        .get("hi")
        .expect("high threshold must be defined");
    assert_eq!("1.0", high_threshold);
    assert_eq!(2, grc.connections.len());
}

#[test]
pub fn parse_limit_ff_with_max_amplitude() -> Result<()> {
    let cmds = "limit_ff 3.0";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    //println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_rail_ff", grc.blocks[1].id);
    let low_threshold = grc.blocks[1]
        .parameters
        .get("lo")
        .expect("low threshold must be defined");
    assert_eq!("-1.0*(3.0)", low_threshold);
    let high_threshold = grc.blocks[1]
        .parameters
        .get("hi")
        .expect("high threshold must be defined");
    assert_eq!("3.0", high_threshold);
    assert_eq!(2, grc.connections.len());

    let mut fg = Flowgraph::new();
    let orig: Vec<f32> = vec![
        -10.0, -5.0, -3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 10.0,
    ];
    let src = fg.add_block(VectorSource::<f32>::new(orig));
    let vect_sink_0 = fg.add_block(VectorSink::<f32>::new(1024));

    let block_under_test = Grc2FutureSdr::new().convert_block(&mut fg, &grc.blocks[1])?;
    let (but_in, in_name) = block_under_test.adapt_input_port("in")?;
    let (but_out, out_name) = block_under_test.adapt_output_port("out")?;

    fg.connect_dyn(&src, "output", but_in, in_name)?;
    fg.connect_dyn(but_out, out_name, &vect_sink_0, "input")?;

    Runtime::new().run(fg)?;

    let snk_0 = vect_sink_0.get().unwrap();
    let snk_0 = snk_0.items();
    // assert!(snk_0.iter().all(|v| -3.0 <= *v && *v <= 3.0));
    let expected: Vec<f32> = vec![
        -3.0, -3.0, -3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0, 3.0,
    ];
    for (i, (expected, actual)) in expected.iter().zip(snk_0.iter()).enumerate() {
        assert_eq!(
            expected, actual,
            "at index {i}, expected: {expected}, got: {actual}"
        );
    }
    Ok(())
}

#[test]
pub fn parse_limit_ff_with_max_amplitude_expr() -> Result<()> {
    let cmds = "limit_ff (6.0/2)";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    //println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_rail_ff", grc.blocks[1].id);
    let low_threshold = grc.blocks[1]
        .parameters
        .get("lo")
        .expect("low threshold must be defined");
    assert_eq!("-1.0*(6.0/2)", low_threshold);
    let high_threshold = grc.blocks[1]
        .parameters
        .get("hi")
        .expect("high threshold must be defined");
    assert_eq!("6.0/2", high_threshold);
    assert_eq!(2, grc.connections.len());

    let mut fg = Flowgraph::new();
    let orig: Vec<f32> = vec![
        -10.0, -5.0, -3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 10.0,
    ];
    let src = fg.add_block(VectorSource::<f32>::new(orig));
    let vect_sink_0 = fg.add_block(VectorSink::<f32>::new(1024));

    let block_under_test = Grc2FutureSdr::new().convert_block(&mut fg, &grc.blocks[1])?;
    let (but_in, in_name) = block_under_test.adapt_input_port("in")?;
    let (but_out, out_name) = block_under_test.adapt_output_port("out")?;

    fg.connect_dyn(&src, "output", but_in, in_name)?;
    fg.connect_dyn(but_out, out_name, &vect_sink_0, "input")?;

    Runtime::new().run(fg)?;

    let snk_0 = vect_sink_0.get().unwrap();
    let snk_0 = snk_0.items();
    // assert!(snk_0.iter().all(|v| -3.0 <= *v && *v <= 3.0));
    let expected: Vec<f32> = vec![
        -3.0, -3.0, -3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0, 3.0,
    ];
    for (i, (expected, actual)) in expected.iter().zip(snk_0.iter()).enumerate() {
        assert_eq!(
            expected, actual,
            "at index {i}, expected: {expected}, got: {actual}"
        );
    }
    Ok(())
}

#[test]
pub fn parse_limit_ff_multiple_commands() {
    let cmds = "csdr convert_u8_f | limit_ff | limit_ff 16.0 | convert_f_s16";
    let result = CsdrParser::parse_multiple_commands(cmds);
    let grc = result.expect("").unwrap();
    // println!("{grc:?}");
    assert_eq!(4 + 2, grc.blocks.len());

    let first_blk = &grc.blocks[2];
    assert_eq!("analog_rail_ff", first_blk.id);
    let low_threshold = first_blk
        .parameters
        .get("lo")
        .expect("low threshold must be defined");
    assert_eq!("-1.0*(1.0)", low_threshold);
    let high_threshold = first_blk
        .parameters
        .get("hi")
        .expect("high threshold must be defined");
    assert_eq!("1.0", high_threshold);

    let second_blk = &grc.blocks[3];
    assert_eq!("analog_rail_ff", second_blk.id);
    let high_threshold = second_blk
        .parameters
        .get("hi")
        .expect("high threshold must be defined");
    assert_eq!("16.0", high_threshold);
    let low_threshold = second_blk
        .parameters
        .get("lo")
        .expect("low threshold must be defined");
    assert_eq!("-1.0*(16.0)", low_threshold);
}

#[test]
pub fn parse_fastdc_block() -> Result<()> {
    let cmds = "fastdcblock_ff";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    // println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("dc_blocker_xx", grc.blocks[1].id);
    let dc_length = grc.blocks[1]
        .parameters
        .get("length")
        .expect("length must be defined");
    assert_eq!("32", dc_length);
    let long_form = grc.blocks[1]
        .parameters
        .get("long_form")
        .expect("long_form must be defined");
    assert_eq!("False", long_form);
    let data_type = grc.blocks[1]
        .parameters
        .get("type")
        .expect("type must be defined");
    assert_eq!("ff", data_type);
    assert_eq!(2, grc.connections.len());

    let mut fg = Flowgraph::new();
    let orig: Vec<f32> = vec![0.0, -1.0, -2.0, -1.0, 0.0, 1.0, 2.0, 1.0];
    let orig: Vec<f32> = orig.iter().map(|x| x + 5.0).collect();
    let orig = orig.repeat(32);
    let src = fg.add_block(VectorSource::<f32>::new(orig));
    let vect_sink_0 = fg.add_block(VectorSink::<f32>::new(1024));

    let block_under_test = Grc2FutureSdr::new().convert_block(&mut fg, &grc.blocks[1])?;
    let (but_in, in_name) = block_under_test.adapt_input_port("in")?;
    let (but_out, out_name) = block_under_test.adapt_output_port("out")?;

    fg.connect_dyn(&src, "output", but_in, in_name)?;
    fg.connect_dyn(but_out, out_name, &vect_sink_0, "input")?;

    Runtime::new().run(fg)?;

    let snk_0 = vect_sink_0.get().unwrap();
    let snk_0 = snk_0.items();
    // println!("{snk_0:?}");
    assert!(snk_0.iter().skip(110).all(|v| -5.0 <= *v && *v <= 5.0));
    assert!(snk_0.iter().skip(175).all(|v| -4.0 <= *v && *v <= 4.0));
    assert!(snk_0.iter().skip(200).all(|v| -3.0 <= *v && *v <= 3.0));
    Ok(())
}

#[test]
pub fn parse_amdemod_cf() -> Result<()> {
    let cmds = "amdemod_cf";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    // println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("blocks_complex_to_mag", grc.blocks[1].id);
    assert_eq!(2, grc.connections.len());

    let mut fg = Flowgraph::new();
    let orig: Vec<Complex32> = (0..10)
        .map(|x| x as f32)
        .map(|x| Complex32::new(x * x, 0.0))
        .collect();
    let src = fg.add_block(VectorSource::<Complex32>::new(orig));
    let vect_sink_0 = fg.add_block(VectorSink::<f32>::new(1024));

    let block_under_test = Grc2FutureSdr::new().convert_block(&mut fg, &grc.blocks[1])?;
    let (but_in, in_name) = block_under_test.adapt_input_port("in")?;
    let (but_out, out_name) = block_under_test.adapt_output_port("out")?;

    fg.connect_dyn(&src, "output", but_in, in_name)?;
    fg.connect_dyn(but_out, out_name, &vect_sink_0, "input")?;

    Runtime::new().run(fg)?;

    let snk_0 = vect_sink_0.get().unwrap();
    let snk_0 = snk_0.items();
    // println!("{snk_0:?}");
    assert!(snk_0
        .iter()
        .enumerate()
        .all(|(i, v)| ((i * i) as f32 - *v).abs() < 0.0001));
    Ok(())
}

#[test]
pub fn parse_agc_ff() -> Result<()> {
    let cmds = "agc_ff";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    // println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_agc_xx", grc.blocks[1].id);
    let reference = grc.blocks[1]
        .parameters
        .get("reference")
        .expect("reference must be defined");
    assert_eq!("0.8", reference);
    let max_gain = grc.blocks[1]
        .parameters
        .get("max_gain")
        .expect("max_gain must be defined");
    assert_eq!("65536.0", max_gain);
    let rate = grc.blocks[1]
        .parameters
        .get("rate")
        .expect("rate must be defined");
    assert_eq!("0.0001", rate);
    let data_type = grc.blocks[1]
        .parameters
        .get("type")
        .expect("type must be defined");
    assert_eq!("float", data_type);
    assert_eq!(2, grc.connections.len());

    let mut fg = Flowgraph::new();
    let orig: Vec<f32> = (0..256).map(|x| (x as f32).sin() * 0.5).collect();
    let orig = orig.repeat(256);
    let src = fg.add_block(VectorSource::<f32>::new(orig));
    let vect_sink_0 = fg.add_block(VectorSink::<f32>::new(1024));

    let block_under_test = Grc2FutureSdr::new().convert_block(&mut fg, &grc.blocks[1])?;
    let (but_in, in_name) = block_under_test.adapt_input_port("in")?;
    let (but_out, out_name) = block_under_test.adapt_output_port("out")?;

    fg.connect_dyn(&src, "output", but_in, in_name)?;
    fg.connect_dyn(but_out, out_name, &vect_sink_0, "input")?;

    Runtime::new().run(fg)?;

    let snk_0 = vect_sink_0.get().unwrap();
    let snk_0 = snk_0.items();
    // println!("{snk_0:?}");
    assert!(snk_0.iter().any(|v| *v > 0.7));
    Ok(())
}

#[test]
pub fn parse_agc_ff_with_reference() -> Result<()> {
    let cmds = "agc_ff --reference 1.0";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    // println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_agc_xx", grc.blocks[1].id);
    let reference = grc.blocks[1]
        .parameters
        .get("reference")
        .expect("reference must be defined");
    assert_eq!("1.0", reference);
    let max_gain = grc.blocks[1]
        .parameters
        .get("max_gain")
        .expect("max_gain must be defined");
    assert_eq!("65536.0", max_gain);
    let rate = grc.blocks[1]
        .parameters
        .get("rate")
        .expect("rate must be defined");
    assert_eq!("0.0001", rate);
    let data_type = grc.blocks[1]
        .parameters
        .get("type")
        .expect("type must be defined");
    assert_eq!("float", data_type);
    Ok(())
}

#[test]
pub fn parse_agc_ff_with_reference_and_max_gain() -> Result<()> {
    let cmds = "agc_ff --reference 0.9 --max 256.0";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    // println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_agc_xx", grc.blocks[1].id);
    let reference = grc.blocks[1]
        .parameters
        .get("reference")
        .expect("reference must be defined");
    assert_eq!("0.9", reference);
    let max_gain = grc.blocks[1]
        .parameters
        .get("max_gain")
        .expect("max_gain must be defined");
    assert_eq!("256.0", max_gain);
    let rate = grc.blocks[1]
        .parameters
        .get("rate")
        .expect("rate must be defined");
    assert_eq!("0.0001", rate);
    let data_type = grc.blocks[1]
        .parameters
        .get("type")
        .expect("type must be defined");
    assert_eq!("float", data_type);
    Ok(())
}

#[test]
pub fn parse_agc_ff_with_rate() -> Result<()> {
    let cmds = "agc_ff --rate 0.0002";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    // println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_agc_xx", grc.blocks[1].id);
    let reference = grc.blocks[1]
        .parameters
        .get("reference")
        .expect("reference must be defined");
    assert_eq!("0.8", reference);
    let max_gain = grc.blocks[1]
        .parameters
        .get("max_gain")
        .expect("max_gain must be defined");
    assert_eq!("65536.0", max_gain);
    let rate = grc.blocks[1]
        .parameters
        .get("rate")
        .expect("rate must be defined");
    assert_eq!("0.0002", rate);
    let data_type = grc.blocks[1]
        .parameters
        .get("type")
        .expect("type must be defined");
    assert_eq!("float", data_type);
    Ok(())
}

#[test]
pub fn parse_fir_decimate_cc() -> Result<()> {
    let cmds = "csdr fir_decimate_cc 50";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    // println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("fir_filter_xxx", grc.blocks[1].id);
    let decimation_factor = grc.blocks[1]
        .parameters
        .get("decim")
        .expect("decimation_factor must be defined");
    assert_eq!("50", decimation_factor);
    let transition_bw = grc.blocks[1]
        .parameters
        .get("transition_bw")
        .expect("transition_bw  must be defined");
    assert_eq!("0.05", transition_bw);
    let window = grc.blocks[1]
        .parameters
        .get("window")
        .expect("window must be defined");
    assert_eq!("HAMMING", window);
    let data_type = grc.blocks[1]
        .parameters
        .get("type")
        .expect("type must be defined");
    assert_eq!("ccc", data_type);

    let mut fg = Flowgraph::new();
    let orig: Vec<Complex32> = (0..350)
        .map(|x| Complex32::new((x as f32).cos() * 0.5, (x as f32).sin() * 0.5))
        .collect();
    let orig = orig.repeat(10);
    let original_length = orig.len();
    let src = fg.add_block(VectorSource::<Complex32>::new(orig));
    let vect_sink_0 = fg.add_block(VectorSink::<Complex32>::new(1024));

    let block_under_test = Grc2FutureSdr::new().convert_block(&mut fg, &grc.blocks[1])?;
    let (but_in, in_name) = block_under_test.adapt_input_port("in")?;
    let (but_out, out_name) = block_under_test.adapt_output_port("out")?;

    fg.connect_dyn(&src, "output", but_in, in_name)?;
    fg.connect_dyn(but_out, out_name, &vect_sink_0, "input")?;

    Runtime::new().run(fg)?;

    let snk_0 = vect_sink_0.get().unwrap();
    let snk_0 = snk_0.items();
    // println!("{snk_0:?}");
    assert_eq!(original_length / 50 - 2, snk_0.len());

    Ok(())
}

#[test]
pub fn parse_fir_decimate_cc_bw() -> Result<()> {
    let cmds = "fir_decimate_cc 50 0.06";
    let result = CsdrParser::parse_command(cmds);
    let grc = result?.unwrap();
    // println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("fir_filter_xxx", grc.blocks[1].id);
    let decimation_factor = grc.blocks[1]
        .parameters
        .get("decim")
        .expect("decimation_factor must be defined");
    assert_eq!("50", decimation_factor);
    let transition_bw = grc.blocks[1]
        .parameters
        .get("transition_bw")
        .expect("transition_bw  must be defined");
    assert_eq!("0.06", transition_bw);
    let window = grc.blocks[1]
        .parameters
        .get("window")
        .expect("window must be defined");
    assert_eq!("HAMMING", window);
    let data_type = grc.blocks[1]
        .parameters
        .get("type")
        .expect("type must be defined");
    assert_eq!("ccc", data_type);
    Ok(())
}

#[test]
pub fn parse_fir_decimate_cc_bw_windows() -> Result<()> {
    let cmds = "fir_decimate_cc 50 0.06 BLACKMAN";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    // println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("fir_filter_xxx", grc.blocks[1].id);
    let decimation_factor = grc.blocks[1]
        .parameters
        .get("decim")
        .expect("decimation_factor must be defined");
    assert_eq!("50", decimation_factor);
    let transition_bw = grc.blocks[1]
        .parameters
        .get("transition_bw")
        .expect("transition_bw  must be defined");
    assert_eq!("0.06", transition_bw);
    let window = grc.blocks[1]
        .parameters
        .get("window")
        .expect("window must be defined");
    assert_eq!("BLACKMAN", window);
    let data_type = grc.blocks[1]
        .parameters
        .get("type")
        .expect("type must be defined");
    assert_eq!("ccc", data_type);
    Ok(())
}

#[test]
pub fn parse_deemphasis_nfm_ff() -> Result<()> {
    let cmds = "deemphasis_nfm_ff 48000";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    // println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_nfm_deemph", grc.blocks[1].id);
    let sample_rate = grc.blocks[1]
        .parameters
        .get("samp_rate")
        .expect("samp_rate must be defined");
    assert_eq!("48000", sample_rate);

    let mut fg = Flowgraph::new();
    let orig: Vec<f32> = (0..360).map(|x| (x as f32).cos() * 0.5).collect();
    let orig = orig.repeat(10);
    let src = fg.add_block(VectorSource::<f32>::new(orig));
    let vect_sink_0 = fg.add_block(VectorSink::<f32>::new(1024));

    let block_under_test = Grc2FutureSdr::new().convert_block(&mut fg, &grc.blocks[1])?;
    let (but_in, in_name) = block_under_test.adapt_input_port("in")?;
    let (but_out, out_name) = block_under_test.adapt_output_port("out")?;

    fg.connect_dyn(&src, "output", but_in, in_name)?;
    fg.connect_dyn(but_out, out_name, &vect_sink_0, "input")?;

    Runtime::new().run(fg)?;

    let snk_0 = vect_sink_0.get().unwrap();
    let _snk_0 = snk_0.items();
    // println!("{snk_0:?}");
    Ok(())
}

#[test]
pub fn parse_weaver_lsb_123456() {
    let cmds = "weaver_lsb_cf 123456";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("weaver_lsb_cf", grc.blocks[1].id);
    let audio_freq = grc.blocks[1].parameter_or("audio_freq", "none");
    assert_ne!("123456", audio_freq);
    println!("{grc:?}");
    assert_eq!(2, grc.connections.len());
}

#[test]
pub fn parse_weaver_usb_123456() {
    let cmds = "weaver_usb_cf 123456";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("weaver_usb_cf", grc.blocks[1].id);
    let audio_freq = grc.blocks[1].parameter_or("audio_freq", "none");
    assert_ne!("123456", audio_freq);
    println!("{grc:?}");
    assert_eq!(2, grc.connections.len());
}

#[test]
pub fn parse_rational_resampler_ff() {
    let cmds = "rational_resampler_ff 123 456";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("rational_resampler_xxx", grc.blocks[1].id);
    let interp = grc.blocks[1].parameter_or("interp", "none");
    assert_eq!("123", interp);
    let decim = grc.blocks[1].parameter_or("decim", "none");
    assert_eq!("456", decim);
    let kind = grc.blocks[1].parameter_or("type", "none");
    assert_eq!("fff", kind);
    // println!("{grc:?}");
    assert_eq!(2, grc.connections.len());
}

#[test]
pub fn parse_rational_resampler_cc() {
    let cmds = "rational_resampler_cc 123 456";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("rational_resampler_xxx", grc.blocks[1].id);
    let interp = grc.blocks[1].parameter_or("interp", "none");
    assert_eq!("123", interp);
    let decim = grc.blocks[1].parameter_or("decim", "none");
    assert_eq!("456", decim);
    let kind = grc.blocks[1].parameter_or("type", "none");
    assert_eq!("ccc", kind);
    // println!("{grc:?}");
    assert_eq!(2, grc.connections.len());
}

#[test]
pub fn parse_fixedlen_to_pdu() {
    let cmds = "fixedlen_to_pdu 240";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(2, grc.blocks.len());
    assert_eq!("satellites_fixedlen_to_pdu", grc.blocks[1].id);
    assert_eq!("240", grc.blocks[1].parameter_or("packet_len", "none"));
    assert_eq!("", grc.blocks[1].parameter_or("syncword_tag", "none"));
    assert_eq!("False", grc.blocks[1].parameter_or("pack", "none"));
    assert_eq!(
        "\"\"",
        grc.blocks[1].parameter_or("packet_len_tag_key", "none")
    );
    assert_eq!("byte", grc.blocks[1].parameter_or("type", "none"));
    assert_eq!(1, grc.connections.len());
}

#[test]
pub fn parse_fixedlen_to_pdu_with_tag() {
    let cmds = "fixedlen_to_pdu (100+20) sync";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(2, grc.blocks.len());
    assert_eq!("satellites_fixedlen_to_pdu", grc.blocks[1].id);
    assert_eq!("120", grc.blocks[1].parameter_or("packet_len", "none"));
    assert_eq!("sync", grc.blocks[1].parameter_or("syncword_tag", "none"));
}

#[test]
pub fn parse_save_kiss_chain() {
    let mut input_file = std::env::temp_dir();
    input_file.push("test_input.kiss");
    std::fs::write(&input_file, vec![0xC0, 0x00, 0xC0]).expect("write dummy kiss");

    let mut temp_file = std::env::temp_dir();
    temp_file.push("test.kiss");
    let temp_file_str = temp_file.to_str().expect("valid temp path");

    let cmds = format!(
        "csdr load_kiss {} ! save_kiss {}",
        input_file.display(),
        temp_file_str
    );
    let result = CsdrParser::parse_multiple_commands(&cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(2, grc.blocks.len());
    assert_eq!("satellites_kiss_file_source", grc.blocks[0].id);
    assert_eq!("satellites_kiss_file_sink", grc.blocks[1].id);

    // Verify it can be converted to a flowgraph
    let mut g2f = Grc2FutureSdr::new();
    let fg = g2f.convert_grc(grc);
    assert!(
        fg.is_ok(),
        "Failed to convert GRC to Flowgraph for save_kiss chain: {:?}",
        fg.err()
    );

    let _ = std::fs::remove_file(input_file);
    let _ = std::fs::remove_file(temp_file);
}

#[test]
pub fn parse_fixedlen_to_pdu_chain() {
    let mut input_file = std::env::temp_dir();
    input_file.push("test_input_u8.bin");
    std::fs::write(&input_file, vec![0u8; 1024]).expect("write dummy u8");

    let mut temp_file = std::env::temp_dir();
    temp_file.push("test2.kiss");
    let temp_file_str = temp_file.to_str().expect("valid temp path");

    let cmds = format!(
        "csdr load_u8 {} ! fixedlen_to_pdu 240 ! save_kiss {}",
        input_file.display(),
        temp_file_str
    );
    let result = CsdrParser::parse_multiple_commands(&cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("blocks_file_source", grc.blocks[0].id);
    assert_eq!("satellites_fixedlen_to_pdu", grc.blocks[1].id);
    assert_eq!("240", grc.blocks[1].parameter_or("packet_len", "none"));
    assert_eq!("byte", grc.blocks[1].parameter_or("type", "none"));
    assert_eq!("satellites_kiss_file_sink", grc.blocks[2].id);

    // Verify connections use port "0" (as GrcBuilder currently does)
    assert_eq!(2, grc.connections.len());
    assert_eq!("0", grc.connections[1][1]); // Output port of fixedlen_to_pdu

    // Verify it can be converted to a flowgraph
    let mut g2f = Grc2FutureSdr::new();
    let fg = g2f.convert_grc(grc);
    assert!(
        fg.is_ok(),
        "Failed to convert GRC to Flowgraph: {:?}",
        fg.err()
    );

    let _ = std::fs::remove_file(input_file);
    let _ = std::fs::remove_file(temp_file);
}

#[test]
pub fn repro_hang_user_command() -> Result<()> {
    use std::env;
    use std::fs::File;
    use std::io::Write;

    let mut input_path = env::temp_dir();
    input_path.push("some_file.sigmf-data");
    let mut output_path = env::temp_dir();
    output_path.push("some_file.kiss");

    {
        let mut f = File::create(&input_path)?;
        // Write some dummy data. 1M of zeros (floats)
        // This is 1,000,000 / 2 = 500,000 complexes.
        let data = vec![0.0f32; 1000 * 1024];
        let bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                data.len() * std::mem::size_of::<f32>(),
            )
        };
        f.write_all(bytes)?;
    }

    let cmd = format!(
        "csdr load_f {} ! shift_addition_cc ((435166900-435200000)/2400000) ! fmdemod_quadri_cf ! rational_resampler_ff 1 50 ! dsb_fc ! timing_recovery_cc GARDNER 20 0.5 2 ! realpart_cf ! binary_slicer_f_u8 ! pattern_search_u8_u8 (8*240) 1 0 1 1 1 0 1 1 1 1 1 1 0 0 1 0 0 1 1 0 0 0 0 0 1 0 0 1 1 1 ! pack_bits_8to1_u8_u8 ! fixedlen_to_pdu 240 ! save_kiss {}",
        input_path.display(),
        output_path.display()
    );

    let grc = CsdrParser::parse_multiple_commands(cmd.as_str())?.expect("Failed to parse command");
    let mut g2f = Grc2FutureSdr::new();
    let fg = g2f.convert_grc(grc)?;

    // This test ensures that the flowgraph completes without hanging.
    // Hangs often occurred in debug mode due to missing `io.call_again = true`
    // in some blocks when they produced or consumed data but didn't finish.
    Runtime::new().run(fg)?;

    // Cleanup
    let _ = std::fs::remove_file(input_path);
    let _ = std::fs::remove_file(output_path);

    Ok(())
}

#[test]
pub fn parse_tcp_kiss_server() {
    let cmds = "csdr load_kiss tests/test.kiss ! tcp_kiss_server 127.0.0.1:8045";
    let result = CsdrParser::parse_multiple_commands(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(2, grc.blocks.len());
    assert_eq!("satellites_kiss_server_sink", grc.blocks[1].id);
    assert_eq!("\"127.0.0.1\"", grc.blocks[1].parameters["address"]);
    assert_eq!("8045", grc.blocks[1].parameters["port"]);
}

#[test]
pub fn parse_tcp_kiss_client() {
    let cmds = "tcp_kiss_client myhost.com:8045";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(1, grc.blocks.len());
    assert_eq!("satellites_kiss_client_source", grc.blocks[0].id);
    assert_eq!("\"myhost.com\"", grc.blocks[0].parameters["address"]);
    assert_eq!("8045", grc.blocks[0].parameters["port"]);
}
