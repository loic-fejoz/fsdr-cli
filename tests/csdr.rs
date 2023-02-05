use fsdr_cli::csdr::CsdrParser;

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
