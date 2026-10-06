use anyhow::Result;
use fsdr_cli::csdr_cmd::CsdrParser;
use fsdr_cli::grc::converter::Grc2FutureSdr;
use futuresdr::blocks::ApplyNM;
use futuresdr::blocks::VectorSink;
use futuresdr::blocks::VectorSource;
use futuresdr::num_complex::Complex32;
use futuresdr::prelude::connect;
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
    let src = fg.add(VectorSource::<u8>::new(orig))?;

    let block_under_test = Grc2FutureSdr::new().convert_block(&mut fg, &grc.blocks[1])?;
    let (but_in, in_name) = block_under_test.adapt_input_port("in")?;

    // Before the fix this line failed with:
    // "Validation error dyn BufferReader has wrong type"
    fg.stream_dyn(src.id(), "output", but_in, in_name)?;

    let _fg = Runtime::new().run(fg)?;
    Ok(())
}

#[test]
pub fn convert_dump_f_connects_to_f32_source() -> Result<()> {
    let cmds = "dump_f";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();

    let mut fg = Flowgraph::new();
    let orig: Vec<f32> = vec![0.0, 1.0, -1.0, 0.5];
    let src = fg.add(VectorSource::<f32>::new(orig))?;

    let block_under_test = Grc2FutureSdr::new().convert_block(&mut fg, &grc.blocks[1])?;
    let (but_in, in_name) = block_under_test.adapt_input_port("in")?;

    fg.stream_dyn(src.id(), "output", but_in, in_name)?;

    let _fg = Runtime::new().run(fg)?;
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
pub fn parse_afc_ff() {
    let cmds = "afc_ff --alpha 0.01 --limit 1.5";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_afc_ff", grc.blocks[1].id);
    assert_eq!("0.01", grc.blocks[1].parameters.get("alpha").unwrap());
    assert_eq!("1.5", grc.blocks[1].parameters.get("limit").unwrap());
}

#[test]
pub fn parse_afc_cc() {
    let cmds = "afc_cc --alpha 0.005 --max-freq 5000 --samp-rate 48000";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_afc_cc", grc.blocks[1].id);
    assert_eq!("0.005", grc.blocks[1].parameters.get("alpha").unwrap());
    assert_eq!("5000", grc.blocks[1].parameters.get("max_freq").unwrap());
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
    let src = fg.add(VectorSource::<f32>::new(orig))?;
    let vect_sink_0 = fg.add(VectorSink::<f32>::new(1024))?;

    let block_under_test = Grc2FutureSdr::new().convert_block(&mut fg, &grc.blocks[1])?;
    let (but_in, in_name) = block_under_test.adapt_input_port("in")?;
    let (but_out, out_name) = block_under_test.adapt_output_port("out")?;

    fg.stream_dyn(src.id(), "output", but_in, in_name)?;
    fg.stream_dyn(but_out, out_name, vect_sink_0.id(), "input")?;

    let fg = Runtime::new().run(fg)?;

    let snk_0 = fg.block(&vect_sink_0)?;
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
    let src = fg.add(VectorSource::<f32>::new(orig))?;
    let vect_sink_0 = fg.add(VectorSink::<f32>::new(1024))?;

    let block_under_test = Grc2FutureSdr::new().convert_block(&mut fg, &grc.blocks[1])?;
    let (but_in, in_name) = block_under_test.adapt_input_port("in")?;
    let (but_out, out_name) = block_under_test.adapt_output_port("out")?;

    fg.stream_dyn(src.id(), "output", but_in, in_name)?;
    fg.stream_dyn(but_out, out_name, vect_sink_0.id(), "input")?;

    let fg = Runtime::new().run(fg)?;

    let snk_0 = fg.block(&vect_sink_0)?;
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
    let src = fg.add(VectorSource::<f32>::new(orig))?;
    let vect_sink_0 = fg.add(VectorSink::<f32>::new(1024))?;

    let block_under_test = Grc2FutureSdr::new().convert_block(&mut fg, &grc.blocks[1])?;
    let (but_in, in_name) = block_under_test.adapt_input_port("in")?;
    let (but_out, out_name) = block_under_test.adapt_output_port("out")?;

    fg.stream_dyn(src.id(), "output", but_in, in_name)?;
    fg.stream_dyn(but_out, out_name, vect_sink_0.id(), "input")?;

    let fg = Runtime::new().run(fg)?;

    let snk_0 = fg.block(&vect_sink_0)?;
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
    let src = fg.add(VectorSource::<Complex32>::new(orig))?;
    let vect_sink_0 = fg.add(VectorSink::<f32>::new(1024))?;

    let block_under_test = Grc2FutureSdr::new().convert_block(&mut fg, &grc.blocks[1])?;
    let (but_in, in_name) = block_under_test.adapt_input_port("in")?;
    let (but_out, out_name) = block_under_test.adapt_output_port("out")?;

    fg.stream_dyn(src.id(), "output", but_in, in_name)?;
    fg.stream_dyn(but_out, out_name, vect_sink_0.id(), "input")?;

    let fg = Runtime::new().run(fg)?;

    let snk_0 = fg.block(&vect_sink_0)?;
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
    let src = fg.add(VectorSource::<f32>::new(orig))?;
    let vect_sink_0 = fg.add(VectorSink::<f32>::new(1024))?;

    let block_under_test = Grc2FutureSdr::new().convert_block(&mut fg, &grc.blocks[1])?;
    let (but_in, in_name) = block_under_test.adapt_input_port("in")?;
    let (but_out, out_name) = block_under_test.adapt_output_port("out")?;

    fg.stream_dyn(src.id(), "output", but_in, in_name)?;
    fg.stream_dyn(but_out, out_name, vect_sink_0.id(), "input")?;

    let fg = Runtime::new().run(fg)?;

    let snk_0 = fg.block(&vect_sink_0)?;
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
    let src = fg.add(VectorSource::<Complex32>::new(orig))?;
    let vect_sink_0 = fg.add(VectorSink::<Complex32>::new(1024))?;

    let block_under_test = Grc2FutureSdr::new().convert_block(&mut fg, &grc.blocks[1])?;
    let (but_in, in_name) = block_under_test.adapt_input_port("in")?;
    let (but_out, out_name) = block_under_test.adapt_output_port("out")?;

    fg.stream_dyn(src.id(), "output", but_in, in_name)?;
    fg.stream_dyn(but_out, out_name, vect_sink_0.id(), "input")?;

    let fg = Runtime::new().run(fg)?;

    let snk_0 = fg.block(&vect_sink_0)?;
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
    let src = fg.add(VectorSource::<f32>::new(orig))?;
    let vect_sink_0 = fg.add(VectorSink::<f32>::new(1024))?;

    let block_under_test = Grc2FutureSdr::new().convert_block(&mut fg, &grc.blocks[1])?;
    let (but_in, in_name) = block_under_test.adapt_input_port("in")?;
    let (but_out, out_name) = block_under_test.adapt_output_port("out")?;

    fg.stream_dyn(src.id(), "output", but_in, in_name)?;
    fg.stream_dyn(but_out, out_name, vect_sink_0.id(), "input")?;

    let fg = Runtime::new().run(fg)?;

    let snk_0 = fg.block(&vect_sink_0)?;
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
    let _fg = Runtime::new().run(fg)?;

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

#[test]
pub fn parse_fmdemod_quadri_cf() {
    let cmds = "fmdemod_quadri_cf";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_quadrature_demod_cf", grc.blocks[1].id);
    assert_eq!("1.0", grc.blocks[1].parameters["gain"]);
}

#[test]
pub fn parse_fmdemod_atan_cf() {
    let cmds = "fmdemod_atan_cf";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_quadrature_demod_cf", grc.blocks[1].id);
    assert_eq!("atan", grc.blocks[1].parameters["algorithm"]);
}

#[test]
pub fn parse_fractional_decimator_ff() {
    let cmds = "fractional_decimator_ff 5.5";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("rational_resampler_xxx", grc.blocks[1].id);
    assert_eq!("5.5", grc.blocks[1].parameters["decim"]);
}

#[test]
pub fn parse_bandpass_fir_fft_cc() {
    let cmds = "bandpass_fir_fft_cc 0.1 0.2 0.05 HAMMING";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("band_pass_filter", grc.blocks[1].id);
    assert_eq!("0.1", grc.blocks[1].parameters["low_cutoff_freq"]);
    assert_eq!("0.2", grc.blocks[1].parameters["high_cutoff_freq"]);
    assert_eq!("0.05", grc.blocks[1].parameters["width"]);
    assert_eq!("window.WIN_HAMMING", grc.blocks[1].parameters["win"]);
}

#[test]
pub fn parse_dsb_fc() {
    let cmds = "dsb_fc";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("dsb", grc.blocks[1].id);
}

#[test]
pub fn parse_deemphasis_wfm_ff() {
    let cmds = "deemphasis_wfm_ff 48000 50e-6";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_fm_deemph", grc.blocks[1].id);
    assert_eq!("48000", grc.blocks[1].parameters["samp_rate"]);
    assert_eq!("50e-6", grc.blocks[1].parameters["tau"]);
}

#[test]
pub fn parse_binary_slicer_f_u8() {
    let cmds = "binary_slicer_f_u8";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("digital_binary_slicer_fb", grc.blocks[1].id);
}

#[test]
pub fn parse_gain_ff() {
    let cmds = "gain_ff 2.5";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("blocks_multiply_const_vxx", grc.blocks[1].id);
    assert_eq!("2.5", grc.blocks[1].parameters["const"]);
}

#[test]
pub fn parse_pack_bits_8to1_u8_u8() {
    let cmds = "pack_bits_8to1_u8_u8";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("blocks_pack_k_bits_bb", grc.blocks[1].id);
    assert_eq!("8", grc.blocks[1].parameters["k"]);
}

#[test]
pub fn parse_pattern_search_u8_u8() {
    let cmds = "pattern_search_u8_u8 240 1 0 1 1";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("pattern_search", grc.blocks[1].id);
    assert_eq!("240", grc.blocks[1].parameters["values_after"]);
    assert_eq!("1,0,1,1", grc.blocks[1].parameters["pattern_values"]);
}

#[test]
pub fn parse_timing_recovery_cc() {
    let cmds = "timing_recovery_cc GARDNER 20 0.5 2";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("timing_recovery", grc.blocks[1].id);
    assert_eq!("GARDNER", grc.blocks[1].parameters["algorithm"]);
    assert_eq!("20", grc.blocks[1].parameters["decimation"]);
    assert_eq!("0.5", grc.blocks[1].parameters["mu"]);
    assert_eq!("2", grc.blocks[1].parameters["max_error"]);
}

#[test]
pub fn parse_audio() {
    let cmds = "audio 48000 2";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(2, grc.blocks.len());
    assert_eq!("audio_sink", grc.blocks[1].id);
    assert_eq!("48000", grc.blocks[1].parameters["samp_rate"]);
    assert_eq!("2", grc.blocks[1].parameters["num_inputs"]);
}

#[test]
pub fn parse_load_f() {
    let cmds = "load_f input.bin";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(2, grc.blocks.len());
    assert_eq!("blocks_file_source", grc.blocks[0].id);
    assert_eq!("input.bin", grc.blocks[0].parameters["file"]);
    assert_eq!("float", grc.blocks[0].parameters["type"]);
}

#[test]
pub fn parse_load_c() {
    let cmds = "load_c input.c32";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(2, grc.blocks.len());
    assert_eq!("blocks_file_source", grc.blocks[0].id);
    assert_eq!("input.c32", grc.blocks[0].parameters["file"]);
    assert_eq!("complex", grc.blocks[0].parameters["type"]);
}

#[test]
pub fn parse_load_u8() {
    let cmds = "load_u8 input.u8";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(2, grc.blocks.len());
    assert_eq!("blocks_file_source", grc.blocks[0].id);
    assert_eq!("input.u8", grc.blocks[0].parameters["file"]);
    assert_eq!("byte", grc.blocks[0].parameters["type"]);
}

#[test]
pub fn parse_fft_cc() {
    let cmds = "fft_cc 512 1024 HAMMING";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("fft_block", grc.blocks[1].id);
    assert_eq!("512", grc.blocks[1].parameters["fft_size"]);
    assert_eq!("1024", grc.blocks[1].parameters["every_n_samples"]);
    assert_eq!("HAMMING", grc.blocks[1].parameters["window"]);
    assert_eq!("complex", grc.blocks[1].parameters["type"]);
}

#[test]
pub fn parse_fft_fc() {
    let cmds = "fft_fc 512 512 BLACKMAN";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("fft_block", grc.blocks[1].id);
    assert_eq!("512", grc.blocks[1].parameters["fft_size"]);
    assert_eq!("512", grc.blocks[1].parameters["every_n_samples"]);
    assert_eq!("BLACKMAN", grc.blocks[1].parameters["window"]);
    assert_eq!("float", grc.blocks[1].parameters["type"]);
}

#[test]
pub fn parse_logpower_cf() {
    let cmds = "logpower_cf 10";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("logpower_cf", grc.blocks[1].id);
    assert_eq!("10", grc.blocks[1].parameters["add_db"]);
}

#[test]
pub fn parse_logaveragepower_cf() {
    let cmds = "logaveragepower_cf 512 10 5.0";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("logaveragepower_cf", grc.blocks[1].id);
    assert_eq!("512", grc.blocks[1].parameters["fft_size"]);
    assert_eq!("10", grc.blocks[1].parameters["avg_number"]);
    assert_eq!("5.0", grc.blocks[1].parameters["add_db"]);
}

#[test]
pub fn parse_fft_exchange_sides_ff() {
    let cmds = "fft_exchange_sides_ff 512";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("fft_exchange_sides_ff", grc.blocks[1].id);
    assert_eq!("512", grc.blocks[1].parameters["fft_size"]);
}

#[test]
pub fn test_logpower_cf_execution() -> Result<()> {
    let mut fg = Flowgraph::new();
    let src = VectorSource::<Complex32>::new(vec![
        Complex32::new(1.0, 0.0),
        Complex32::new(0.0, 2.0),
        Complex32::new(3.0, 4.0),
    ]);
    let logp = fsdr_cli::blocks::LogPowerCf::new(0.0);
    let snk = VectorSink::<f32>::new(10);

    connect!(fg, src > logp > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    assert_eq!(out.len(), 3);
    // 10 * log10(1^2) + 0 = 0
    assert!((out[0] - 0.0).abs() < 1e-4);
    // 10 * log10(2^2) = 10 * log10(4) ~= 6.0206
    assert!((out[1] - (10.0 * 4.0f32.log10())).abs() < 1e-4);
    // 10 * log10(3^2 + 4^2) = 10 * log10(25) ~= 13.9794
    assert!((out[2] - (10.0 * 25.0f32.log10())).abs() < 1e-4);

    Ok(())
}

#[test]
pub fn test_fft_exchange_sides_ff_execution() -> Result<()> {
    let mut fg = Flowgraph::new();
    let src = VectorSource::<f32>::new(vec![1.0, 2.0, 3.0, 4.0]);
    let swap = fsdr_cli::blocks::FftExchangeSidesFf::new(4);
    let snk = VectorSink::<f32>::new(10);

    connect!(fg, src > swap > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    assert_eq!(out, &[3.0, 4.0, 1.0, 2.0]);

    Ok(())
}

#[test]
pub fn test_fft_cc_execution() -> Result<()> {
    let mut fg = Flowgraph::new();
    // 4-point impulse [1, 0, 0, 0] with BOXCAR window should yield [1, 1, 1, 1]
    let src = VectorSource::<Complex32>::new(vec![
        Complex32::new(1.0, 0.0),
        Complex32::new(0.0, 0.0),
        Complex32::new(0.0, 0.0),
        Complex32::new(0.0, 0.0),
    ]);
    let fft = fsdr_cli::blocks::FftCc::new(4, 4, "NONE");
    let snk = VectorSink::<Complex32>::new(10);

    connect!(fg, src > fft > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    assert_eq!(out.len(), 4);
    for sample in out {
        assert!((sample.re - 1.0).abs() < 1e-4);
        assert!(sample.im.abs() < 1e-4);
    }

    Ok(())
}

#[test]
pub fn test_logaveragepower_cf_execution() -> Result<()> {
    let mut fg = Flowgraph::new();
    let src = VectorSource::<Complex32>::new(vec![
        Complex32::new(1.0, 0.0),
        Complex32::new(0.0, 2.0),
        Complex32::new(3.0, 0.0),
        Complex32::new(0.0, 4.0),
    ]);
    let logavg = fsdr_cli::blocks::LogAveragePowerCf::new(2, 2, 0.0);
    let snk = VectorSink::<f32>::new(10);

    connect!(fg, src > logavg > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    assert_eq!(out.len(), 2);
    assert!((out[0] - (10.0 * 5.0f32.log10())).abs() < 1e-4);
    assert!((out[1] - 10.0).abs() < 1e-4);

    Ok(())
}

#[test]
pub fn parse_dcblock_ff() {
    let cmds = "dcblock_ff 0.995";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("dcblock_ff", grc.blocks[1].id);
    assert_eq!("0.995", grc.blocks[1].parameters["r"]);
}

#[test]
pub fn parse_decimating_shift_addition_cc() {
    let cmds = "decimating_shift_addition_cc 0.25 4";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("decimating_shift_addition_cc", grc.blocks[1].id);
    assert_eq!("0.25", grc.blocks[1].parameters["rate"]);
    assert_eq!("4", grc.blocks[1].parameters["decimation"]);
}

#[test]
pub fn parse_add_dcoffset_cc() {
    let cmds = "add_dcoffset_cc 1.5 0.5";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("add_dcoffset_cc", grc.blocks[1].id);
    assert_eq!("1.5", grc.blocks[1].parameters["offset_re"]);
    assert_eq!("0.5", grc.blocks[1].parameters["offset_im"]);
}

#[test]
pub fn test_dcblock_ff_execution() -> Result<()> {
    let mut fg = Flowgraph::new();
    // Constant DC input [2.0, 2.0, 2.0, ...] should decay towards 0.0
    let src = VectorSource::<f32>::new(vec![2.0; 100]);
    let dcblock = fsdr_cli::blocks::DcBlockFf::new(0.9);
    let snk = VectorSink::<f32>::new(100);

    connect!(fg, src > dcblock > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    assert_eq!(out.len(), 100);
    // After 100 samples of constant DC with R=0.9, output should be close to 0
    assert!(out.last().unwrap().abs() < 1e-3);

    Ok(())
}

#[test]
pub fn test_add_dcoffset_cc_execution() -> Result<()> {
    let mut fg = Flowgraph::new();
    let src =
        VectorSource::<Complex32>::new(vec![Complex32::new(0.0, 0.0), Complex32::new(1.0, 2.0)]);
    let add_dc = fsdr_cli::blocks::AddDcOffsetCc::new(Complex32::new(1.0, 0.5));
    let snk = VectorSink::<Complex32>::new(10);

    connect!(fg, src > add_dc > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    assert_eq!(out.len(), 2);
    assert_eq!(out[0], Complex32::new(1.0, 0.5));
    assert_eq!(out[1], Complex32::new(2.0, 2.5));

    Ok(())
}

#[test]
pub fn test_decimating_shift_addition_cc_execution() -> Result<()> {
    let mut fg = Flowgraph::new();
    // 4 complex samples at rate 0.0 (no shift), decimation 2
    let src = VectorSource::<Complex32>::new(vec![
        Complex32::new(1.0, 0.0),
        Complex32::new(9.0, 9.0),
        Complex32::new(3.0, 0.0),
        Complex32::new(9.0, 9.0),
    ]);
    let dec_shift = fsdr_cli::blocks::DecimatingShiftAdditionCc::new(0.0, 2);
    let snk = VectorSink::<Complex32>::new(10);

    connect!(fg, src > dec_shift > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    assert_eq!(out.len(), 2);
    assert!((out[0].re - 1.0).abs() < 1e-4);
    assert!((out[0].im - 0.0).abs() < 1e-4);
    assert!((out[1].re - 3.0).abs() < 1e-4);
    assert!((out[1].im - 0.0).abs() < 1e-4);

    Ok(())
}

#[test]
pub fn test_firdes_helpers() {
    use futuresdr::futuredsp::Filter;
    use futuresdr::futuredsp::PolyphaseResamplingFir;

    let taps = futuresdr::futuredsp::firdes::kaiser::multirate::<f32>(2, 75, 12, 0.0001);
    let fir = PolyphaseResamplingFir::<f32, f32, _>::new(2, 75, taps);
    let input = vec![1.0f32; 1500];
    let mut output = vec![0.0f32; 100];
    let (consumed, produced, _) = fir.filter(&input, &mut output);
    println!(
        "consumed: {}, produced: {}, output first 10: {:?}",
        consumed,
        produced,
        &output[..produced.min(10)]
    );
}

#[test]
pub fn parse_pack_bits_1to8_u8_u8() {
    let cmds = "pack_bits_1to8_u8_u8";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("blocks_unpack_k_bits_bb", grc.blocks[1].id);
    assert_eq!("8", grc.blocks[1].parameters["k"]);
}

#[test]
pub fn parse_flowcontrol() {
    let cmds = "flowcontrol 48000";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("blocks_throttle", grc.blocks[1].id);
    assert_eq!("48000", grc.blocks[1].parameters["samples_per_second"]);
}

#[test]
pub fn parse_clone() {
    let cmds = "clone";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(0, grc.blocks.len());
}

#[test]
pub fn parse_none() {
    let cmds = "none";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(2, grc.blocks.len());
    assert_eq!("blocks_null_sink", grc.blocks[1].id);
}

#[test]
pub fn parse_repeat_u8() {
    let cmds = "repeat_u8 5";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("repeat_u8", grc.blocks[1].id);
    assert_eq!("5", grc.blocks[1].parameters["repeat"]);
}

#[test]
pub fn test_repeat_u8_execution() -> Result<()> {
    let mut fg = Flowgraph::new();
    let src = VectorSource::<u8>::new(vec![0xAA, 0x55]);
    let rep = fsdr_cli::blocks::RepeatU8::new(3);
    let snk = VectorSink::<u8>::new(10);

    connect!(fg, src > rep > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    assert_eq!(out, &[0xAA, 0xAA, 0xAA, 0x55, 0x55, 0x55]);

    Ok(())
}

#[test]
pub fn test_pack_bits_1to8_and_8to1_roundtrip() -> Result<()> {
    let mut fg = Flowgraph::new();
    let src = VectorSource::<u8>::new(vec![0xA5, 0x3C]);
    let unpack = ApplyNM::<_, u8, u8, 1, 8>::new(move |v: &[u8], d: &mut [u8]| {
        let byte = v[0];
        for (i, item) in d.iter_mut().enumerate().take(8) {
            *item = (byte >> (7 - i)) & 1;
        }
    });
    let pack = ApplyNM::<_, u8, u8, 8, 1>::new(move |v: &[u8], d: &mut [u8]| {
        d[0] = v
            .iter()
            .rev()
            .enumerate()
            .map(|(i, u)| (*u) << i)
            .reduce(|a, b| a | b)
            .expect("guarantee to not be empty due to ApplyNM");
    });
    let snk = VectorSink::<u8>::new(10);

    connect!(fg, src > unpack > pack > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    assert_eq!(out, &[0xA5, 0x3C]);

    Ok(())
}

#[test]
pub fn parse_mono2stereo_s16() {
    let cmds = "mono2stereo_s16";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("mono2stereo_s16", grc.blocks[1].id);
}

#[test]
pub fn parse_dbpsk_decoder_c_u8() {
    let cmds = "dbpsk_decoder_c_u8";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("dbpsk_decoder_c_u8", grc.blocks[1].id);
}

#[test]
pub fn parse_psk31_varicode_decoder() {
    let cmds = "psk31_varicode_decoder_u8_u8";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("psk31_varicode_decoder_u8_u8", grc.blocks[1].id);
}

#[test]
pub fn parse_psk31_varicode_encoder() {
    let cmds = "psk31_varicode_encoder_u8_u8";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("psk31_varicode_encoder_u8_u8", grc.blocks[1].id);
}

#[test]
pub fn parse_bpsk_costas_loop_cc() {
    let cmds = "bpsk_costas_loop_cc 0.05 0.707";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("bpsk_costas_loop_cc", grc.blocks[1].id);
    assert_eq!("0.05", grc.blocks[1].parameters["loop_bw"]);
    assert_eq!("0.707", grc.blocks[1].parameters["damping"]);
}

#[test]
pub fn parse_pll_cc() {
    let cmds = "pll_cc 0.01 0.707";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("bpsk_costas_loop_cc", grc.blocks[1].id);
}

#[test]
pub fn test_mono2stereo_s16_execution() -> Result<()> {
    let mut fg = Flowgraph::new();
    let src = VectorSource::<i16>::new(vec![100, 200]);
    let m2s = fsdr_cli::blocks::Mono2StereoS16::new();
    let snk = VectorSink::<i16>::new(10);

    connect!(fg, src > m2s > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    assert_eq!(out, &[100, 100, 200, 200]);

    Ok(())
}

#[test]
pub fn test_dbpsk_decoder_execution() -> Result<()> {
    let mut fg = Flowgraph::new();
    // 0 phase -> 0 phase (diff 0 -> 1) -> PI phase (diff PI -> 0)
    let src = VectorSource::<Complex32>::new(vec![
        Complex32::new(1.0, 0.0),
        Complex32::new(1.0, 0.0),
        Complex32::new(-1.0, 0.0),
    ]);
    let dbpsk = fsdr_cli::blocks::DBPskDecoder::new();
    let snk = VectorSink::<u8>::new(10);

    connect!(fg, src > dbpsk > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    assert_eq!(out, &[1, 1, 0]);

    Ok(())
}

#[test]
pub fn test_varicode_encode_decode_roundtrip() -> Result<()> {
    let mut fg = Flowgraph::new();
    let msg = b"HELLO";
    let src = VectorSource::<u8>::new(msg.to_vec());
    let enc = fsdr_cli::blocks::VaricodeEncoder::new();
    let dec = fsdr_cli::blocks::VaricodeDecoder::new();
    let snk = VectorSink::<u8>::new(20);

    connect!(fg, src > enc > dec > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    assert_eq!(out, msg);

    Ok(())
}

#[test]
pub fn test_costas_loop_execution() -> Result<()> {
    let mut fg = Flowgraph::new();
    let src = VectorSource::<Complex32>::new(vec![
        Complex32::new(1.0, 0.0),
        Complex32::new(1.0, 0.0),
        Complex32::new(1.0, 0.0),
    ]);
    let costas = fsdr_cli::blocks::CostasLoopCc::new(0.05, 0.707);
    let snk = VectorSink::<Complex32>::new(10);

    connect!(fg, src > costas > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    assert_eq!(out.len(), 3);
    assert!((out[0].re - 1.0).abs() < 1e-3);
    assert!(out[0].im.abs() < 1e-3);

    Ok(())
}

#[test]
pub fn parse_fmmod_fc() {
    let cmds = "fmmod_fc";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("fmmod_fc", grc.blocks[1].id);
}

#[test]
pub fn parse_fixed_amplitude_cc() {
    let cmds = "fixed_amplitude_cc 2.5";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("fixed_amplitude_cc", grc.blocks[1].id);
    assert_eq!("2.5", grc.blocks[1].parameters["amplitude"]);
}

#[test]
pub fn parse_add_const_cc() {
    let cmds = "add_const_cc 0.5 1.5";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("add_const_cc", grc.blocks[1].id);
    assert_eq!("0.5", grc.blocks[1].parameters["real"]);
    assert_eq!("1.5", grc.blocks[1].parameters["imag"]);
}

#[test]
pub fn parse_differential_encoder_u8_u8() {
    let cmds = "differential_encoder_u8_u8";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("differential_encoder_u8_u8", grc.blocks[1].id);
}

#[test]
pub fn parse_differential_decoder_u8_u8() {
    let cmds = "differential_decoder_u8_u8";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("differential_decoder_u8_u8", grc.blocks[1].id);
}

#[test]
pub fn parse_invert_u8_u8() {
    let cmds = "invert_u8_u8";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("invert_u8_u8", grc.blocks[1].id);
}

#[test]
pub fn parse_bfsk_demod_cf() {
    let cmds = "bfsk_demod_cf";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("bfsk_demod_cf", grc.blocks[1].id);
}

#[test]
pub fn parse_detect_nan_ff() {
    let cmds = "detect_nan_ff";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("detect_nan_ff", grc.blocks[1].id);
}

#[test]
pub fn parse_yes_f() {
    let cmds = "yes_f 4.2";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(2, grc.blocks.len());
    assert_eq!("yes_f", grc.blocks[0].id);
    assert_eq!("4.2", grc.blocks[0].parameters["value"]);
}

#[test]
pub fn test_fmmod_fc_execution() -> Result<()> {
    let mut fg = Flowgraph::new();
    let src = VectorSource::<f32>::new(vec![0.0, 0.0]);
    let mod_blk = fsdr_cli::blocks::FmModFc::new();
    let snk = VectorSink::<Complex32>::new(10);

    connect!(fg, src > mod_blk > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    assert_eq!(out.len(), 2);
    assert!((out[0].re - 1.0).abs() < 1e-4);
    assert!(out[0].im.abs() < 1e-4);

    Ok(())
}

#[test]
pub fn test_fixed_amplitude_cc_execution() -> Result<()> {
    let mut fg = Flowgraph::new();
    let src = VectorSource::<Complex32>::new(vec![Complex32::new(3.0, 4.0)]);
    let fix_amp = fsdr_cli::blocks::FixedAmplitudeCc::new(10.0);
    let snk = VectorSink::<Complex32>::new(10);

    connect!(fg, src > fix_amp > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    assert_eq!(out.len(), 1);
    assert!((out[0].norm() - 10.0).abs() < 1e-4);
    assert!((out[0].re - 6.0).abs() < 1e-4);
    assert!((out[0].im - 8.0).abs() < 1e-4);

    Ok(())
}

#[test]
pub fn test_add_const_cc_execution() -> Result<()> {
    let mut fg = Flowgraph::new();
    let src = VectorSource::<Complex32>::new(vec![Complex32::new(1.0, 2.0)]);
    let add_c = fsdr_cli::blocks::AddConstCc::new(Complex32::new(3.0, 4.0));
    let snk = VectorSink::<Complex32>::new(10);

    connect!(fg, src > add_c > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    assert_eq!(out.len(), 1);
    assert!((out[0].re - 4.0).abs() < 1e-4);
    assert!((out[0].im - 6.0).abs() < 1e-4);

    Ok(())
}

#[test]
pub fn test_differential_coding_roundtrip() -> Result<()> {
    let mut fg = Flowgraph::new();
    let src = VectorSource::<u8>::new(vec![1, 0, 1, 1, 0]);
    let enc = fsdr_cli::blocks::DifferentialEncoderU8::new();
    let dec = fsdr_cli::blocks::DifferentialDecoderU8::new();
    let snk = VectorSink::<u8>::new(10);

    connect!(fg, src > enc > dec > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    assert_eq!(out, &[1, 0, 1, 1, 0]);

    Ok(())
}

#[test]
pub fn test_invert_u8_execution() -> Result<()> {
    let mut fg = Flowgraph::new();
    let src = VectorSource::<u8>::new(vec![1, 0, 1, 0]);
    let inv = fsdr_cli::blocks::InvertU8::new();
    let snk = VectorSink::<u8>::new(10);

    connect!(fg, src > inv > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    assert_eq!(out, &[0, 1, 0, 1]);

    Ok(())
}

#[test]
pub fn test_detect_nan_execution() -> Result<()> {
    let mut fg = Flowgraph::new();
    let src = VectorSource::<f32>::new(vec![1.5, f32::NAN, 2.5, f32::INFINITY]);
    let nan_blk = fsdr_cli::blocks::DetectNanFf::new();
    let snk = VectorSink::<f32>::new(10);

    connect!(fg, src > nan_blk > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    assert_eq!(out, &[1.5, 0.0, 2.5, 0.0]);

    Ok(())
}

#[test]
pub fn parse_encode_ima_adpcm_i16_u8() {
    let cmds = "encode_ima_adpcm_i16_u8";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("encode_ima_adpcm_i16_u8", grc.blocks[1].id);
}

#[test]
pub fn parse_decode_ima_adpcm_u8_i16() {
    let cmds = "decode_ima_adpcm_u8_i16";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("decode_ima_adpcm_u8_i16", grc.blocks[1].id);
}

#[test]
pub fn parse_compress_fft_adpcm_f_u8() {
    let cmds = "compress_fft_adpcm_f_u8 512";
    let result = CsdrParser::parse_command(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("compress_fft_adpcm_f_u8", grc.blocks[1].id);
    assert_eq!("512", grc.blocks[1].parameters["fft_size"]);
}

#[test]
pub fn test_ima_adpcm_encode_decode_roundtrip() -> Result<()> {
    let mut fg = Flowgraph::new();
    let original_samples: Vec<i16> = (0..200)
        .map(|k| (5000.0 * (2.0 * std::f32::consts::PI * (k as f32) / 50.0).sin()) as i16)
        .collect();
    let src = VectorSource::<i16>::new(original_samples.clone());
    let enc = fsdr_cli::blocks::AdpcmEncoderI16U8::new();
    let dec = fsdr_cli::blocks::AdpcmDecoderU8I16::new();
    let snk = VectorSink::<i16>::new(250);

    connect!(fg, src > enc > dec > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    assert_eq!(out.len(), original_samples.len());
    // After the initial adaptation (first ~10 samples), reconstructed samples track closely
    for (orig, rec) in original_samples[10..].iter().zip(out[10..].iter()) {
        assert!((orig - rec).abs() < 500, "orig: {}, rec: {}", orig, rec);
    }

    Ok(())
}

#[test]
pub fn test_compress_fft_adpcm_execution() -> Result<()> {
    let mut fg = Flowgraph::new();
    let fft_size = 16;
    let input_floats: Vec<f32> = (0..fft_size).map(|i| i as f32).collect();
    let src = VectorSource::<f32>::new(input_floats);
    let compress = fsdr_cli::blocks::CompressFftAdpcmFU8::new(fft_size);
    let snk = VectorSink::<u8>::new(50);

    connect!(fg, src > compress > snk;);

    let term_fg = Runtime::new().run(fg)?;
    let snk_blk = term_fg.block(&snk)?;
    let out = snk_blk.items();

    // 10 padding samples (5 bytes) + 16 fft samples (8 bytes) = 13 bytes
    assert_eq!(out.len(), (10 + fft_size) / 2);

    Ok(())
}

#[test]
pub fn parse_power_tagger_cc() {
    let cmds = "csdr power_tagger_cc --samp-rate 2400000 --off-threshold -50dB --off-delay 300ms --off-tag msgend";
    let result = CsdrParser::parse_multiple_commands(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_power_tagger_cc", grc.blocks[1].id);
    assert_eq!("2400000", grc.blocks[1].parameters["samp_rate"]);
    assert_eq!("-50dB", grc.blocks[1].parameters["off_threshold"]);
    assert_eq!("300ms", grc.blocks[1].parameters["off_delay"]);
    assert_eq!("msgend", grc.blocks[1].parameters["off_tag"]);
}

#[test]
pub fn parse_ctcss_detect_ff() {
    let cmds = "csdr ctcss_detect_ff --samp-rate 48000 --tone 88.5 --threshold 0.01 --duration 150ms --tag msgstart";
    let result = CsdrParser::parse_multiple_commands(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_ctcss_detect_ff", grc.blocks[1].id);
    assert_eq!("48000", grc.blocks[1].parameters["samp_rate"]);
    assert_eq!("88.5", grc.blocks[1].parameters["tone"]);
    assert_eq!("0.01", grc.blocks[1].parameters["threshold"]);
    assert_eq!("150ms", grc.blocks[1].parameters["duration"]);
    assert_eq!("msgstart", grc.blocks[1].parameters["tag"]);
}

#[test]
pub fn parse_timer_tagger_ff() {
    let cmds =
        "csdr timer_tagger_ff --samp-rate 48000 --duration 30s --start msgstart --end msgend";
    let result = CsdrParser::parse_multiple_commands(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(3, grc.blocks.len());
    assert_eq!("blocks_timer_tagger_ff", grc.blocks[1].id);
    assert_eq!("48000", grc.blocks[1].parameters["samp_rate"]);
    assert_eq!("30s", grc.blocks[1].parameters["duration"]);
    assert_eq!("msgstart", grc.blocks[1].parameters["start"]);
    assert_eq!("msgend", grc.blocks[1].parameters["end"]);
}

#[test]
pub fn parse_cmd_trigger_f() {
    let cmds =
        "csdr cmd_trigger_f --start-tag msgstart --end-tag msgend --cmd './script.sh $input_file'";
    let result = CsdrParser::parse_multiple_commands(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(2, grc.blocks.len());
    assert_eq!("blocks_cmd_trigger_f", grc.blocks[1].id);
    assert_eq!("msgstart", grc.blocks[1].parameters["start_tag"]);
    assert_eq!("msgend", grc.blocks[1].parameters["end_tag"]);
    assert_eq!(
        "./script.sh $input_file",
        grc.blocks[1].parameters["cmd"].trim_matches('\'')
    );
}

#[test]
pub fn parse_cmd_trigger_f_unquoted() {
    let cmds =
        "csdr cmd_trigger_f --start-tag msgstart --end-tag msgend --cmd ./script.sh $input_file";
    let result = CsdrParser::parse_multiple_commands(cmds);
    let grc = result.expect("").unwrap();
    assert_eq!(2, grc.blocks.len());
    assert_eq!("blocks_cmd_trigger_f", grc.blocks[1].id);
    assert_eq!("msgstart", grc.blocks[1].parameters["start_tag"]);
    assert_eq!("msgend", grc.blocks[1].parameters["end_tag"]);
    assert_eq!(
        "./script.sh $input_file",
        grc.blocks[1].parameters["cmd"].trim_matches('\'')
    );
}
