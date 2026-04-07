use anyhow::Result;
use fsdr_cli::grc::converter::Grc2FutureSdr;
use fsdr_cli::grc::{BlockInstance, Grc, Metadata, Options};
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Runtime;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn get_temp_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("fsdr_test_{}_{}_{}", name, std::process::id(), now));
    path
}

#[test]
pub fn test_run_grc_flowgraph() -> Result<()> {
    let input_path = get_temp_path("input.f32");
    let output_path = get_temp_path("output.f32");

    // 1. Create input data: [1.0, 2.0, 3.0, 4.0]
    let input_data: [f32; 4] = [1.0, 2.0, 3.0, 4.0];
    let mut input_file = fs::File::create(&input_path)?;
    input_file.write_all(unsafe {
        std::slice::from_raw_parts(
            input_data.as_ptr() as *const u8,
            std::mem::size_of::<[f32; 4]>(),
        )
    })?;

    // 2. Build GRC representation: Source -> AddConst(10.0) -> Sink
    let blocks = vec![
        BlockInstance::new("source", "blocks_file_source")
            .with("file", input_path.to_str().unwrap())
            .with("type", "float")
            .with("repeat", "false")
            .with("vlen", "1"),
        BlockInstance::new("add_const", "blocks_add_const_vxx")
            .with("const", "10.0")
            .with("type", "float"),
        BlockInstance::new("sink", "blocks_file_sink")
            .with("file", output_path.to_str().unwrap())
            .with("type", "float")
            .with("unbuffered", "false")
            .with("vlen", "1")
            .with("append", "false"),
    ];

    let connections = vec![
        [
            "source".to_string(),
            "0".to_string(),
            "add_const".to_string(),
            "0".to_string(),
        ],
        [
            "add_const".to_string(),
            "0".to_string(),
            "sink".to_string(),
            "0".to_string(),
        ],
    ];

    let grc = Grc {
        options: Options::default(),
        blocks,
        connections,
        metadata: Metadata {
            file_format: 1,
            grc_version: "3.10.3.0".to_string(),
        },
    };

    // 3. Convert to FutureSDR flowgraph and run
    let mut converter = Grc2FutureSdr::new();
    let fg = converter.convert_grc(grc)?;
    Runtime::new().run(fg)?;

    // 4. Verify output: [11.0, 12.0, 13.0, 14.0]
    let output_data_bytes = fs::read(&output_path)?;
    let output_data: &[f32] = unsafe {
        std::slice::from_raw_parts(
            output_data_bytes.as_ptr() as *const f32,
            output_data_bytes.len() / std::mem::size_of::<f32>(),
        )
    };

    assert_eq!(output_data.len(), 4);
    assert_eq!(output_data[0], 11.0);
    assert_eq!(output_data[1], 12.0);
    assert_eq!(output_data[2], 13.0);
    assert_eq!(output_data[3], 14.0);

    // Cleanup
    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_path);

    Ok(())
}

#[test]
pub fn test_run_grc_multiply() -> Result<()> {
    let input_path = get_temp_path("input_mul.f32");
    let output_path = get_temp_path("output_mul.f32");

    // 1. Create input data: [1.0, 2.0, 3.0, 4.0]
    let input_data: [f32; 4] = [1.0, 2.0, 3.0, 4.0];
    let mut input_file = fs::File::create(&input_path)?;
    input_file.write_all(unsafe {
        std::slice::from_raw_parts(
            input_data.as_ptr() as *const u8,
            std::mem::size_of::<[f32; 4]>(),
        )
    })?;

    // 2. Build GRC representation: Source -> MultiplyConst(2.0) -> Sink
    let blocks = vec![
        BlockInstance::new("source", "blocks_file_source")
            .with("file", input_path.to_str().unwrap())
            .with("type", "float")
            .with("repeat", "false")
            .with("vlen", "1"),
        BlockInstance::new("mul_const", "blocks_multiply_const_vxx")
            .with("const", "2.0")
            .with("type", "float"),
        BlockInstance::new("sink", "blocks_file_sink")
            .with("file", output_path.to_str().unwrap())
            .with("type", "float")
            .with("unbuffered", "false")
            .with("vlen", "1")
            .with("append", "false"),
    ];

    let connections = vec![
        [
            "source".to_string(),
            "0".to_string(),
            "mul_const".to_string(),
            "0".to_string(),
        ],
        [
            "mul_const".to_string(),
            "0".to_string(),
            "sink".to_string(),
            "0".to_string(),
        ],
    ];

    let grc = Grc {
        options: Options::default(),
        blocks,
        connections,
        metadata: Metadata {
            file_format: 1,
            grc_version: "3.10.3.0".to_string(),
        },
    };

    // 3. Convert to FutureSDR flowgraph and run
    let mut converter = Grc2FutureSdr::new();
    let fg = converter.convert_grc(grc)?;
    Runtime::new().run(fg)?;

    // 4. Verify output: [2.0, 4.0, 6.0, 8.0]
    let output_data_bytes = fs::read(&output_path)?;
    let output_data: &[f32] = unsafe {
        std::slice::from_raw_parts(
            output_data_bytes.as_ptr() as *const f32,
            output_data_bytes.len() / std::mem::size_of::<f32>(),
        )
    };

    assert_eq!(output_data.len(), 4);
    assert_eq!(output_data[0], 2.0);
    assert_eq!(output_data[1], 4.0);
    assert_eq!(output_data[2], 6.0);
    assert_eq!(output_data[3], 8.0);

    // Cleanup
    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_path);

    Ok(())
}

#[test]
pub fn test_load_and_run_grc_file() -> Result<()> {
    use fsdr_cli::grc::GrcParser;

    let grc_path = get_temp_path("test.grc");
    let input_path = get_temp_path("input_file.f32");
    let output_path = get_temp_path("output_file.f32");

    // 1. Create input data: [1.0, 2.0, 3.0, 4.0]
    let input_data: [f32; 4] = [1.0, 2.0, 3.0, 4.0];
    let mut input_file = fs::File::create(&input_path)?;
    input_file.write_all(unsafe {
        std::slice::from_raw_parts(
            input_data.as_ptr() as *const u8,
            std::mem::size_of::<[f32; 4]>(),
        )
    })?;

    // 2. Build GRC and save to file
    let blocks = vec![
        BlockInstance::new("source", "blocks_file_source")
            .with("file", input_path.to_str().unwrap())
            .with("type", "float")
            .with("repeat", "false")
            .with("vlen", "1"),
        BlockInstance::new("add_const", "blocks_add_const_vxx")
            .with("const", "5.0")
            .with("type", "float"),
        BlockInstance::new("sink", "blocks_file_sink")
            .with("file", output_path.to_str().unwrap())
            .with("type", "float")
            .with("unbuffered", "false")
            .with("vlen", "1")
            .with("append", "false"),
    ];

    let connections = vec![
        [
            "source".to_string(),
            "0".to_string(),
            "add_const".to_string(),
            "0".to_string(),
        ],
        [
            "add_const".to_string(),
            "0".to_string(),
            "sink".to_string(),
            "0".to_string(),
        ],
    ];

    let grc = Grc {
        options: Options::default(),
        blocks,
        connections,
        metadata: Metadata {
            file_format: 1,
            grc_version: "3.10.3.0".to_string(),
        },
    };

    GrcParser::save(&grc_path, &grc)?;

    // 3. Load from file and run
    let loaded_grc = GrcParser::load(&grc_path)?;
    let mut converter = Grc2FutureSdr::new();
    let fg = converter.convert_grc(loaded_grc)?;
    Runtime::new().run(fg)?;

    // 4. Verify output: [6.0, 7.0, 8.0, 9.0]
    let output_data_bytes = fs::read(&output_path)?;
    let output_data: &[f32] = unsafe {
        std::slice::from_raw_parts(
            output_data_bytes.as_ptr() as *const f32,
            output_data_bytes.len() / std::mem::size_of::<f32>(),
        )
    };

    assert_eq!(output_data.len(), 4);
    assert_eq!(output_data[0], 6.0);
    assert_eq!(output_data[1], 7.0);
    assert_eq!(output_data[2], 8.0);
    assert_eq!(output_data[3], 9.0);

    // Cleanup
    let _ = fs::remove_file(grc_path);
    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_path);

    Ok(())
}

#[test]
pub fn test_run_complex_grc_chain() -> Result<()> {
    let input_path = get_temp_path("input_complex.f32");
    let output_path = get_temp_path("output_complex.f32");

    // 1. Create input data: [1.0, 2.0]
    let input_data: [f32; 2] = [1.0, 2.0];
    let mut input_file = fs::File::create(&input_path)?;
    input_file.write_all(unsafe {
        std::slice::from_raw_parts(
            input_data.as_ptr() as *const u8,
            std::mem::size_of::<[f32; 2]>(),
        )
    })?;

    // 2. Build GRC: Source -> Add(10.0) -> Mul(2.0) -> Sink
    let blocks = vec![
        BlockInstance::new("source", "blocks_file_source")
            .with("file", input_path.to_str().unwrap())
            .with("type", "float")
            .with("repeat", "false")
            .with("vlen", "1"),
        BlockInstance::new("add", "blocks_add_const_vxx")
            .with("const", "10.0")
            .with("type", "float"),
        BlockInstance::new("mul", "blocks_multiply_const_vxx")
            .with("const", "2.0")
            .with("type", "float"),
        BlockInstance::new("sink", "blocks_file_sink")
            .with("file", output_path.to_str().unwrap())
            .with("type", "float")
            .with("unbuffered", "false")
            .with("vlen", "1")
            .with("append", "false"),
    ];

    let connections = vec![
        [
            "source".to_string(),
            "0".to_string(),
            "add".to_string(),
            "0".to_string(),
        ],
        [
            "add".to_string(),
            "0".to_string(),
            "mul".to_string(),
            "0".to_string(),
        ],
        [
            "mul".to_string(),
            "0".to_string(),
            "sink".to_string(),
            "0".to_string(),
        ],
    ];

    let grc = Grc {
        options: Options::default(),
        blocks,
        connections,
        metadata: Metadata {
            file_format: 1,
            grc_version: "3.10.3.0".to_string(),
        },
    };

    // 3. Convert and run
    let mut converter = Grc2FutureSdr::new();
    let fg = converter.convert_grc(grc)?;
    Runtime::new().run(fg)?;

    // 4. Verify output: [(1+10)*2, (2+10)*2] = [22.0, 24.0]
    let output_data_bytes = fs::read(&output_path)?;
    let output_data: &[f32] = unsafe {
        std::slice::from_raw_parts(
            output_data_bytes.as_ptr() as *const f32,
            output_data_bytes.len() / std::mem::size_of::<f32>(),
        )
    };

    assert_eq!(output_data.len(), 2);
    assert_eq!(output_data[0], 22.0);
    assert_eq!(output_data[1], 24.0);

    // Cleanup
    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_path);

    Ok(())
}

#[test]
pub fn test_run_grc_lpf_agc() -> Result<()> {
    let input_path = get_temp_path("input_lpf.c32");
    let output_path = get_temp_path("output_lpf.f32");

    // 1. Create complex input data: [1.0+0j, 2.0+0j, ...] repeated to fill the filter
    let mut input_data = Vec::new();
    for i in 0..100 {
        input_data.push(Complex32::new(i as f32, 0.0));
    }
    let mut input_file = fs::File::create(&input_path)?;
    input_file.write_all(unsafe {
        std::slice::from_raw_parts(
            input_data.as_ptr() as *const u8,
            input_data.len() * std::mem::size_of::<Complex32>(),
        )
    })?;

    // 2. Build GRC: Source -> LPF -> Realpart -> AGC -> Sink
    let blocks = vec![
        BlockInstance::new("source", "blocks_file_source")
            .with("file", input_path.to_str().unwrap())
            .with("type", "complex")
            .with("repeat", "false")
            .with("vlen", "1"),
        BlockInstance::new("lpf", "low_pass_filter")
            .with("type", "fir_filter_ccf")
            .with("decim", "1")
            .with("interp", "1")
            .with("gain", "1.0")
            .with("samp_rate", "1.0")
            .with("cutoff_freq", "0.4")
            .with("width", "0.1")
            .with("win", "window.WIN_HAMMING"),
        BlockInstance::new("realpart", "blocks_complex_to_real").with("vlen", "1"),
        BlockInstance::new("agc", "analog_agc_xx")
            .with("type", "float")
            .with("reference", "1.0")
            .with("max_gain", "100.0")
            .with("rate", "1e-4"),
        BlockInstance::new("sink", "blocks_file_sink")
            .with("file", output_path.to_str().unwrap())
            .with("type", "float")
            .with("unbuffered", "false")
            .with("vlen", "1")
            .with("append", "false"),
    ];

    let connections = vec![
        [
            "source".to_string(),
            "0".to_string(),
            "lpf".to_string(),
            "0".to_string(),
        ],
        [
            "lpf".to_string(),
            "0".to_string(),
            "realpart".to_string(),
            "0".to_string(),
        ],
        [
            "realpart".to_string(),
            "0".to_string(),
            "agc".to_string(),
            "0".to_string(),
        ],
        [
            "agc".to_string(),
            "0".to_string(),
            "sink".to_string(),
            "0".to_string(),
        ],
    ];

    let grc = Grc {
        options: Options::default(),
        blocks,
        connections,
        metadata: Metadata {
            file_format: 1,
            grc_version: "3.10.3.0".to_string(),
        },
    };

    // 3. Convert and run
    let mut converter = Grc2FutureSdr::new();
    let fg = converter.convert_grc(grc)?;
    Runtime::new().run(fg)?;

    // 4. Verify output exists and has some data
    let output_data_bytes = fs::read(&output_path)?;
    assert!(!output_data_bytes.is_empty());

    // Cleanup
    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_path);

    Ok(())
}

#[test]
pub fn test_grc_save_content_validity() -> Result<()> {
    use fsdr_cli::grc::GrcParser;

    let grc_path = get_temp_path("content_check.grc");

    let blocks = vec![BlockInstance::new("test_block", "blocks_null_sink")];

    let grc = Grc {
        options: Options::default(),
        blocks,
        connections: Vec::new(),
        metadata: Metadata {
            file_format: 1,
            grc_version: "3.10.3.0".to_string(),
        },
    };

    GrcParser::save(&grc_path, &grc)?;

    let content = fs::read_to_string(&grc_path)?;

    // Check for essential YAML fields GNU Radio expects
    assert!(content.contains("options:"));
    assert!(content.contains("parameters:"));
    assert!(content.contains("author: fsdr-cli"));
    assert!(content.contains("generate_options: qt_gui"));
    assert!(content.contains("blocks:"));
    assert!(content.contains("- name: test_block"));
    assert!(content.contains("id: blocks_null_sink"));
    assert!(content.contains("metadata:"));
    assert!(content.contains("file_format: 1"));
    assert!(content.contains("grc_version: 3.10.3.0"));

    let _ = fs::remove_file(grc_path);
    Ok(())
}

#[test]
pub fn test_run_grc_fmdemod() -> Result<()> {
    let input_path = get_temp_path("input_fm.c32");
    let output_path = get_temp_path("output_fm.f32");

    // 1. Create a frequency modulated signal
    let mut input_data = Vec::new();
    let mut phase = 0.0f32;
    let freq = 0.1f32; // normalized frequency
    let deviation = 0.05f32;
    for i in 0..100 {
        let mod_signal = (i as f32 * 0.05).sin();
        phase += 2.0 * std::f32::consts::PI * (freq + deviation * mod_signal);
        input_data.push(Complex32::new(phase.cos(), phase.sin()));
    }
    let mut input_file = fs::File::create(&input_path)?;
    input_file.write_all(unsafe {
        std::slice::from_raw_parts(
            input_data.as_ptr() as *const u8,
            input_data.len() * std::mem::size_of::<Complex32>(),
        )
    })?;

    // 2. Build GRC: Source -> FMDemod -> Sink
    let blocks = vec![
        BlockInstance::new("source", "blocks_file_source")
            .with("file", input_path.to_str().unwrap())
            .with("type", "complex")
            .with("repeat", "false")
            .with("vlen", "1"),
        BlockInstance::new("fmdemod", "analog_quadrature_demod_cf")
            .with("gain", "1.0")
            .with("algorithm", "quadri"),
        BlockInstance::new("sink", "blocks_file_sink")
            .with("file", output_path.to_str().unwrap())
            .with("type", "float")
            .with("unbuffered", "false")
            .with("vlen", "1")
            .with("append", "false"),
    ];

    let connections = vec![
        [
            "source".to_string(),
            "0".to_string(),
            "fmdemod".to_string(),
            "0".to_string(),
        ],
        [
            "fmdemod".to_string(),
            "0".to_string(),
            "sink".to_string(),
            "0".to_string(),
        ],
    ];

    let grc = Grc {
        options: Options::default(),
        blocks,
        connections,
        metadata: Metadata {
            file_format: 1,
            grc_version: "3.10.3.0".to_string(),
        },
    };

    // 3. Convert and run
    let mut converter = Grc2FutureSdr::new();
    let fg = converter.convert_grc(grc)?;
    Runtime::new().run(fg)?;

    // 4. Verify output exists
    let output_data_bytes = fs::read(&output_path)?;
    assert!(!output_data_bytes.is_empty());

    // Cleanup
    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_path);

    Ok(())
}
