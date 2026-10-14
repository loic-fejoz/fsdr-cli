use anyhow::{Context, Result};
use fsdr_cli::audio::PulseAudioGuard;
use fsdr_cli::beacon::BeaconTracker;
use fsdr_cli::cat::{CatClientProxy, CatServer};
use fsdr_cli::config::HubConfig;
use fsdr_cli::csdr_cmd::CsdrParser;
use fsdr_cli::grc::converter::Grc2FutureSdr;
use fsdr_cli::telemetry::BokehServer;
use futuresdr::runtime::{Flowgraph, Runtime};
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

    // 3. Initialize CAT servers and client proxies
    let mut _cat_servers = Vec::new();
    let mut _cat_proxies = Vec::new();

    for ch in &config.channels {
        let initial_freq = ch
            .frequency
            .or(ch.frequency_rx)
            .or(ch.frequency_tx)
            .unwrap_or(144_000_000);

        if let Some(port) = ch.cat_port {
            let server = CatServer::spawn(port, initial_freq).with_context(|| {
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
                let server = CatServer::spawn(port, initial_freq).with_context(|| {
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
                let freq_source = Arc::new(std::sync::atomic::AtomicU64::new(initial_freq));
                let proxy =
                    CatClientProxy::spawn(target.clone(), lo_offset, lo_multiplier, freq_source);
                println!(
                    "[fsdr-hub] CAT TX proxy for channel '{}' connected to {}",
                    ch.id, target
                );
                _cat_proxies.push(proxy);
            }
        }
    }

    // 4. Initialize Telemetry (Bokeh)
    let mut _bokeh_server = None;
    if let Some(ref tel) = config.telemetry {
        if let Some(ref bokeh_cfg) = tel.bokeh {
            let listen_addr = bokeh_cfg
                .listen_addr
                .clone()
                .unwrap_or_else(|| "127.0.0.1:5006".to_string());
            let (tx, rx) = futures::channel::mpsc::unbounded::<Vec<f32>>();
            fsdr_cli::blocks::set_global_bokeh_sender(tx);
            let server = BokehServer::spawn(listen_addr.clone(), rx).with_context(|| {
                format!("Failed to spawn Bokeh telemetry server on {}", listen_addr)
            })?;
            println!(
                "[fsdr-hub] Bokeh WebSocket telemetry server running on {}",
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

    for ch in &config.channels {
        let vars = config.build_env_vars(&ch.id, delta_f)?;

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

                    // If we have an active SDR source (e.g. synthetic or hardware SDR),
                    // remove dummy blocks_file_source_0 reading from stdin so we can wire directly from SDR source!
                    let mut first_dsp_name = None;
                    if let Some(_src_id) = sdr_source_id {
                        if !grc.blocks.is_empty()
                            && grc.blocks[0].id == "blocks_file_source"
                            && grc.blocks[0].parameter_or("file", "") == "-"
                        {
                            let dummy_name = grc.blocks[0].name.clone();
                            // Find the target block connected to dummy source port 0
                            if let Some(conn) = grc.connections.iter().find(|c| c[0] == dummy_name)
                            {
                                first_dsp_name = Some((conn[2].clone(), conn[3].clone()));
                            }
                            // Remove dummy block and dummy connection
                            grc.blocks.remove(0);
                            grc.connections.retain(|c| c[0] != dummy_name);
                        }
                    }

                    let adapters = grc_converter
                        .convert_grc_into(grc, &mut active_fg)
                        .with_context(|| {
                            format!(
                                "Failed to instantiate DSP blocks for channel '{}' (RX): {}",
                                ch.id, norm
                            )
                        })?;

                    // Connect SDR source to the first DSP block of this RX channel
                    if let (Some(src_id), Some((target_name, target_port))) =
                        (sdr_source_id, first_dsp_name)
                    {
                        if let Some(adapter) = adapters.get(&target_name) {
                            let (target_blk_id, target_in_port) =
                                adapter.adapt_input_port(&target_port)?;
                            active_fg.stream_dyn(
                                src_id,
                                sdr_output_port,
                                target_blk_id,
                                target_in_port,
                            )?;
                            if verbose {
                                println!(
                                    "[fsdr-hub][verbose] Connected SDR source -> channel '{}' ({}:{})",
                                    ch.id, target_name, target_in_port
                                );
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
                    let grc = CsdrParser::parse_multiple_commands(&norm)
                        .with_context(|| {
                            format!(
                                "Failed to parse pipeline syntax for channel '{}' (TX): {}",
                                ch.id, norm
                            )
                        })?
                        .context(format!("Pipeline was empty for channel '{}' (TX)", ch.id))?;
                    grc_converter
                        .convert_grc_into(grc, &mut active_fg)
                        .with_context(|| {
                            format!(
                                "Failed to instantiate DSP blocks for channel '{}' (TX): {}",
                                ch.id, norm
                            )
                        })?;
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
                        if let Some(_src_id) = sdr_source_id {
                            if !grc.blocks.is_empty()
                                && grc.blocks[0].id == "blocks_file_source"
                                && grc.blocks[0].parameter_or("file", "") == "-"
                            {
                                let dummy_name = grc.blocks[0].name.clone();
                                if let Some(conn) =
                                    grc.connections.iter().find(|c| c[0] == dummy_name)
                                {
                                    first_dsp_name = Some((conn[2].clone(), conn[3].clone()));
                                }
                                grc.blocks.remove(0);
                                grc.connections.retain(|c| c[0] != dummy_name);
                            }
                        }

                        if let Ok(adapters) = grc_converter.convert_grc_into(grc, &mut active_fg) {
                            if let (Some(src_id), Some((target_name, target_port))) =
                                (sdr_source_id, first_dsp_name)
                            {
                                if let Some(adapter) = adapters.get(&target_name) {
                                    if let Ok((target_blk_id, target_in_port)) =
                                        adapter.adapt_input_port(&target_port)
                                    {
                                        let _ = active_fg.stream_dyn(
                                            src_id,
                                            sdr_output_port,
                                            target_blk_id,
                                            target_in_port,
                                        );
                                        if verbose {
                                            println!(
                                                "[fsdr-hub][verbose] Connected SDR source -> telemetry probe '{}' ({}:{})",
                                                probe.name, target_name, target_in_port
                                            );
                                        }
                                    }
                                }
                            }
                            pipeline_count += 1;
                            println!(
                                "[fsdr-hub] Loaded telemetry probe pipeline '{}'",
                                probe.name
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
