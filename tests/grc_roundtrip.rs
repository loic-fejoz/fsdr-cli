use anyhow::Result;
use fsdr_cli::csdr_cmd::CsdrParser;
use fsdr_cli::grc::GrcParser;
use std::fs;
use std::path::PathBuf;

fn get_temp_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!(
        "fsdr_roundtrip_{}_{}_{}",
        name,
        std::process::id(),
        now
    ));
    path
}

#[test]
pub fn test_grc_save_load_roundtrip() -> Result<()> {
    let grc_path = get_temp_path("roundtrip.grc");

    // 1. Parse a complex command to GRC
    let cmd = "csdr load_c input.bin ! shift_addition_cc 0.25 ! fir_decimate_cc 10 ! fmdemod_quadri_cf ! convert_f_u8 ! dump_u8";
    let grc = CsdrParser::parse_multiple_commands(cmd)?.expect("Failed to parse");

    // 2. Save to file
    GrcParser::save(&grc_path, &grc)?;

    // 3. Load from file
    let loaded_grc = GrcParser::load(&grc_path)?;

    // 4. Verify roundtrip equality (Grc implements PartialEq)
    assert_eq!(grc, loaded_grc);

    // 5. Check some specific properties
    assert_eq!(grc.blocks.len(), 6); // load_c, shift, decimate, fmdemod, convert, dump
                                     // Let's count them:
                                     // 0: blocks_file_source (load_c)
                                     // 1: blocks_freqshift_cc (shift_addition_cc)
                                     // 2: fir_filter_xxx (fir_decimate_cc)
                                     // 3: analog_quadrature_demod_cf (fmdemod_quadri_cf)
                                     // 4: blocks_float_to_uchar (convert_f_u8)
                                     // 5: dump_u8 (dump_u8)
    assert_eq!(grc.blocks[0].id, "blocks_file_source");
    assert_eq!(grc.blocks[1].id, "blocks_freqshift_cc");
    assert_eq!(grc.blocks[2].id, "fir_filter_xxx");
    assert_eq!(grc.blocks[3].id, "analog_quadrature_demod_cf");
    assert_eq!(grc.blocks[4].id, "blocks_float_to_uchar");
    assert_eq!(grc.blocks[5].id, "dump_u8");

    // Cleanup
    let _ = fs::remove_file(grc_path);
    Ok(())
}

#[test]
pub fn test_grc_files_in_tests_directory() -> Result<()> {
    let grc_files = vec![
        "tests/am-demodulation.grc",
        "tests/chain1.grc",
        "tests/chain3.grc",
        "tests/hackrf-wfm.grc",
        "tests/kiss_example.grc",
        "tests/nfm.grc",
        "tests/realpart_cf.grc",
        "tests/ssb-decoder.grc",
    ];

    for file in grc_files {
        let grc = GrcParser::load(file);
        assert!(grc.is_ok(), "Failed to load {}: {:?}", file, grc.err());

        // Save to a temporary file and reload to ensure our save is compatible with what we load
        let temp_path = get_temp_path("re-save.grc");
        let grc = grc.unwrap();
        GrcParser::save(&temp_path, &grc)?;
        let re_loaded = GrcParser::load(&temp_path)?;
        assert_eq!(grc, re_loaded, "Roundtrip failed for {}", file);

        let _ = fs::remove_file(temp_path);
    }
    Ok(())
}

#[test]
pub fn test_csdr_output_option_saves_grc() -> Result<()> {
    // This tests the functionality in main.rs indirectly by checking CsdrCmd::output and parse.
    // In main.rs, if csdr_cmd.output() is Some, it saves the GRC.

    let grc_path = get_temp_path("output_opt.grc");
    let grc_path_str = grc_path.to_str().unwrap();

    let cmd = format!(
        "csdr --output {} load_u8 input.u8 ! convert_u8_f ! dump_f",
        grc_path_str
    );

    // We need to simulate what main.rs does
    use fsdr_cli::cmd_grammar::CommandsParser;
    use fsdr_cli::cmd_line::HighLevelCmdLine;
    use fsdr_cli::csdr_cmd::CsdrCmd;

    let input = CommandsParser::parse_main(&cmd)?;
    let csdr_cmd = input.as_csdr_cmd().expect("Should be a csdr cmd");

    let output_file = csdr_cmd.output()?.expect("Should have output file");
    assert_eq!(output_file, grc_path_str);

    let grc = csdr_cmd.parse()?.expect("Should have GRC");
    GrcParser::save(output_file, &grc)?;

    assert!(grc_path.exists());
    let loaded_grc = GrcParser::load(&grc_path)?;
    assert_eq!(grc, loaded_grc);

    let _ = fs::remove_file(grc_path);
    Ok(())
}
