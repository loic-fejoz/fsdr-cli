use anyhow::Result;
use fsdr_cli::blocks::CatServer;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

#[test]
fn test_cat_server_tcp_commands() -> Result<()> {
    let port = 18532;
    let _server = CatServer::new(port, 144500000, 144500000, 885)?;

    // Give server a moment to start listening
    thread::sleep(Duration::from_millis(50));

    let stream = TcpStream::connect(format!("127.0.0.1:{}", port))?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream;
    let mut response = String::new();

    // Helper to send line and read line response
    let mut send_cmd = |cmd: &str| -> Result<String> {
        writer.write_all(cmd.as_bytes())?;
        response.clear();
        reader.read_line(&mut response)?;
        Ok(response.trim().to_string())
    };

    // Test initial getters
    assert_eq!(send_cmd("f\n")?, "144500000");
    assert_eq!(send_cmd("i\n")?, "144500000");
    assert_eq!(send_cmd("c\n")?, "885");
    assert_eq!(send_cmd("d\n")?, "23");
    assert_eq!(send_cmd("\\get_ctcss_sql\n")?, "885");
    assert_eq!(send_cmd("\\get_dcs_sql\n")?, "23");
    assert_eq!(send_cmd("v\n")?, "VFOA");
    assert_eq!(send_cmd("m\n")?, "FM 15000");

    // Test setters
    assert_eq!(send_cmd("F 145830000\n")?, "RPRT 0");
    assert_eq!(send_cmd("f\n")?, "145830000");

    assert_eq!(send_cmd("I 145900000\n")?, "RPRT 0");
    assert_eq!(send_cmd("i\n")?, "145900000");

    assert_eq!(send_cmd("C 1000\n")?, "RPRT 0");
    assert_eq!(send_cmd("c\n")?, "1000");

    assert_eq!(send_cmd("D 47\n")?, "RPRT 0");
    assert_eq!(send_cmd("d\n")?, "47");

    assert_eq!(send_cmd("\\set_ctcss_sql 1230\n")?, "RPRT 0");
    assert_eq!(send_cmd("\\get_ctcss_sql\n")?, "1230");

    assert_eq!(send_cmd("\\set_dcs_sql 54\n")?, "RPRT 0");
    assert_eq!(send_cmd("\\get_dcs_sql\n")?, "54");

    // Test error cases
    assert_eq!(send_cmd("F invalid_freq\n")?, "RPRT 1");
    assert_eq!(send_cmd("INVALID_CMD\n")?, "RPRT 1");

    Ok(())
}

use fsdr_cli::csdr_cmd::CsdrParser;

#[test]
fn test_cat_server_cli_parser() -> Result<()> {
    let cmd = "cat_server --port 4532 --rx_freq 145830000 --tx_freq 145830000 --ctcss_tone 885";
    let grc = CsdrParser::parse_command(cmd)?.expect("should parse GRC");
    assert_eq!(grc.blocks.len(), 1);
    assert_eq!(grc.blocks[0].id, "cat_server");
    assert_eq!(grc.blocks[0].parameter_or("port", ""), "4532");
    assert_eq!(grc.blocks[0].parameter_or("rx_freq", ""), "145830000");
    assert_eq!(grc.blocks[0].parameter_or("tx_freq", ""), "145830000");
    assert_eq!(grc.blocks[0].parameter_or("ctcss_tone", ""), "885");

    Ok(())
}
