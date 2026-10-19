use anyhow::{Context, Result};
use fsdr_blocks::math::FrequencyShifter;
use fsdr_cli::audio::PulseAudioGuard;
use fsdr_cli::beacon::BeaconTracker;
use fsdr_cli::cat::{CatClientProxy, CatServer};
use fsdr_cli::config::HubConfig;
use fsdr_cli::csdr_cmd::CsdrParser;
use fsdr_cli::grc::converter::Grc2FutureSdr;
use fsdr_cli::hub::{ChannelShifterInfo, DspFrequencyController};
use fsdr_cli::telemetry::BokehServer;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::{BlockId, Flowgraph, Runtime};
use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn print_usage() {
    println!(
        "Usage: fsdr-hub [OPTIONS]\n\n\
         Options:\n  \
         -c, --config <FILE>    Path to TOML configuration file [default: config.toml]\n  \
         -v, --verbose          Enable verbose flowgraph diagnostic logging\n  \
         -h, --help             Display this help message\n"
    );
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut config_path = "config.toml".to_string();
    let mut output_grc_path = None;
    let mut verbose = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-c" | "--config" => {
                if i + 1 < args.len() {
                    config_path = args[i + 1].clone();
                    i += 1;
                } else {
                    eprintln!("Error: --config requires a file path argument");
                    std::process::exit(1);
                }
            }
            "-o" | "--output" => {
                if i + 1 < args.len() {
                    output_grc_path = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "-v" | "--verbose" => {
                verbose = true;
            }
            "-h" | "--help" => {
                print_usage();
                return Ok(());
            }
            _ => {
                // If positional argument without flag, assume config file
                if !args[i].starts_with('-') {
                    config_path = args[i].clone();
                }
            }
        }
        i += 1;
    }

    println!("[fsdr-hub] Loading configuration from: {config_path}");
    if let Some(ref path) = output_grc_path {
        println!("[fsdr-hub] Target GRC pipeline output: {path}");
    }
    let config = HubConfig::load_from_file(&config_path)
        .with_context(|| format!("Failed to load configuration file '{config_path}'"))?;

    // 1. Initialize PulseAudio virtual audio devices via RAII guard with signal cleanup
    let pa_guard = Arc::new(std::sync::Mutex::new(PulseAudioGuard::setup_from_config(
        &config,
    )?));
    let pa_guard_sig = pa_guard.clone();

    // 2. Initialize Beacon Tracker
    let beacon_tracker = if let Some(ref beacon_cfg) = config.beacon_tracker {
        let enabled = beacon_cfg.enabled;
        let max_drift = beacon_cfg.max_drift_hz.unwrap_or(15_000.0) as i64;
        let holdover = beacon_cfg.holdover_on_loss.unwrap_or(true);
        BeaconTracker::new(enabled, max_drift, holdover)
    } else {
        BeaconTracker::new(false, 15_000, true)
    };

    // 3. Initialize CAT servers and client proxies with shared channel frequencies
    let hw = config.get_hardware();
    let sdr_sample_rate = hw.sample_rate.unwrap_or(2_000_000) as f32;
    let rx_center_freq = hw.rx_freq.unwrap_or(0) as f64;
    let tx_center_freq = hw.tx_freq.unwrap_or(0) as f64;

    let dsp_controller = DspFrequencyController::new();

    let (ws_event_tx, ws_event_rx) = futures::channel::mpsc::unbounded::<String>();
    let channels_freq_map = Arc::new(std::sync::RwLock::new(std::collections::HashMap::<
        String,
        Arc<std::sync::atomic::AtomicU64>,
    >::new()));

    let mut _cat_servers = Vec::new();
    let mut _cat_proxies = Vec::new();

    for ch in &config.channels {
        let initial_freq = ch
            .frequency
            .or(ch.frequency_rx)
            .or(ch.frequency_tx)
            .unwrap_or_else(|| {
                if let Some(off) = ch.frequency_offset {
                    (rx_center_freq as i64 + off).max(0) as u64
                } else {
                    144_000_000
                }
            });

        let freq_arc = Arc::new(std::sync::atomic::AtomicU64::new(initial_freq));
        channels_freq_map
            .write()
            .unwrap()
            .insert(ch.id.clone(), freq_arc.clone());

        dsp_controller.register_channel(ChannelShifterInfo {
            channel_id: ch.id.clone(),
            rx_shifter_id: None,
            tx_shifter_id: None,
            sdr_sample_rate,
            rx_center_freq,
            tx_center_freq,
            current_frequency: initial_freq,
        });

        let tx_cb = ws_event_tx.clone();
        let ch_id_cb = ch.id.clone();
        let dsp_cb = dsp_controller.clone();
        let on_freq_change = Arc::new(move |new_freq: u64| {
            let msg = serde_json::json!({
                "event": "frequency_changed",
                "channel": ch_id_cb,
                "frequency": new_freq
            })
            .to_string();
            let _ = tx_cb.unbounded_send(msg);
            dsp_cb.set_frequency(&ch_id_cb, new_freq);
        });

        if let Some(port) = ch.cat_port {
            let server = CatServer::spawn_with_shared_freq(
                port,
                freq_arc.clone(),
                Some(on_freq_change.clone()),
            )
            .with_context(|| {
                format!(
                    "Failed to spawn CAT server on port {} for channel '{}'",
                    port, ch.id
                )
            })?;
            println!(
                "[fsdr-hub] CAT server for channel '{}' running on port {}",
                ch.id, port
            );
            _cat_servers.push(server);
        } else if let Some(ref cat_rx) = ch.cat_rx {
            if let Some(port) = cat_rx.listen_port {
                let server =
                    CatServer::spawn_with_shared_freq(port, freq_arc.clone(), Some(on_freq_change))
                        .with_context(|| {
                            format!(
                                "Failed to spawn CAT RX server on port {} for channel '{}'",
                                port, ch.id
                            )
                        })?;
                println!(
                    "[fsdr-hub] CAT RX server for channel '{}' running on port {}",
                    ch.id, port
                );
                _cat_servers.push(server);
            }
        }

        if let Some(ref cat_tx) = ch.cat_tx {
            if let Some(ref target) = cat_tx.target {
                let lo_offset = cat_tx.lo_offset.unwrap_or(0);
                let lo_multiplier = cat_tx.lo_multiplier.unwrap_or(1) as f64;
                let proxy =
                    CatClientProxy::spawn(target.clone(), lo_offset, lo_multiplier, freq_arc);
                println!(
                    "[fsdr-hub] CAT TX proxy for channel '{}' connected to {}",
                    ch.id, target
                );
                _cat_proxies.push(proxy);
            }
        }
    }

    // 4. Initialize Telemetry (Bokeh) with bidirectional unified WebSocket
    let mut _bokeh_server = None;
    if let Some(ref tel) = config.telemetry {
        if let Some(ref bokeh_cfg) = tel.bokeh {
            let listen_addr = bokeh_cfg
                .listen_addr
                .clone()
                .unwrap_or_else(|| "127.0.0.1:5006".to_string());
            let (tx, rx) = futures::channel::mpsc::unbounded::<Vec<f32>>();
            fsdr_cli::blocks::set_global_bokeh_sender(tx);

            // Initial state generator for newly connected WebSocket clients
            let map_init = channels_freq_map.clone();
            let initial_state_fn: Option<Arc<dyn Fn() -> String + Send + Sync + 'static>> =
                Some(Arc::new(move || {
                    let mut map_json = serde_json::Map::new();
                    if let Ok(map) = map_init.read() {
                        for (id, freq_atom) in map.iter() {
                            map_json.insert(
                                id.clone(),
                                serde_json::json!({
                                    "frequency": freq_atom.load(Ordering::Relaxed)
                                }),
                            );
                        }
                    }
                    serde_json::json!({
                        "event": "state",
                        "channels": map_json
                    })
                    .to_string()
                }));

            // Command handler for incoming WebSocket client messages
            let map_cmd = channels_freq_map.clone();
            let event_tx_cmd = ws_event_tx.clone();
            let dsp_ws = dsp_controller.clone();
            let rx_center_ws = rx_center_freq;
            let cmd_handler: Option<fsdr_cli::telemetry::CommandHandler> =
                Some(Arc::new(move |cmd_str: &str| -> Option<String> {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(cmd_str) {
                        if let Some(cmd) = val.get("cmd").and_then(|c| c.as_str()) {
                            match cmd {
                                "set_frequency" => {
                                    if let Some(freq) =
                                        val.get("frequency").and_then(|f| f.as_u64())
                                    {
                                        let req_ch = val.get("channel").and_then(|c| c.as_str());
                                        if let Ok(map) = map_cmd.read() {
                                            let target = if let Some(ch_name) = req_ch {
                                                map.get(ch_name)
                                                    .map(|atom| (ch_name.to_string(), atom))
                                            } else {
                                                map.iter().next().map(|(k, v)| (k.clone(), v))
                                            };
                                            if let Some((target_id, target_atom)) = target {
                                                target_atom.store(freq, Ordering::Relaxed);
                                                let evt = serde_json::json!({
                                                    "event": "frequency_changed",
                                                    "channel": target_id,
                                                    "frequency": freq
                                                })
                                                .to_string();
                                                let _ = event_tx_cmd.unbounded_send(evt);
                                                dsp_ws.set_frequency(&target_id, freq);
                                                return None;
                                            }
                                        }
                                    }
                                }
                                "set_offset" | "set_frequency_offset" => {
                                    if let Some(offset) = val.get("offset").and_then(|o| o.as_i64())
                                    {
                                        let req_ch = val.get("channel").and_then(|c| c.as_str());
                                        if let Ok(map) = map_cmd.read() {
                                            let target = if let Some(ch_name) = req_ch {
                                                map.get(ch_name)
                                                    .map(|atom| (ch_name.to_string(), atom))
                                            } else {
                                                map.iter().next().map(|(k, v)| (k.clone(), v))
                                            };
                                            if let Some((target_id, target_atom)) = target {
                                                let new_freq =
                                                    (rx_center_ws as i64 + offset).max(0) as u64;
                                                target_atom.store(new_freq, Ordering::Relaxed);
                                                let evt = serde_json::json!({
                                                    "event": "frequency_changed",
                                                    "channel": target_id,
                                                    "frequency": new_freq
                                                })
                                                .to_string();
                                                let _ = event_tx_cmd.unbounded_send(evt);
                                                dsp_ws.set_offset(&target_id, offset);
                                                return None;
                                            }
                                        }
                                    }
                                }
                                "get_state" => {
                                    let mut map_json = serde_json::Map::new();
                                    if let Ok(map) = map_cmd.read() {
                                        for (id, freq_atom) in map.iter() {
                                            map_json.insert(
                                                id.clone(),
                                                serde_json::json!({
                                                    "frequency": freq_atom.load(Ordering::Relaxed)
                                                }),
                                            );
                                        }
                                    }
                                    return Some(
                                        serde_json::json!({
                                            "event": "state",
                                            "channels": map_json
                                        })
                                        .to_string(),
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                    None
                }));

            let server = BokehServer::spawn_with_control(
                listen_addr.clone(),
                rx,
                ws_event_tx,
                ws_event_rx,
                initial_state_fn,
                cmd_handler,
            )
            .with_context(|| {
                format!("Failed to spawn Bokeh telemetry server on {}", listen_addr)
            })?;
            println!(
                "[fsdr-hub] Bokeh unified WebSocket telemetry server running on {}",
                listen_addr
            );
            _bokeh_server = Some(server);
        }
    }

    // 5. Build Flowgraph from active channels
    let delta_f = beacon_tracker.delta_f();
    println!("[fsdr-hub] Initial beacon delta_f: {} Hz", delta_f);

    let mut active_fg = Flowgraph::new();
    let mut grc_converter = Grc2FutureSdr::new();
    let mut pipeline_count = 0;

    // Check hardware SDR device: if "synthetic" or "virtual", add SyntheticSdrSource block
    let hw = config.get_hardware();
    let is_synthetic = hw
        .device
        .as_deref()
        .map(|d| d == "synthetic" || d == "virtual")
        .unwrap_or(false);

    let (sdr_source_id, sdr_output_port) = if is_synthetic {
        let sample_rate = hw.sample_rate.unwrap_or(2_000_000) as f32;
        let src = fsdr_cli::blocks::SyntheticSdrSource::new(sample_rate);
        let id = active_fg.add(src)?.id();
        println!(
            "[fsdr-hub] Initialized Synthetic SDR Source at {} SPS (multi-carrier test signal)",
            sample_rate
        );
        (Some(id), "output")
    } else if let Some(ref dev_name) = hw.device {
        #[cfg(feature = "soapy")]
        {
            let sample_rate = hw.sample_rate.unwrap_or(2_000_000) as f64;
            let rx_freq = hw.rx_freq.unwrap_or(14_200_000) as f64;
            let seify_args = if let Some(ref uri) = hw.uri {
                if uri.contains('=') {
                    uri.clone()
                } else if let Some((driver, dev_id)) = uri.split_once(':') {
                    format!(
                        "driver=soapy,soapy:driver={},soapy:{}={}",
                        driver, driver, dev_id
                    )
                } else {
                    format!("driver=soapy,soapy:driver={}", uri)
                }
            } else {
                format!("driver=soapy,soapy:driver={}", dev_name)
            };

            println!(
                "[fsdr-hub] Connecting to hardware SDR device '{}' via Seify/Soapy (args: '{}')...",
                dev_name, seify_args
            );
            let mut builder = futuresdr::blocks::seify::Builder::new(seify_args.as_str())
                .with_context(|| {
                    format!(
                        "Failed to open SDR device '{}' with args '{}'",
                        dev_name, seify_args
                    )
                })?
                .sample_rate(sample_rate)
                .frequency(rx_freq);

            if let Some(gain) = hw.rx_gain {
                builder = builder.gain(gain);
            }

            let src = builder.build_source().with_context(|| {
                format!(
                    "Failed to configure and build Seify source for device '{}'",
                    dev_name
                )
            })?;
            let id = active_fg.add(src)?.id();
            println!(
                "[fsdr-hub] Successfully initialized Hardware SDR Source '{}' at {} SPS, {} Hz",
                dev_name, sample_rate, rx_freq
            );
            (Some(id), "outputs[0]")
        }
        #[cfg(not(feature = "soapy"))]
        {
            anyhow::bail!(
                "Hardware SDR device '{}' was configured, but fsdr-hub was compiled without the 'soapy' feature. Recompile with `--features soapy`.",
                dev_name
            );
        }
    } else {
        println!("[fsdr-hub] Notice: No SDR device specified. DSP pipelines will read from STDIN.");
        (None, "output")
    };

    let mut tap_nodes: HashMap<String, (BlockId, String)> = HashMap::new();
    if let Some(src_id) = sdr_source_id {
        tap_nodes.insert(
            "sdr_raw_bus".to_string(),
            (src_id, sdr_output_port.to_string()),
        );
        tap_nodes.insert(
            "rf_corrected_bus".to_string(),
            (src_id, sdr_output_port.to_string()),
        );
    }

    for ch in &config.channels {
        let vars = config.build_env_vars(&ch.id, delta_f)?;
        let initial_channel_freq = channels_freq_map
            .read()
            .unwrap()
            .get(&ch.id)
            .map(|f| f.load(Ordering::Relaxed))
            .unwrap_or(ch.frequency.unwrap_or(rx_center_freq as u64));

        // Build RX pipeline if defined
        if let Some(ref rx) = ch.rx {
            if rx.enabled.unwrap_or(true) {
                if let Some(ref pipe) = rx.pipeline {
                    let norm = HubConfig::normalize_pipeline_with_vars(pipe, &vars);
                    let mut grc = CsdrParser::parse_multiple_commands(&norm)
                        .with_context(|| {
                            format!(
                                "Failed to parse pipeline syntax for channel '{}' (RX): {}",
                                ch.id, norm
                            )
                        })?
                        .context(format!("Pipeline was empty for channel '{}' (RX)", ch.id))?;

                    let initial_rx_shift = DspFrequencyController::calc_rx_shift(
                        initial_channel_freq,
                        rx_center_freq,
                        delta_f,
                        sdr_sample_rate,
                    );

                    // Automatically instantiate and register the channel's NCO FrequencyShifter
                    let rx_shifter = FrequencyShifter::<Complex32>::new(initial_rx_shift, 1.0);
                    let rx_shifter_id = active_fg.add(rx_shifter)?.id();
                    dsp_controller.update_rx_shifter(&ch.id, rx_shifter_id);

                    if verbose {
                        println!(
                            "[fsdr-hub][verbose] Registered automatic RX frequency shifter for channel '{}': block {:?} (initial shift: {:+.6})",
                            ch.id, rx_shifter_id, initial_rx_shift
                        );
                    }

                    // Register default tap nodes for this channel
                    tap_nodes.insert(
                        format!("channels.{}.rx_raw", ch.id),
                        (rx_shifter_id, "output".to_string()),
                    );
                    tap_nodes.insert(
                        format!("channels.{}.rx", ch.id),
                        (rx_shifter_id, "output".to_string()),
                    );

                    // Connect SDR source to this channel's rx_shifter
                    if let Some(src_id) = sdr_source_id {
                        active_fg.stream_dyn(src_id, sdr_output_port, rx_shifter_id, "input")?;
                        if verbose {
                            println!(
                                "[fsdr-hub][verbose] Connected SDR source -> channel '{}' RX shifter",
                                ch.id
                            );
                        }
                    }

                    // If we have a dummy blocks_file_source_0 reading from stdin, remove it so we wire from rx_shifter!
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

                    // Strip any legacy shift_addition_cc or decimating_shift_addition_cc at start of pipeline
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
                            println!(
                                "[fsdr-hub] Channel '{}' RX pipeline contained legacy shift_addition_cc; replaced with automatic NCO shifter.",
                                ch.id
                            );
                        }
                    }

                    let grc_blocks_snapshot = grc.blocks.clone();
                    let grc_connections_snapshot = grc.connections.clone();

                    let adapters = grc_converter
                        .convert_grc_into(grc, &mut active_fg)
                        .with_context(|| {
                            format!(
                                "Failed to instantiate DSP blocks for channel '{}' (RX): {}",
                                ch.id, norm
                            )
                        })?;

                    // Connect rx_shifter output -> first DSP block of this RX channel
                    if let Some((target_name, target_port)) = first_dsp_name {
                        if let Some(adapter) = adapters.get(&target_name) {
                            let (target_blk_id, target_in_port) =
                                adapter.adapt_input_port(&target_port)?;
                            active_fg.stream_dyn(
                                rx_shifter_id,
                                "output",
                                target_blk_id,
                                target_in_port,
                            )?;
                            if verbose {
                                println!(
                                    "[fsdr-hub][verbose] Connected RX shifter -> channel '{}' ({}:{})",
                                    ch.id, target_name, target_in_port
                                );
                            }
                        }
                    }

                    // Register the channel's complex baseband tap node (e.g. before demodulator)
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
                                if verbose {
                                    println!(
                                        "[fsdr-hub][verbose] Registered channel complex tap 'channels.{}.rx' at {}:{}",
                                        ch.id, tap_name, tap_p
                                    );
                                }
                            }
                        }
                    }

                    pipeline_count += 1;
                    println!("[fsdr-hub] Loaded RX pipeline for channel '{}'", ch.id);
                }
            }
        }

        // Build TX pipeline if defined and enabled
        if let Some(ref tx) = ch.tx {
            if tx.enabled.unwrap_or(true) && beacon_tracker.is_tx_allowed() {
                if let Some(ref pipe) = tx.pipeline {
                    let norm = HubConfig::normalize_pipeline_with_vars(pipe, &vars);
                    let mut grc = CsdrParser::parse_multiple_commands(&norm)
                        .with_context(|| {
                            format!(
                                "Failed to parse pipeline syntax for channel '{}' (TX): {}",
                                ch.id, norm
                            )
                        })?
                        .context(format!("Pipeline was empty for channel '{}' (TX)", ch.id))?;

                    // Remove dummy blocks_file_sink if present
                    let mut last_dsp_name = None;
                    if !grc.blocks.is_empty()
                        && grc.blocks.last().map(|b| b.id.as_str()) == Some("blocks_file_sink")
                    {
                        let dummy_sink = grc.blocks.pop().unwrap();
                        if let Some(conn) = grc.connections.iter().find(|c| c[2] == dummy_sink.name)
                        {
                            last_dsp_name = Some((conn[0].clone(), conn[1].clone()));
                        }
                        grc.connections.retain(|c| c[2] != dummy_sink.name);
                    }

                    // Strip legacy shift_addition_cc at end of TX pipeline
                    if let Some((last_name, _)) = last_dsp_name.clone() {
                        if let Some(pos) = grc.blocks.iter().position(|b| {
                            b.name == last_name
                                && (b.id == "blocks_freqshift_cc"
                                    || b.id == "decimating_shift_addition_cc")
                        }) {
                            if let Some(conn) = grc.connections.iter().find(|c| c[2] == last_name) {
                                last_dsp_name = Some((conn[0].clone(), conn[1].clone()));
                            }
                            grc.blocks.remove(pos);
                            grc.connections
                                .retain(|c| c[0] != last_name && c[2] != last_name);
                            println!(
                                "[fsdr-hub] Channel '{}' TX pipeline contained legacy shift_addition_cc; replaced with automatic NCO shifter.",
                                ch.id
                            );
                        }
                    }

                    let adapters = grc_converter
                        .convert_grc_into(grc, &mut active_fg)
                        .with_context(|| {
                            format!(
                                "Failed to instantiate DSP blocks for channel '{}' (TX): {}",
                                ch.id, norm
                            )
                        })?;

                    if let Some((src_name, src_port)) = last_dsp_name {
                        if let Some(adapter) = adapters.get(&src_name) {
                            let initial_tx_shift = DspFrequencyController::calc_tx_shift(
                                initial_channel_freq,
                                tx_center_freq,
                                sdr_sample_rate,
                            );

                            let tx_shifter =
                                FrequencyShifter::<Complex32>::new(initial_tx_shift, 1.0);
                            let tx_shifter_id = active_fg.add(tx_shifter)?.id();
                            dsp_controller.update_tx_shifter(&ch.id, tx_shifter_id);

                            if verbose {
                                println!(
                                    "[fsdr-hub][verbose] Registered automatic TX frequency shifter for channel '{}': block {:?} (initial shift: {:+.6})",
                                    ch.id, tx_shifter_id, initial_tx_shift
                                );
                            }

                            let (src_blk_id, src_out_port) =
                                adapter.adapt_output_port(&src_port)?;
                            active_fg.stream_dyn(
                                src_blk_id,
                                src_out_port,
                                tx_shifter_id,
                                "input",
                            )?;

                            // Terminate the TX stream safely into a NullSink (until routed to an SDR TX hardware sink)
                            let tx_null_sink = futuresdr::blocks::NullSink::<Complex32>::new();
                            let tx_null_sink_id = active_fg.add(tx_null_sink)?.id();
                            active_fg.stream_dyn(
                                tx_shifter_id,
                                "output",
                                tx_null_sink_id,
                                "input",
                            )?;

                            tap_nodes.insert(
                                format!("channels.{}.tx", ch.id),
                                (tx_shifter_id, "output".to_string()),
                            );

                            if verbose {
                                println!(
                                    "[fsdr-hub][verbose] Connected channel '{}' TX ({}:{}) -> TX shifter -> TX NullSink",
                                    ch.id, src_name, src_out_port
                                );
                            }
                        }
                    }

                    pipeline_count += 1;
                    println!("[fsdr-hub] Loaded TX pipeline for channel '{}'", ch.id);
                }
            }
        }
    }

    // 6. Build Telemetry probe pipelines
    if let Some(ref tel) = config.telemetry {
        if let Some(ref bokeh_cfg) = tel.bokeh {
            for probe in &bokeh_cfg.probes {
                if let Some(ref pipe) = probe.pipeline {
                    let norm = HubConfig::normalize_pipeline(pipe);
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

                        if let Ok(adapters) = grc_converter.convert_grc_into(grc, &mut active_fg) {
                            let source_node = probe.source_node.as_deref().unwrap_or("sdr_raw_bus");
                            if let (Some((src_id, src_port)), Some((target_name, target_port))) =
                                (tap_nodes.get(source_node), first_dsp_name)
                            {
                                if let Some(adapter) = adapters.get(&target_name) {
                                    if let Ok((target_blk_id, target_in_port)) =
                                        adapter.adapt_input_port(&target_port)
                                    {
                                        let _ = active_fg.stream_dyn(
                                            *src_id,
                                            src_port.as_str(),
                                            target_blk_id,
                                            target_in_port,
                                        );
                                        if verbose {
                                            println!(
                                                "[fsdr-hub][verbose] Connected tap '{}' ({:?}:{}) -> telemetry probe '{}' ({}:{})",
                                                source_node, src_id, src_port, probe.name, target_name, target_in_port
                                            );
                                        }
                                    }
                                }
                            } else {
                                eprintln!(
                                    "[fsdr-hub] Warning: Could not resolve source_node '{}' for probe '{}'",
                                    source_node, probe.name
                                );
                            }
                            pipeline_count += 1;
                            println!(
                                "[fsdr-hub] Loaded telemetry probe pipeline '{}' (source: '{}')",
                                probe.name, source_node
                            );
                        }
                    }
                }
            }
        }
    }

    if verbose {
        println!(
            "[fsdr-hub][verbose] Unified Flowgraph constructed with {} active pipelines",
            pipeline_count
        );
    }

    println!(
        "[fsdr-hub] Successfully initialized {} DSP pipelines into unified Flowgraph",
        pipeline_count
    );

    // Setup graceful shutdown on Ctrl-C / SIGTERM
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let tokio_rt = tokio::runtime::Runtime::new()?;
    tokio_rt.spawn(async move {
        #[cfg(unix)]
        let mut sigterm =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).ok();
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = async {
                #[cfg(unix)]
                if let Some(ref mut st) = sigterm {
                    st.recv().await;
                } else {
                    futures::future::pending::<()>().await;
                }
                #[cfg(not(unix))]
                futures::future::pending::<()>().await;
            } => {},
        }
        println!("\n[fsdr-hub] Shutdown signal received, cleaning up virtual audio sinks...");
        if let Ok(mut guard) = pa_guard_sig.lock() {
            guard.cleanup();
        }
        r.store(false, Ordering::SeqCst);
        std::process::exit(0);
    });

    // If active DSP pipelines exist, run runtime
    let mut _rt = None;
    let mut _running_fg = None;
    if pipeline_count > 0 {
        println!("[fsdr-hub] Running FutureSDR flowgraph...");
        let rt = Runtime::new();
        let running_fg = rt.start(active_fg)?;
        let fg_handle = running_fg.handle();
        tokio_rt.spawn(dsp_controller.run_dispatcher(fg_handle, beacon_tracker.clone(), verbose));
        _running_fg = Some(running_fg);
        _rt = Some(rt);
    }

    while running.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    if let Ok(mut guard) = pa_guard.lock() {
        guard.cleanup();
    }

    println!("[fsdr-hub] Exiting cleanly.");
    Ok(())
}
