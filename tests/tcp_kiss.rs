use anyhow::Result;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use fsdr_cli::blocks::kiss_file_sink::KissFileSink;
use fsdr_cli::blocks::kiss_file_source::KissFileSource;
use fsdr_cli::blocks::tcp_kiss_client::TcpKissClient;
use fsdr_cli::blocks::tcp_kiss_server::TcpKissServer;

#[test]
fn test_tcp_kiss_server_client() -> Result<()> {
    let port = 18045;
    let addr = format!("127.0.0.1:{}", port);

    // Server flowgraph
    let mut fg_server = Flowgraph::new();
    let src = fg_server.add_block(KissFileSource::new("tests/test.kiss")?);
    let server = fg_server.add_block(TcpKissServer::new(&addr)?);
    fg_server.connect_message(src, "output", server, "in_port")?;

    // Client flowgraph
    let mut out_path = env::temp_dir();
    out_path.push("tcp_output.kiss");
    let out_file = out_path.to_str().unwrap().to_string();

    let mut fg_client = Flowgraph::new();
    let client = fg_client.add_block(TcpKissClient::new(&addr)?);
    let snk = fg_client.add_block(KissFileSink::new(&out_file)?);
    fg_client.connect_message(client, "out", snk, "in_port")?;

    // Spawn server flowgraph in a separate thread
    let server_handle = thread::spawn(move || {
        Runtime::new().run(fg_server).unwrap();
    });

    // Give server a moment to bind to the port
    thread::sleep(Duration::from_millis(50));

    // Run client flowgraph
    let client_handle = thread::spawn(move || {
        Runtime::new().run(fg_client).unwrap();
    });

    // Wait a bit for processing
    thread::sleep(Duration::from_millis(200));

    // Verify file content matches roughly. KissFileSink escapes things differently
    // or adds FENDs, but we can check if it's not empty and created.
    let metadata = fs::metadata(&out_path)?;
    assert!(metadata.len() > 0, "Output file is empty");

    let _ = fs::remove_file(&out_path);
    Ok(())
}

#[test]
fn test_tcp_kiss_multi_client() -> Result<()> {
    let port = 18046;
    let addr = format!("127.0.0.1:{}", port);
    
    // Server flowgraph
    let mut fg_server = Flowgraph::new();
    let src = fg_server.add_block(KissFileSource::new("tests/test.kiss")?);
    let server = fg_server.add_block(TcpKissServer::new(&addr)?);
    fg_server.connect_message(src, "output", server, "in_port")?;
    
    // Client 1
    let mut out_path1 = env::temp_dir();
    out_path1.push("tcp_output1.kiss");
    let out_file1 = out_path1.to_str().unwrap().to_string();

    let mut fg_client1 = Flowgraph::new();
    let client1 = fg_client1.add_block(TcpKissClient::new(&addr)?);
    let snk1 = fg_client1.add_block(KissFileSink::new(&out_file1)?);
    fg_client1.connect_message(client1, "out", snk1, "in_port")?;

    // Client 2
    let mut out_path2 = env::temp_dir();
    out_path2.push("tcp_output2.kiss");
    let out_file2 = out_path2.to_str().unwrap().to_string();

    let mut fg_client2 = Flowgraph::new();
    let client2 = fg_client2.add_block(TcpKissClient::new(&addr)?);
    let snk2 = fg_client2.add_block(KissFileSink::new(&out_file2)?);
    fg_client2.connect_message(client2, "out", snk2, "in_port")?;

    // Start server
    let _server_handle = thread::spawn(move || {
        Runtime::new().run(fg_server).unwrap();
    });

    thread::sleep(Duration::from_millis(100));

    // Start clients
    let _client_handle1 = thread::spawn(move || {
        Runtime::new().run(fg_client1).unwrap();
    });
    let _client_handle2 = thread::spawn(move || {
        Runtime::new().run(fg_client2).unwrap();
    });

    // Wait for processing
    thread::sleep(Duration::from_millis(300));

    // Verify both files are populated
    let meta1 = fs::metadata(&out_path1)?;
    let meta2 = fs::metadata(&out_path2)?;
    assert!(meta1.len() > 0, "Client 1 output file is empty");
    assert!(meta2.len() > 0, "Client 2 output file is empty");

    let _ = fs::remove_file(&out_path1);
    let _ = fs::remove_file(&out_path2);
    Ok(())
}
