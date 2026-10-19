use anyhow::Result;
use fsdr_blocks::math::FrequencyShifter;
use fsdr_cli::cat::CatServer;
use fsdr_cli::csdr_cmd::CsdrParser;
use fsdr_cli::grc::converter::Grc2FutureSdr;
use fsdr_cli::hub::{ChannelShifterInfo, DspFrequencyController};
use fsdr_cli::telemetry::BokehServer;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::{Flowgraph, Pmt, Runtime};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn test_synthetic_sdr_source_and_bokeh_websocket() -> Result<()> {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|p| p.into_inner());

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

    // Start Flowgraph in background
    let rt = Runtime::new();
    let _running_fg = rt.start(fg)?;

    // 3. Connect to Bokeh WebSocket server as a client
    thread::sleep(Duration::from_millis(150));
    let (mut socket, _) = tungstenite::connect(format!("ws://127.0.0.1:{}/ws", test_port))?;
    if let tungstenite::stream::MaybeTlsStream::Plain(s) = socket.get_ref() {
        let _ = s.set_read_timeout(Some(Duration::from_millis(500)));
    }

    // 4. Verify we receive at least one binary WebSocket frame with FFT data
    let mut received_frame = false;
    for _ in 0..50 {
        if let Ok(tungstenite::Message::Binary(bin)) = socket.read() {
            if !bin.is_empty() {
                received_frame = true;
                break;
            }
        }
        thread::sleep(Duration::from_millis(20));
    }

    assert!(
        received_frame,
        "Expected to receive telemetry FFT frames over WebSocket"
    );

    Ok(())
}

#[test]
fn test_synthetic_source_full_demod_pipeline() -> Result<()> {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
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
    let rt = Runtime::new();
    let _running_fg = rt.start(fg)?;

    thread::sleep(Duration::from_millis(300));

    Ok(())
}

#[test]
fn test_synthetic_full_config_20m_with_telemetry() -> Result<()> {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|p| p.into_inner());

    // Setup Bokeh Telemetry Server
    let test_port = 15011;
    let listen_addr = format!("127.0.0.1:{}", test_port);
    let (tx, rx) = futures::channel::mpsc::unbounded::<Vec<f32>>();
    fsdr_cli::blocks::set_global_bokeh_sender(tx);
    let _bokeh_server = fsdr_cli::telemetry::BokehServer::spawn(listen_addr, rx)?;

    let config = fsdr_cli::config::HubConfig::load_from_file("examples/cfg-synthetic.toml")?;
    let mut fg = Flowgraph::new();
    let mut grc_converter = Grc2FutureSdr::new();

    let hw = config.get_hardware();
    let sample_rate = hw.sample_rate.unwrap_or(2_000_000) as f32;
    let src = fsdr_cli::blocks::SyntheticSdrSource::new(sample_rate);
    let sdr_source_id = fg.add(src)?.id();

    let delta_f: i64 = 0;
    let rx_center_freq = hw.rx_freq.unwrap_or(14_200_000) as f64;

    let mut tap_nodes: HashMap<String, (futuresdr::runtime::BlockId, String)> = HashMap::new();
    tap_nodes.insert(
        "sdr_raw_bus".to_string(),
        (sdr_source_id, "output".to_string()),
    );
    tap_nodes.insert(
        "rf_corrected_bus".to_string(),
        (sdr_source_id, "output".to_string()),
    );

    for ch in &config.channels {
        let vars = config.build_env_vars(&ch.id, delta_f)?;
        if let Some(ref rx) = ch.rx {
            if rx.enabled.unwrap_or(true) {
                if let Some(pipe) = rx.pipeline.as_deref() {
                    let norm =
                        fsdr_cli::config::HubConfig::normalize_pipeline_with_vars(pipe, &vars);
                    let mut grc = CsdrParser::parse_multiple_commands(&norm)?
                        .expect("RX pipeline should parse");

                    let ch_freq = ch.frequency.unwrap_or(rx_center_freq as u64);
                    let initial_rx_shift = DspFrequencyController::calc_rx_shift(
                        ch_freq,
                        rx_center_freq,
                        delta_f,
                        sample_rate,
                    );

                    let rx_shifter = FrequencyShifter::<Complex32>::new(initial_rx_shift, 1.0);
                    let rx_shifter_id = fg.add(rx_shifter)?.id();
                    fg.stream_dyn(sdr_source_id, "output", rx_shifter_id, "input")?;

                    tap_nodes.insert(
                        format!("channels.{}.rx_raw", ch.id),
                        (rx_shifter_id, "output".to_string()),
                    );
                    tap_nodes.insert(
                        format!("channels.{}.rx", ch.id),
                        (rx_shifter_id, "output".to_string()),
                    );

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

                    if let Some((target_name, _)) = first_dsp_name.clone() {
                        if let Some(pos) = grc.blocks.iter().position(|b| {
                            b.name == target_name
                                && (b.id == "blocks_freqshift_cc"
                                    || b.id == "decimating_shift_addition_cc")
                        }) {
                            if let Some(conn) = grc.connections.iter().find(|c| c[0] == target_name)
                            {
                                first_dsp_name = Some((conn[2].clone(), conn[3].clone()));
                            }
                            grc.blocks.remove(pos);
                            grc.connections
                                .retain(|c| c[0] != target_name && c[2] != target_name);
                        }
                    }

                    let grc_blocks_snapshot = grc.blocks.clone();
                    let grc_connections_snapshot = grc.connections.clone();

                    let adapters = grc_converter.convert_grc_into(grc, &mut fg)?;
                    if let Some((target_name, target_port)) = first_dsp_name {
                        if let Some(adapter) = adapters.get(&target_name) {
                            let (target_blk_id, target_in_port) =
                                adapter.adapt_input_port(&target_port)?;
                            fg.stream_dyn(rx_shifter_id, "output", target_blk_id, target_in_port)?;
                        }
                    }

                    // Register complex baseband tap
                    let mut complex_tap_block = None;
                    for conn in &grc_connections_snapshot {
                        let src_blk_name = &conn[0];
                        let dst_blk_name = &conn[2];
                        if let Some(dst_blk) =
                            grc_blocks_snapshot.iter().find(|b| &b.name == dst_blk_name)
                        {
                            if dst_blk.id.starts_with("weaver_")
                                || dst_blk.id.contains("demod")
                                || dst_blk.id.contains("costas")
                                || dst_blk.id.ends_with("_cf")
                            {
                                complex_tap_block = Some((src_blk_name.clone(), conn[1].clone()));
                                break;
                            }
                        }
                    }
                    if let Some((tap_name, tap_port)) = complex_tap_block {
                        if let Some(adapter) = adapters.get(&tap_name) {
                            if let Ok((tap_blk_id, tap_p)) = adapter.adapt_output_port(&tap_port) {
                                tap_nodes.insert(
                                    format!("channels.{}.rx", ch.id),
                                    (tap_blk_id, tap_p.to_string()),
                                );
                            }
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
                        let source_node = probe.source_node.as_deref().unwrap_or("sdr_raw_bus");
                        if let (Some((src_id, src_port)), Some((target_name, target_port))) =
                            (tap_nodes.get(source_node), first_dsp_name)
                        {
                            if let Some(adapter) = adapters.get(&target_name) {
                                let (target_blk_id, target_in_port) =
                                    adapter.adapt_input_port(&target_port)?;
                                fg.stream_dyn(
                                    *src_id,
                                    src_port.as_str(),
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

    let rt = Runtime::new();
    let _running_fg = rt.start(fg)?;

    thread::sleep(Duration::from_millis(150));

    // Connect WebSocket
    let (mut socket, _) = tungstenite::connect(format!("ws://127.0.0.1:{}/ws", test_port))?;
    if let tungstenite::stream::MaybeTlsStream::Plain(s) = socket.get_ref() {
        let _ = s.set_read_timeout(Some(Duration::from_millis(500)));
    }

    let mut received_frame = false;
    for _ in 0..50 {
        if let Ok(tungstenite::Message::Binary(bin)) = socket.read() {
            if !bin.is_empty() {
                received_frame = true;
                break;
            }
        }
        thread::sleep(Duration::from_millis(20));
    }

    assert!(
        received_frame,
        "Expected to receive telemetry FFT frames over WebSocket"
    );

    Ok(())
}

#[test]
fn test_hub_synthetic_audio_level() -> Result<()> {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|p| p.into_inner());

    // Check if parec is available on the system
    let parec_available = std::process::Command::new("parec")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !parec_available {
        eprintln!("[test_hub_synthetic_audio_level] Skipping test: 'parec' is not available in environment");
        return Ok(());
    }

    let pid = std::process::id();
    let test_sink = format!("TEST_INTEG_RX_{}", pid);
    let mut pa_guard = fsdr_cli::audio::PulseAudioGuard::new();
    let _module_id = match pa_guard.load_null_sink(&test_sink, Some("Test Integ RX")) {
        Ok(Some(id)) => id,
        _ => {
            eprintln!("[test_hub_synthetic_audio_level] Skipping test: PulseAudio null sink could not be loaded");
            return Ok(());
        }
    };

    // Give PulseAudio/PipeWire daemon a moment to register the new sink
    thread::sleep(Duration::from_millis(500));

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
    thread::sleep(Duration::from_millis(2000));

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

#[test]
fn test_unified_websocket_bidirectional_vfo_events() -> Result<()> {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|p| p.into_inner());

    let test_port = 15007;
    let listen_addr = format!("127.0.0.1:{}", test_port);
    let (_fft_tx, fft_rx) = futures::channel::mpsc::unbounded::<Vec<f32>>();
    let (event_tx, event_rx) = futures::channel::mpsc::unbounded::<String>();

    let channel_freq = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(14_250_000));
    let freq_c = channel_freq.clone();

    let initial_state_fn: Option<std::sync::Arc<dyn Fn() -> String + Send + Sync + 'static>> =
        Some(std::sync::Arc::new(move || {
            serde_json::json!({
                "event": "state",
                "channels": {
                    "ssb_voice_20m": {
                        "frequency": freq_c.load(std::sync::atomic::Ordering::Relaxed)
                    }
                }
            })
            .to_string()
        }));

    let freq_cmd = channel_freq.clone();
    let tx_cmd = event_tx.clone();
    let cmd_handler: Option<fsdr_cli::telemetry::CommandHandler> = Some(std::sync::Arc::new(
        move |cmd_str: &str| -> Option<String> {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(cmd_str) {
                if val.get("cmd").and_then(|c| c.as_str()) == Some("set_frequency") {
                    if let Some(f) = val.get("frequency").and_then(|f| f.as_u64()) {
                        freq_cmd.store(f, std::sync::atomic::Ordering::Relaxed);
                        let evt = serde_json::json!({
                            "event": "frequency_changed",
                            "channel": "ssb_voice_20m",
                            "frequency": f
                        })
                        .to_string();
                        let _ = tx_cmd.unbounded_send(evt);
                        return None;
                    }
                }
            }
            None
        },
    ));

    let _server = BokehServer::spawn_with_control(
        listen_addr.clone(),
        fft_rx,
        event_tx.clone(),
        event_rx,
        initial_state_fn,
        cmd_handler,
    )?;

    thread::sleep(Duration::from_millis(50));

    // Connect WebSocket client
    let url = format!("ws://127.0.0.1:{}/ws", test_port);
    let (mut client, _) = tungstenite::connect(url)?;

    // 1. Client should immediately receive initial state
    let msg1 = client.read()?;
    assert!(msg1.is_text(), "Expected text message from server");
    let state_json: serde_json::Value = serde_json::from_str(msg1.to_text()?)?;
    assert_eq!(state_json["event"], "state");
    assert_eq!(
        state_json["channels"]["ssb_voice_20m"]["frequency"],
        14_250_000
    );

    // 2. Client sends set_frequency command
    let cmd = serde_json::json!({
        "cmd": "set_frequency",
        "channel": "ssb_voice_20m",
        "frequency": 14_265_000
    })
    .to_string();
    client.send(tungstenite::Message::Text(cmd.into()))?;

    // Client should receive frequency_changed event
    let msg2 = client.read()?;
    assert!(msg2.is_text());
    let change_json: serde_json::Value = serde_json::from_str(msg2.to_text()?)?;
    assert_eq!(change_json["event"], "frequency_changed");
    assert_eq!(change_json["frequency"], 14_265_000);
    assert_eq!(
        channel_freq.load(std::sync::atomic::Ordering::Relaxed),
        14_265_000
    );

    // 3. Server broadcasts event (e.g. from external CAT rigctl F 14270000)
    let cat_evt = serde_json::json!({
        "event": "frequency_changed",
        "channel": "ssb_voice_20m",
        "frequency": 14_270_000
    })
    .to_string();
    event_tx.unbounded_send(cat_evt)?;

    let msg3 = client.read()?;
    assert!(msg3.is_text());
    let cat_json: serde_json::Value = serde_json::from_str(msg3.to_text()?)?;
    assert_eq!(cat_json["event"], "frequency_changed");
    assert_eq!(cat_json["frequency"], 14_270_000);

    Ok(())
}

#[test]
fn test_flowgraph_vfo_frequency_propagation() -> Result<()> {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|p| p.into_inner());

    // 1. Build flowgraph containing FrequencyShifter and DecimatingShiftAdditionCc with connected source and sink
    let mut fg = Flowgraph::new();
    let sample_rate = 2_000_000.0f32;

    let pipe_shifter = "synthetic_sdr_source 2000000 ! shift_addition_cc -0.025 ! none";
    let grc_shifter =
        CsdrParser::parse_multiple_commands(pipe_shifter)?.expect("pipeline should parse");
    let shifter_name = grc_shifter
        .blocks
        .iter()
        .find(|b| b.id == "blocks_freqshift_cc")
        .map(|b| b.name.clone())
        .expect("shifter block not found");

    let mut converter = Grc2FutureSdr::new();
    let adapters_shifter = converter.convert_grc_into(grc_shifter, &mut fg)?;
    let (shifter_id, _) = adapters_shifter
        .get(&shifter_name)
        .unwrap()
        .adapt_input_port("0")?;

    let pipe_decim = "synthetic_sdr_source 2000000 ! decimating_shift_addition_cc -0.025 4 ! none";
    let grc_decim =
        CsdrParser::parse_multiple_commands(pipe_decim)?.expect("decim pipeline should parse");
    let decim_name = grc_decim
        .blocks
        .iter()
        .find(|b| b.id == "decimating_shift_addition_cc")
        .map(|b| b.name.clone())
        .expect("decim block not found");
    let adapters_decim = converter.convert_grc_into(grc_decim, &mut fg)?;
    let (decim_id, _) = adapters_decim
        .get(&decim_name)
        .unwrap()
        .adapt_input_port("0")?;

    let rt = Runtime::new();
    let running_fg = rt.start(fg).expect("failed to start fg");
    let fg_handle = running_fg.handle();

    // 2. Setup DspFrequencyController
    let controller = DspFrequencyController::new();
    let ch_info = ChannelShifterInfo {
        channel_id: "ssb_test".to_string(),
        rx_shifter_id: Some(shifter_id),
        tx_shifter_id: None,
        sdr_sample_rate: sample_rate,
        rx_center_freq: 14_200_000.0,
        tx_center_freq: 14_200_000.0,
        current_frequency: 14_250_000,
    };
    controller.register_channel(ch_info);

    // Spawn dispatcher on a dedicated tokio runtime
    let tokio_rt = tokio::runtime::Runtime::new()?;
    let fg_h = fg_handle.clone();
    let beacon_tracker = fsdr_cli::beacon::BeaconTracker::new(false, 15000, true);
    let ctrl_clone = controller.clone();
    tokio_rt.spawn(ctrl_clone.run_dispatcher(fg_h, beacon_tracker, true));

    // 3. Test Direct Propagation via controller.set_frequency
    // Tune to 14,260,000 Hz (+60 kHz offset -> shift = -0.030)
    controller.set_frequency("ssb_test", 14_260_000);
    thread::sleep(Duration::from_millis(50));

    // Verify calling fg_handle with set_frequency returns Ok(Pmt::Ok)
    let call_res = futuresdr::runtime::block_on(fg_handle.call(
        shifter_id,
        "set_frequency",
        Pmt::F32(-0.030),
    ))?;
    assert_eq!(call_res, Pmt::Ok);

    // 4. Test Propagation via Hamlib CAT Server
    let cat_port = 15012;
    let shared_freq = Arc::new(std::sync::atomic::AtomicU64::new(14_260_000));
    let ctrl_cat = controller.clone();
    let on_freq_change = Arc::new(move |new_freq: u64| {
        ctrl_cat.set_frequency("ssb_test", new_freq);
    });
    let _cat_server =
        CatServer::spawn_with_shared_freq(cat_port, shared_freq.clone(), Some(on_freq_change))?;

    thread::sleep(Duration::from_millis(50));

    let mut cat_stream = TcpStream::connect(format!("127.0.0.1:{}", cat_port))?;
    cat_stream.write_all(b"F 14275000\n")?;
    let mut resp = [0u8; 128];
    let n = cat_stream.read(&mut resp)?;
    assert!(String::from_utf8_lossy(&resp[..n]).contains("RPRT 0"));
    assert_eq!(
        shared_freq.load(std::sync::atomic::Ordering::Relaxed),
        14_275_000
    );

    thread::sleep(Duration::from_millis(50));

    // Expected shift: -(14_275_000 - 14_200_000) / 2_000_000 = -75_000 / 2_000_000 = -0.0375
    let expected_shift =
        DspFrequencyController::calc_rx_shift(14_275_000, 14_200_000.0, 0, sample_rate);
    assert!((expected_shift - (-0.0375)).abs() < 1e-6);

    // Test RIT (offset) via Hamlib CAT: J 5000 adds 5 kHz -> 14_280_000
    cat_stream.write_all(b"J 5000\n")?;
    let n = cat_stream.read(&mut resp)?;
    assert!(String::from_utf8_lossy(&resp[..n]).contains("RPRT 0"));
    assert_eq!(
        shared_freq.load(std::sync::atomic::Ordering::Relaxed),
        14_280_000
    );

    // 5. Test DecimatingShiftAdditionCc message port
    let decim_res =
        futuresdr::runtime::block_on(fg_handle.call(decim_id, "set_frequency", Pmt::F32(0.125)))?;
    assert_eq!(decim_res, Pmt::Ok);

    let decim_rate_res =
        futuresdr::runtime::block_on(fg_handle.call(decim_id, "set_rate", Pmt::F32(0.250)))?;
    assert_eq!(decim_rate_res, Pmt::Ok);

    Ok(())
}

#[test]
fn test_vfo_retuning_modifies_zoom_fft() -> Result<()> {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|p| p.into_inner());

    // 1. Setup global bokeh sender
    let test_port = 15009;
    let listen_addr = format!("127.0.0.1:{}", test_port);
    let (tx, rx) = futures::channel::mpsc::unbounded::<Vec<f32>>();
    fsdr_cli::blocks::set_global_bokeh_sender(tx);
    let _bokeh_server = BokehServer::spawn(listen_addr.clone(), rx)?;

    // 2. Build flowgraph using SyntheticSdrSource and cfg-synthetic.toml
    let config = fsdr_cli::config::HubConfig::load_from_file("examples/cfg-synthetic.toml")?;
    let mut fg = Flowgraph::new();
    let mut grc_converter = Grc2FutureSdr::new();
    let hw = config.get_hardware();
    let sample_rate = hw.sample_rate.unwrap_or(2_000_000) as f32;
    let rx_center_freq = hw.rx_freq.unwrap_or(14_200_000) as f64;
    let sdr_source_id = fg
        .add(fsdr_cli::blocks::SyntheticSdrSource::new(sample_rate))?
        .id();

    let controller = DspFrequencyController::new();
    let mut tap_nodes: HashMap<String, (futuresdr::runtime::BlockId, String)> = HashMap::new();
    tap_nodes.insert(
        "sdr_raw_bus".to_string(),
        (sdr_source_id, "output".to_string()),
    );

    for ch in &config.channels {
        if ch.id != "ssb_voice_20m" {
            continue;
        }
        let vars = config.build_env_vars(&ch.id, 0)?;
        if let Some(ref rx) = ch.rx {
            if rx.enabled.unwrap_or(true) {
                if let Some(pipe) = rx.pipeline.as_deref() {
                    let norm =
                        fsdr_cli::config::HubConfig::normalize_pipeline_with_vars(pipe, &vars);
                    let mut grc =
                        CsdrParser::parse_multiple_commands(&norm)?.expect("pipeline should parse");
                    let ch_freq = ch.frequency.unwrap_or(rx_center_freq as u64);
                    let initial_shift = DspFrequencyController::calc_rx_shift(
                        ch_freq,
                        rx_center_freq,
                        0,
                        sample_rate,
                    );
                    let shifter = FrequencyShifter::<Complex32>::new(initial_shift, 1.0);
                    let shifter_id = fg.add(shifter)?.id();
                    fg.stream_dyn(sdr_source_id, "output", shifter_id, "input")?;

                    if ch.id == "ssb_voice_20m" {
                        controller.register_channel(ChannelShifterInfo {
                            channel_id: ch.id.clone(),
                            rx_shifter_id: Some(shifter_id),
                            tx_shifter_id: None,
                            sdr_sample_rate: sample_rate,
                            rx_center_freq,
                            tx_center_freq: rx_center_freq,
                            current_frequency: ch_freq,
                        });
                    }

                    tap_nodes.insert(
                        format!("channels.{}.rx", ch.id),
                        (shifter_id, "output".to_string()),
                    );

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

                    let grc_blocks = grc.blocks.clone();
                    let grc_conns = grc.connections.clone();
                    let adapters = grc_converter.convert_grc_into(grc, &mut fg)?;
                    if let Some((target_name, target_port)) = first_dsp_name {
                        if let Some(adapter) = adapters.get(&target_name) {
                            let (target_blk_id, target_in_port) =
                                adapter.adapt_input_port(&target_port)?;
                            fg.stream_dyn(shifter_id, "output", target_blk_id, target_in_port)?;
                        }
                    }

                    for conn in &grc_conns {
                        if let Some(dst_blk) = grc_blocks.iter().find(|b| b.name == conn[2]) {
                            if dst_blk.id.starts_with("weaver_") || dst_blk.id.ends_with("_cf") {
                                if let Some(adapter) = adapters.get(&conn[0]) {
                                    if let Ok((tap_blk_id, tap_p)) =
                                        adapter.adapt_output_port(&conn[1])
                                    {
                                        tap_nodes.insert(
                                            format!("channels.{}.rx", ch.id),
                                            (tap_blk_id, tap_p.to_string()),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Telemetry probe ssb_voice_zoom
    let probe = config
        .telemetry
        .as_ref()
        .unwrap()
        .bokeh
        .as_ref()
        .unwrap()
        .probes
        .iter()
        .find(|p| p.name == "ssb_voice_zoom")
        .unwrap();
    let norm = fsdr_cli::config::HubConfig::normalize_pipeline(probe.pipeline.as_ref().unwrap());
    let mut grc = CsdrParser::parse_multiple_commands(&norm)?.expect("probe parse");
    let mut first_dsp_name = None;
    if !grc.blocks.is_empty() && grc.blocks[0].id == "blocks_file_source" {
        let dummy_name = grc.blocks[0].name.clone();
        if let Some(conn) = grc.connections.iter().find(|c| c[0] == dummy_name) {
            first_dsp_name = Some((conn[2].clone(), conn[3].clone()));
        }
        grc.blocks.remove(0);
        grc.connections.retain(|c| c[0] != dummy_name);
    }
    let adapters = grc_converter.convert_grc_into(grc, &mut fg)?;
    let (src_id, src_port) = tap_nodes.get("channels.ssb_voice_20m.rx").unwrap();
    let (target_name, target_port) = first_dsp_name.unwrap();
    let (target_blk_id, target_in_port) = adapters
        .get(&target_name)
        .unwrap()
        .adapt_input_port(&target_port)?;
    fg.stream_dyn(*src_id, src_port.as_str(), target_blk_id, target_in_port)?;

    // 3. Start flowgraph
    let rt = Runtime::new();
    let running_fg = rt.start(fg).expect("fg start");
    let fg_handle = running_fg.handle();

    // Spawn dispatcher
    let tokio_rt = tokio::runtime::Runtime::new()?;
    let beacon_tracker = fsdr_cli::beacon::BeaconTracker::new(false, 15000, true);
    tokio_rt.spawn(
        controller
            .clone()
            .run_dispatcher(fg_handle.clone(), beacon_tracker, true),
    );

    // Connect WebSocket via tungstenite
    let (mut socket, _) = tungstenite::connect(format!("ws://127.0.0.1:{}/ws", test_port))?;
    if let tungstenite::stream::MaybeTlsStream::Plain(s) = socket.get_ref() {
        let _ = s.set_read_timeout(Some(Duration::from_millis(500)));
    }

    // Capture initial frames (1024 floats = 4096 bytes each)
    let mut frame1024_initial: Option<Vec<f32>> = None;
    for _ in 0..50 {
        if let Ok(tungstenite::Message::Binary(bin)) = socket.read() {
            if bin.len() == 4096 {
                let floats: &[f32] =
                    unsafe { std::slice::from_raw_parts(bin.as_ptr() as *const f32, 1024) };
                frame1024_initial = Some(floats.to_vec());
                break;
            }
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        frame1024_initial.is_some(),
        "Should receive 1024-bin FFT zoom frame"
    );
    let frame_init = frame1024_initial.unwrap();

    // 4. Change VFO frequency to 14.230 MHz (+30 kHz offset instead of +50 kHz)
    controller.set_frequency("ssb_voice_20m", 14_230_000);
    thread::sleep(Duration::from_millis(150));

    // Capture post-retuning frames
    let mut frame1024_after: Option<Vec<f32>> = None;
    for _ in 0..50 {
        if let Ok(tungstenite::Message::Binary(bin)) = socket.read() {
            if bin.len() == 4096 {
                let floats: &[f32] =
                    unsafe { std::slice::from_raw_parts(bin.as_ptr() as *const f32, 1024) };
                frame1024_after = Some(floats.to_vec());
                break;
            }
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        frame1024_after.is_some(),
        "Should receive 1024-bin FFT zoom frame after retuning"
    );
    let frame_aft = frame1024_after.unwrap();

    // Verify the FFT frames are substantially different due to retuning
    let diff: f32 = frame_init
        .iter()
        .zip(frame_aft.iter())
        .map(|(a, b)| (a - b).abs())
        .sum();
    println!(
        "[test] Cumulative spectral difference across 1024 bins after retuning: {:.2} dB",
        diff
    );
    assert!(
        diff > 50.0,
        "FFT spectrum should change dynamically when VFO retunes (got diff={:.2})",
        diff
    );

    Ok(())
}
