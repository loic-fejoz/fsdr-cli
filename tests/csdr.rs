use fsdr_cli::csdr::CsdrParser;
use fsdr_cli::grc::converter::Grc2FutureSdr;
use futuresdr::anyhow::Result;
use futuresdr::blocks::VectorSink;
use futuresdr::blocks::VectorSinkBuilder;
use futuresdr::blocks::VectorSource;
use futuresdr::macros::connect;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;

#[test]
pub fn parse_convert_u8_f() {
    let mut cmds = "convert_u8_f".split_whitespace().peekable();
    let result = CsdrParser::parse_command(&mut cmds);
    assert!(result.is_ok());
    let grc = result.expect("");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("convert_u8_f", grc.blocks[0].id);
    println!("{grc:?}");
    assert_eq!(2, grc.connections.len());
}

#[test]
pub fn parse_multiple_commands_retrocompatibility() {
    let cmds = "csdr convert_u8_f | csdr fmdemod_quadri_cf | csdr fractional_decimator_ff 5 | csdr deemphasis_wfm_ff 48000 50e-6 | csdr convert_f_s16";
    let mut cmds = cmds.split_whitespace().peekable();
    let result = CsdrParser::parse_multiple_commands(&mut cmds);
    assert!(result.is_ok());
    let grc = result.expect("");
    println!("{grc:?}");
    assert_eq!(7, grc.blocks.len());
    assert_eq!(6, grc.connections.len());
}

#[test]
pub fn parse_multiple_commands() {
    let cmds = "csdr convert_u8_f | fmdemod_quadri_cf | fractional_decimator_ff 5 | deemphasis_wfm_ff 48000 50e-6 | convert_f_s16";
    let mut cmds = cmds.split_whitespace().peekable();
    let result = CsdrParser::parse_multiple_commands(&mut cmds);
    assert!(result.is_ok());
    let grc = result.expect("");
    println!("{grc:?}");
    assert_eq!(7, grc.blocks.len());
    assert_eq!(6, grc.connections.len());
}

#[test]
pub fn parse_shift_addition_cc_1256() {
    let mut cmds = "shift_addition_cc 1256".split_whitespace().peekable();
    let result = CsdrParser::parse_command(&mut cmds);
    assert!(result.is_ok());
    let grc = result.expect("");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("blocks_freqshift_cc", grc.blocks[0].id);
    println!("{grc:?}");
    assert_eq!(2, grc.connections.len());
}

#[test]
pub fn parse_limit_ff() {
    let mut cmds = "limit_ff".split_whitespace().peekable();
    let result = CsdrParser::parse_command(&mut cmds);
    assert!(result.is_ok());
    let grc = result.expect("");
    println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_rail_ff", grc.blocks[0].id);
    let low_threshold = grc.blocks[0]
        .parameters
        .get("lo")
        .expect("low threshold must be defined");
    assert_eq!("-1.0", low_threshold);
    let high_threshold = grc.blocks[0]
        .parameters
        .get("hi")
        .expect("high threshold must be defined");
    assert_eq!("1.0", high_threshold);
    assert_eq!(2, grc.connections.len());
}

#[test]
pub fn parse_limit_ff_with_max_amplitude() -> Result<()> {
    let mut cmds = "limit_ff 3.0".split_whitespace().peekable();
    let result = CsdrParser::parse_command(&mut cmds);
    assert!(result.is_ok());
    let grc = result.expect("");
    //println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_rail_ff", grc.blocks[0].id);
    let low_threshold = grc.blocks[0]
        .parameters
        .get("lo")
        .expect("low threshold must be defined");
    assert_eq!("-3.0", low_threshold);
    let high_threshold = grc.blocks[0]
        .parameters
        .get("hi")
        .expect("high threshold must be defined");
    assert_eq!("3.0", high_threshold);
    assert_eq!(2, grc.connections.len());

    let mut fg = Flowgraph::new();
    let orig: Vec<f32> = vec![
        -10.0, -5.0, -3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 10.0,
    ];
    let src = VectorSource::<f32>::new(orig);
    let vect_sink_0 = VectorSinkBuilder::<f32>::new().build();

    let block_under_test = Grc2FutureSdr::convert_add_block(&mut fg, &grc.blocks[0], &grc);
    let block_under_test = block_under_test.unwrap().expect("");

    connect!(fg,
        src > block_under_test;
        block_under_test > vect_sink_0;
    );
    fg = Runtime::new().run(fg)?;

    let snk_0 = fg.kernel::<VectorSink<f32>>(vect_sink_0).unwrap();
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
    let mut cmds = cmds.split_whitespace().peekable();
    let result = CsdrParser::parse_multiple_commands(&mut cmds);
    assert!(result.is_ok());
    let grc = result.expect("");
    // println!("{grc:?}");
    assert_eq!(4 + 2, grc.blocks.len());

    let first_blk = &grc.blocks[1];
    assert_eq!("analog_rail_ff", first_blk.id);
    let low_threshold = first_blk
        .parameters
        .get("lo")
        .expect("low threshold must be defined");
    assert_eq!("-1.0", low_threshold);
    let high_threshold = first_blk
        .parameters
        .get("hi")
        .expect("high threshold must be defined");
    assert_eq!("1.0", high_threshold);

    let second_blk = &grc.blocks[2];
    assert_eq!("analog_rail_ff", second_blk.id);
    let low_threshold = second_blk
        .parameters
        .get("lo")
        .expect("low threshold must be defined");
    assert_eq!("-16.0", low_threshold);
    let high_threshold = second_blk
        .parameters
        .get("hi")
        .expect("high threshold must be defined");
    assert_eq!("16.0", high_threshold);
}

#[test]
pub fn parse_fastdc_block() -> Result<()> {
    let mut cmds = "fastdcblock_ff".split_whitespace().peekable();
    let result = CsdrParser::parse_command(&mut cmds);
    assert!(result.is_ok());
    let grc = result.expect("");
    // println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("dc_blocker_xx", grc.blocks[0].id);
    let dc_length = grc.blocks[0]
        .parameters
        .get("length")
        .expect("length must be defined");
    assert_eq!("32", dc_length);
    let long_form = grc.blocks[0]
        .parameters
        .get("long_form")
        .expect("long_form must be defined");
    assert_eq!("False", long_form);
    let data_type = grc.blocks[0]
        .parameters
        .get("type")
        .expect("type must be defined");
    assert_eq!("ff", data_type);
    assert_eq!(2, grc.connections.len());

    let mut fg = Flowgraph::new();
    let orig: Vec<f32> = vec![0.0, -1.0, -2.0, -1.0, 0.0, 1.0, 2.0, 1.0];
    let orig: Vec<f32> = orig.iter().map(|x| x + 5.0).collect();
    let orig = orig.repeat(32);
    let src = VectorSource::<f32>::new(orig);
    let vect_sink_0 = VectorSinkBuilder::<f32>::new().build();

    let block_under_test = Grc2FutureSdr::convert_add_block(&mut fg, &grc.blocks[0], &grc);
    let block_under_test = block_under_test.unwrap().expect("");

    connect!(fg,
        src > block_under_test;
        block_under_test > vect_sink_0;
    );
    fg = Runtime::new().run(fg)?;

    let snk_0 = fg.kernel::<VectorSink<f32>>(vect_sink_0).unwrap();
    let snk_0 = snk_0.items();
    // println!("{snk_0:?}");
    assert!(snk_0.iter().skip(110).all(|v| -5.0 <= *v && *v <= 5.0));
    assert!(snk_0.iter().skip(175).all(|v| -4.0 <= *v && *v <= 4.0));
    assert!(snk_0.iter().skip(200).all(|v| -3.0 <= *v && *v <= 3.0));
    Ok(())
}

#[test]
pub fn parse_amdemod_cf() -> Result<()> {
    let mut cmds = "amdemod_cf".split_whitespace().peekable();
    let result = CsdrParser::parse_command(&mut cmds);
    assert!(result.is_ok());
    let grc = result.expect("");
    // println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("blocks_complex_to_mag", grc.blocks[0].id);
    assert_eq!(2, grc.connections.len());

    let mut fg = Flowgraph::new();
    let orig: Vec<Complex32> = (0..10)
        .map(|x| x as f32)
        .map(|x| Complex32::new(x * x, 0.0))
        .collect();
    let src = VectorSource::<Complex32>::new(orig);
    let vect_sink_0 = VectorSinkBuilder::<f32>::new().build();

    let block_under_test = Grc2FutureSdr::convert_add_block(&mut fg, &grc.blocks[0], &grc);
    let block_under_test = block_under_test.unwrap().expect("");

    connect!(fg,
        src > block_under_test;
        block_under_test > vect_sink_0;
    );
    fg = Runtime::new().run(fg)?;

    let snk_0 = fg.kernel::<VectorSink<f32>>(vect_sink_0).unwrap();
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
    let mut cmds = "agc_ff".split_whitespace().peekable();
    let result = CsdrParser::parse_command(&mut cmds);
    assert!(result.is_ok());
    let grc = result.expect("");
    // println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_agc_xx", grc.blocks[0].id);
    let reference = grc.blocks[0]
        .parameters
        .get("reference")
        .expect("reference must be defined");
    assert_eq!("0.8", reference);
    let max_gain = grc.blocks[0]
        .parameters
        .get("max_gain")
        .expect("max_gain must be defined");
    assert_eq!("65536.0", max_gain);
    let rate = grc.blocks[0]
        .parameters
        .get("rate")
        .expect("rate must be defined");
    assert_eq!("0.0001", rate);
    let data_type = grc.blocks[0]
        .parameters
        .get("type")
        .expect("type must be defined");
    assert_eq!("float", data_type);
    assert_eq!(2, grc.connections.len());

    let mut fg = Flowgraph::new();
    let orig: Vec<f32> = (0..256).map(|x| (x as f32).sin() * 0.5).collect();
    let orig = orig.repeat(256);
    let src = VectorSource::<f32>::new(orig);
    let vect_sink_0 = VectorSinkBuilder::<f32>::new().build();

    let block_under_test = Grc2FutureSdr::convert_add_block(&mut fg, &grc.blocks[0], &grc);
    let block_under_test = block_under_test.unwrap().expect("");

    connect!(fg,
        src > block_under_test;
        block_under_test > vect_sink_0;
    );
    fg = Runtime::new().run(fg)?;

    let snk_0 = fg.kernel::<VectorSink<f32>>(vect_sink_0).unwrap();
    let snk_0 = snk_0.items();
    // println!("{snk_0:?}");
    assert!(snk_0.iter().any(|v| *v > 0.7));
    Ok(())
}

#[test]
pub fn parse_agc_ff_with_reference() -> Result<()> {
    let mut cmds = "agc_ff --reference 1.0".split_whitespace().peekable();
    let result = CsdrParser::parse_command(&mut cmds);
    assert!(result.is_ok());
    let grc = result.expect("");
    // println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_agc_xx", grc.blocks[0].id);
    let reference = grc.blocks[0]
        .parameters
        .get("reference")
        .expect("reference must be defined");
    assert_eq!("1.0", reference);
    let max_gain = grc.blocks[0]
        .parameters
        .get("max_gain")
        .expect("max_gain must be defined");
    assert_eq!("65536.0", max_gain);
    let rate = grc.blocks[0]
        .parameters
        .get("rate")
        .expect("rate must be defined");
    assert_eq!("0.0001", rate);
    let data_type = grc.blocks[0]
        .parameters
        .get("type")
        .expect("type must be defined");
    assert_eq!("float", data_type);
    Ok(())
}

#[test]
pub fn parse_agc_ff_with_reference_and_max_gain() -> Result<()> {
    let mut cmds = "agc_ff --reference 0.9 --max 256.0"
        .split_whitespace()
        .peekable();
    let result = CsdrParser::parse_command(&mut cmds);
    assert!(result.is_ok());
    let grc = result.expect("");
    // println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_agc_xx", grc.blocks[0].id);
    let reference = grc.blocks[0]
        .parameters
        .get("reference")
        .expect("reference must be defined");
    assert_eq!("0.9", reference);
    let max_gain = grc.blocks[0]
        .parameters
        .get("max_gain")
        .expect("max_gain must be defined");
    assert_eq!("256.0", max_gain);
    let rate = grc.blocks[0]
        .parameters
        .get("rate")
        .expect("rate must be defined");
    assert_eq!("0.0001", rate);
    let data_type = grc.blocks[0]
        .parameters
        .get("type")
        .expect("type must be defined");
    assert_eq!("float", data_type);
    Ok(())
}

#[test]
pub fn parse_agc_ff_with_rate() -> Result<()> {
    let mut cmds = "agc_ff --rate 0.0002".split_whitespace().peekable();
    let result = CsdrParser::parse_command(&mut cmds);
    assert!(result.is_ok());
    let grc = result.expect("");
    // println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("analog_agc_xx", grc.blocks[0].id);
    let reference = grc.blocks[0]
        .parameters
        .get("reference")
        .expect("reference must be defined");
    assert_eq!("0.8", reference);
    let max_gain = grc.blocks[0]
        .parameters
        .get("max_gain")
        .expect("max_gain must be defined");
    assert_eq!("65536.0", max_gain);
    let rate = grc.blocks[0]
        .parameters
        .get("rate")
        .expect("rate must be defined");
    assert_eq!("0.0002", rate);
    let data_type = grc.blocks[0]
        .parameters
        .get("type")
        .expect("type must be defined");
    assert_eq!("float", data_type);
    Ok(())
}

#[test]
pub fn parse_fir_decimate_cc() -> Result<()> {
    let mut cmds = "fir_decimate_cc 50".split_whitespace().peekable();
    let result = CsdrParser::parse_command(&mut cmds);
    assert!(result.is_ok());
    let grc = result.expect("");
    // println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("fir_filter_xxx", grc.blocks[0].id);
    let decimation_factor = grc.blocks[0]
        .parameters
        .get("decim")
        .expect("decimation_factor must be defined");
    assert_eq!("50", decimation_factor);
    let transition_bw = grc.blocks[0]
        .parameters
        .get("transition_bw")
        .expect("transition_bw  must be defined");
    assert_eq!("0.05", transition_bw);
    let window = grc.blocks[0]
        .parameters
        .get("window")
        .expect("window must be defined");
    assert_eq!("HAMMING", window);
    let data_type = grc.blocks[0]
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
    let src = VectorSource::<Complex32>::new(orig);
    let vect_sink_0 = VectorSinkBuilder::<Complex32>::new().build();

    let block_under_test = Grc2FutureSdr::convert_add_block(&mut fg, &grc.blocks[0], &grc);
    let block_under_test = block_under_test.unwrap().expect("");

    connect!(fg,
        src > block_under_test;
        block_under_test > vect_sink_0;
    );
    fg = Runtime::new().run(fg)?;

    let snk_0 = fg.kernel::<VectorSink<Complex32>>(vect_sink_0).unwrap();
    let snk_0 = snk_0.items();
    // println!("{snk_0:?}");
    assert_eq!(original_length / 50 - 2, snk_0.len());

    Ok(())
}

#[test]
pub fn parse_fir_decimate_cc_bw() -> Result<()> {
    let mut cmds = "fir_decimate_cc 50 0.06".split_whitespace().peekable();
    let result = CsdrParser::parse_command(&mut cmds);
    assert!(result.is_ok());
    let grc = result.expect("");
    // println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("fir_filter_xxx", grc.blocks[0].id);
    let decimation_factor = grc.blocks[0]
        .parameters
        .get("decim")
        .expect("decimation_factor must be defined");
    assert_eq!("50", decimation_factor);
    let transition_bw = grc.blocks[0]
        .parameters
        .get("transition_bw")
        .expect("transition_bw  must be defined");
    assert_eq!("0.06", transition_bw);
    let window = grc.blocks[0]
        .parameters
        .get("window")
        .expect("window must be defined");
    assert_eq!("HAMMING", window);
    let data_type = grc.blocks[0]
        .parameters
        .get("type")
        .expect("type must be defined");
    assert_eq!("ccc", data_type);
    Ok(())
}

#[test]
pub fn parse_fir_decimate_cc_bw_windows() -> Result<()> {
    let mut cmds = "fir_decimate_cc 50 0.06 BLACKMAN"
        .split_whitespace()
        .peekable();
    let result = CsdrParser::parse_command(&mut cmds);
    assert!(result.is_ok());
    let grc = result.expect("");
    // println!("{grc:?}");
    assert_eq!(3, grc.blocks.len());
    assert_eq!("fir_filter_xxx", grc.blocks[0].id);
    let decimation_factor = grc.blocks[0]
        .parameters
        .get("decim")
        .expect("decimation_factor must be defined");
    assert_eq!("50", decimation_factor);
    let transition_bw = grc.blocks[0]
        .parameters
        .get("transition_bw")
        .expect("transition_bw  must be defined");
    assert_eq!("0.06", transition_bw);
    let window = grc.blocks[0]
        .parameters
        .get("window")
        .expect("window must be defined");
    assert_eq!("BLACKMAN", window);
    let data_type = grc.blocks[0]
        .parameters
        .get("type")
        .expect("type must be defined");
    assert_eq!("ccc", data_type);
    Ok(())
}
