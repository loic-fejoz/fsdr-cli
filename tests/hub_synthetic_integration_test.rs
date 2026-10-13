use anyhow::Result;
use fsdr_cli::csdr_cmd::CsdrParser;
use fsdr_cli::grc::converter::Grc2FutureSdr;
use fsdr_cli::telemetry::BokehServer;
use futuresdr::runtime::{Flowgraph, Runtime};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn test_synthetic_sdr_source_and_bokeh_websocket() -> Result<()> {
    let _guard = TEST_MUTEX.lock().unwrap();

    // 1. Setup Bokeh Telemetry Server on a dedicated test port
    let test_port = 15006;
    let listen_addr = format!("127.0.0.1:{}", test_port);
    let (tx, rx) = futures::channel::mpsc::unbounded::<Vec<f32>>();
    fsdr_cli::blocks::set_global_bokeh_sender(tx);
    let _bokeh_server = BokehServer::spawn(listen_addr.clone(), rx)?;

    thread::sleep(Duration::from_millis(50));

    // 2. Build flowgraph: synthetic_sdr_source -> FFT -> logpower -> bokeh_spectrum_sink
    let mut fg = Flowgraph::new();
    let pipe = "synthetic_sdr_source 2000000 ! fft_cc 1024 1024 HAMMING ! logpower_cf ! bokeh_spectrum_sink 1024 20";
    let grc = CsdrParser::parse_multiple_commands(pipe)?.expect("probe pipeline should parse");
    let mut converter = Grc2FutureSdr::new();
    converter.convert_grc_into(grc, &mut fg)?;

    // Start Flowgraph in background thread
    let _fg_handle = thread::spawn(move || {
        let rt = Runtime::new();
        let _ = rt.start(fg);
    });

    // 3. Connect to Bokeh WebSocket server as a client
    thread::sleep(Duration::from_millis(150));
    let mut stream = TcpStream::connect(&listen_addr)?;
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;

    // Perform WebSocket HTTP upgrade handshake
    let req = format!(
        "GET /ws HTTP/1.1\r\n\
         Host: 127.0.0.1:{}\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
         Sec-WebSocket-Version: 13\r\n\r\n",
        test_port
    );
    stream.write_all(req.as_bytes())?;

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf)?;
    let response = String::from_utf8_lossy(&buf[..n]);
    assert!(
        response.contains("101 Switching Protocols"),
        "WebSocket upgrade handshake failed: {}",
        response
    );

    // 4. Verify we receive at least one binary WebSocket frame with FFT data
    let mut frame_buf = vec![0u8; 8192];
    let mut total_read = 0;
    for _ in 0..10 {
        if let Ok(bytes) = stream.read(&mut frame_buf[total_read..]) {
            total_read += bytes;
            if total_read > 100 {
                break;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }

    assert!(
        total_read > 0,
        "Expected to receive telemetry FFT frames over WebSocket, received 0 bytes"
    );

    Ok(())
}

#[test]
fn test_synthetic_source_full_demod_pipeline() -> Result<()> {
    // Test that the synthetic source streams into multi-channel demodulation pipelines without errors
    let mut fg = Flowgraph::new();

    // 1. Full chain: Synthetic IQ -> SSB Weaver Demod -> AGC -> Limit -> None (Null sink)
    let ssb_pipe = "synthetic_sdr_source 2000000 ! shift_addition_cc 0.025 ! rational_resampler_cc 48000 2000000 ! weaver_usb_cf 0.03125 ! agc_ff ! limit_ff ! none";
    let grc_ssb =
        CsdrParser::parse_multiple_commands(ssb_pipe)?.expect("SSB pipeline should parse");

    // 2. Full chain: Synthetic IQ -> AFSK Demod -> HDLC Deframing
    let aprs_pipe = "synthetic_sdr_source 2000000 ! shift_addition_cc -0.0485 ! rational_resampler_cc 48000 2000000 ! weaver_lsb_cf 0.0354 ! afsk1200_demod_f_u8 48000 ! hdlc_deframing_u8_pdu";
    let grc_aprs =
        CsdrParser::parse_multiple_commands(aprs_pipe)?.expect("APRS pipeline should parse");

    // 3. Full chain: Synthetic IQ -> OQPSK Costas loop -> OQPSK Demod -> None (Null sink)
    let oqpsk_pipe = "synthetic_sdr_source 2000000 ! shift_addition_cc 0.075 ! rational_resampler_cc 48000 2000000 ! oqpsk_costas_loop_cc 0.05 ! oqpsk_demod_c_u8 ! none";
    let grc_oqpsk =
        CsdrParser::parse_multiple_commands(oqpsk_pipe)?.expect("OQPSK pipeline should parse");

    let mut converter = Grc2FutureSdr::new();
    converter.convert_grc_into(grc_ssb, &mut fg)?;
    converter.convert_grc_into(grc_aprs, &mut fg)?;
    converter.convert_grc_into(grc_oqpsk, &mut fg)?;

    // Run flowgraph for 300 ms to verify stability and processing throughput
    let fg_handle = thread::spawn(move || {
        let rt = Runtime::new();
        let _ = rt.start(fg);
    });

    thread::sleep(Duration::from_millis(300));
    drop(fg_handle);

    Ok(())
}

#[test]
fn test_synthetic_full_config_20m_with_telemetry() -> Result<()> {
    let _guard = TEST_MUTEX.lock().unwrap();

    let config = fsdr_cli::config::HubConfig::load_from_file("examples/cfg-synthetic.toml")?;
    let mut fg = Flowgraph::new();
    let mut grc_converter = Grc2FutureSdr::new();

    let hw = config.get_hardware();
    let sample_rate = hw.sample_rate.unwrap_or(2_000_000) as f32;
    let src = fsdr_cli::blocks::SyntheticSdrSource::new(sample_rate);
    let sdr_source_id = fg.add(src)?.id();

    let delta_f: i64 = 0;

    for ch in &config.channels {
        let vars = config.build_env_vars(&ch.id, delta_f)?;
        if let Some(ref rx) = ch.rx {
            if rx.enabled.unwrap_or(true) {
                if let Some(pipe) = rx.pipeline.as_deref() {
                    let norm =
                        fsdr_cli::config::HubConfig::normalize_pipeline_with_vars(pipe, &vars);
                    let mut grc = CsdrParser::parse_multiple_commands(&norm)?
                        .expect("RX pipeline should parse");

                    let mut first_dsp_name = None;
                    if !grc.blocks.is_empty()
                        && grc.blocks[0].id == "blocks_file_source"
                        && grc.blocks[0].parameter_or("file", "") == "-"
                    {
                        let dummy_name = grc.blocks[0].name.clone();
                        if let Some(conn) = grc.connections.iter().find(|c| c[0] == dummy_name) {
                            first_dsp_name = Some((conn[2].clone(), conn[3].clone()));
                        }
                        grc.blocks.remove(0);
                        grc.connections.retain(|c| c[0] != dummy_name);
                    }

                    let adapters = grc_converter.convert_grc_into(grc, &mut fg)?;
                    if let Some((target_name, target_port)) = first_dsp_name {
                        if let Some(adapter) = adapters.get(&target_name) {
                            let (target_blk_id, target_in_port) =
                                adapter.adapt_input_port(&target_port)?;
                            fg.stream_dyn(sdr_source_id, "output", target_blk_id, target_in_port)?;
                        }
                    }
                }
            }
        }

        if let Some(ref tx) = ch.tx {
            if tx.enabled.unwrap_or(true) {
                if let Some(pipe) = tx.pipeline.as_deref() {
                    let norm =
                        fsdr_cli::config::HubConfig::normalize_pipeline_with_vars(pipe, &vars);
                    let grc = CsdrParser::parse_multiple_commands(&norm)?
                        .expect("TX pipeline should parse");
                    grc_converter.convert_grc_into(grc, &mut fg)?;
                }
            }
        }
    }

    // Telemetry probes
    if let Some(ref tel) = config.telemetry {
        if let Some(ref bokeh_cfg) = tel.bokeh {
            for probe in &bokeh_cfg.probes {
                if let Some(pipe) = probe.pipeline.as_deref() {
                    let norm = fsdr_cli::config::HubConfig::normalize_pipeline(pipe);
                    if let Ok(Some(mut grc)) = CsdrParser::parse_multiple_commands(&norm) {
                        let mut first_dsp_name = None;
                        if !grc.blocks.is_empty()
                            && grc.blocks[0].id == "blocks_file_source"
                            && grc.blocks[0].parameter_or("file", "") == "-"
                        {
                            let dummy_name = grc.blocks[0].name.clone();
                            if let Some(conn) = grc.connections.iter().find(|c| c[0] == dummy_name)
                            {
                                first_dsp_name = Some((conn[2].clone(), conn[3].clone()));
                            }
                            grc.blocks.remove(0);
                            grc.connections.retain(|c| c[0] != dummy_name);
                        }
                        let adapters = grc_converter.convert_grc_into(grc, &mut fg)?;
                        if let Some((target_name, target_port)) = first_dsp_name {
                            if let Some(adapter) = adapters.get(&target_name) {
                                let (target_blk_id, target_in_port) =
                                    adapter.adapt_input_port(&target_port)?;
                                fg.stream_dyn(
                                    sdr_source_id,
                                    "output",
                                    target_blk_id,
                                    target_in_port,
                                )?;
                            }
                        }
                    }
                }
            }
        }
    }

    // Setup Bokeh Telemetry Server
    let test_port = 15008;
    let listen_addr = format!("127.0.0.1:{}", test_port);
    let (tx, rx) = futures::channel::mpsc::unbounded::<Vec<f32>>();
    fsdr_cli::blocks::set_global_bokeh_sender(tx);
    let _bokeh_server = fsdr_cli::telemetry::BokehServer::spawn(listen_addr.clone(), rx)?;

    let handle = thread::spawn(move || {
        let rt = Runtime::new();
        let _ = rt.start(fg);
    });

    thread::sleep(Duration::from_millis(200));

    // Connect WebSocket
    let mut stream = TcpStream::connect(&listen_addr)?;
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;

    let req = format!(
        "GET /ws HTTP/1.1\r\n\
         Host: 127.0.0.1:{}\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
         Sec-WebSocket-Version: 13\r\n\r\n",
        test_port
    );
    stream.write_all(req.as_bytes())?;

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf)?;
    let response = String::from_utf8_lossy(&buf[..n]);
    assert!(
        response.contains("101 Switching Protocols"),
        "WebSocket upgrade handshake failed: {}",
        response
    );

    let mut frame_buf = vec![0u8; 8192];
    let mut total_read = 0;
    for _ in 0..20 {
        if let Ok(bytes) = stream.read(&mut frame_buf[total_read..]) {
            total_read += bytes;
            if total_read > 500 {
                break;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }

    assert!(
        total_read > 0,
        "Expected to receive telemetry FFT frames over WebSocket, received 0 bytes"
    );

    drop(handle);
    Ok(())
}

#[test]
fn test_hub_synthetic_audio_level() -> Result<()> {
    let _guard = TEST_MUTEX.lock().unwrap();

    let test_sink = "TEST_INTEG_RX";
    let mut pa_guard = fsdr_cli::audio::PulseAudioGuard::new();
    let _ = pa_guard.load_null_sink(test_sink, Some("Test Integ RX"));

    let mut fg = Flowgraph::new();
    // f_ssb is at +50 kHz (+0.025 normalized frequency). Downconvert by -0.025 to baseband.
    let pipe = format!(
        "synthetic_sdr_source 2000000 ! shift_addition_cc -0.025 ! rational_resampler_cc 48000 2000000 ! weaver_usb_cf 0.03125 ! agc_ff ! limit_ff ! pulse_sink \"{}\"",
        test_sink
    );
    let grc = CsdrParser::parse_multiple_commands(&pipe)?.expect("pipeline should parse");
    let mut converter = Grc2FutureSdr::new();
    converter.convert_grc_into(grc, &mut fg)?;

    let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
    let handle = thread::spawn(move || {
        let rt = Runtime::new();
        let running_fg = rt.start(fg).expect("failed to start fg");
        let _ = stop_rx.recv();
        drop(running_fg);
        drop(rt);
    });

    // Give the runtime a brief moment to produce and stream audio samples to PulseAudio/PipeWire
    thread::sleep(Duration::from_millis(1200));

    // Run python audio level meter on the virtual monitor sink
    let monitor_name = format!("{}.monitor", test_sink);
    let status = std::process::Command::new("python3")
        .args(["examples/measure_audio_level.py", &monitor_name, "2.0"])
        .status();

    let _ = stop_tx.send(());
    let _ = handle.join();
    pa_guard.cleanup();

    if let Ok(s) = status {
        assert!(
            s.success(),
            "measure_audio_level.py failed to detect active audio"
        );
    }

    Ok(())
}
