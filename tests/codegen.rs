use fsdr_cli::csdr_cmd::CsdrParser;
use fsdr_cli::grc::converter::convert_grc_to_rust;

#[test]
fn test_codegen_basic() {
    let cmds = "csdr load_f input.bin | gain_ff 2.5 | dump_f";
    let grc = CsdrParser::parse_multiple_commands(cmds).unwrap().unwrap();
    let rust_code = convert_grc_to_rust(grc).unwrap();

    println!("Generated code:\n{}", rust_code);

    assert!(rust_code.contains("futuresdr"));
    assert!(rust_code.contains("Flowgraph :: new ()"));
    // The gain_ff 2.5 should result in an add_multiply_const_f32 call
    // which should have baked in the 2.5 value.
    assert!(rust_code.contains("2.5f32"));
}

#[test]
fn test_codegen_fir() {
    // fir_decimate_cc uses build_fir_filter_ccc which has #[fsdr_instantiate]
    let cmds = "csdr fir_decimate_cc 5";
    let grc = CsdrParser::parse_multiple_commands(cmds).unwrap().unwrap();
    let rust_code = convert_grc_to_rust(grc).unwrap();

    println!("Generated code:\n{}", rust_code);

    assert!(rust_code.contains("vec ! [")); // Taps baked in
    assert!(rust_code.contains("5usize")); // Decimation baked in
    assert!(rust_code.contains("FirBuilder :: resampling_with_taps"));
}

#[test]
fn test_codegen_complex_chain() {
    let cmds = "csdr load_f input.bin | agc_ff --reference 0.5 | fir_decimate_cc 2 | dump_c";
    let grc = CsdrParser::parse_multiple_commands(cmds).unwrap().unwrap();
    let rust_code = convert_grc_to_rust(grc).unwrap();

    println!("Generated code:\n{}", rust_code);

    // Check for multiple blocks and connections
    assert!(rust_code.contains("blk_0"));
    assert!(rust_code.contains("blk_1"));
    assert!(rust_code.contains("blk_2"));
    assert!(rust_code.contains("fg . connect_dyn"));
}
