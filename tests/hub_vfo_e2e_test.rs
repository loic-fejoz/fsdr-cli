use anyhow::{Context, Result};
use fsdr_cli::audio::PulseAudioGuard;
use fsdr_cli::config::HubConfig;
use std::process::{Child, Command};
use std::time::Duration;

#[test]
fn test_hub_vfo_retuning_python_e2e() -> Result<()> {
    // 1. Check Python environment and dependencies
    let python_bin = if std::path::Path::new(".venv/bin/python3").exists() {
        ".venv/bin/python3"
    } else {
        "python3"
    };

    let check = Command::new(python_bin)
        .args(["-c", "import tornado, numpy"])
        .status();

    if check.is_err() || !check.unwrap().success() {
        eprintln!(
            "[test] Skipping Python E2E retuning test: tornado/numpy not installed in environment"
        );
        return Ok(());
    }

    let config_path = "examples/cfg-synthetic.toml";
    let config = HubConfig::load_from_file(config_path)?;

    // 2. Clean previous audio endpoints belonging to this config before starting
    PulseAudioGuard::cleanup_config_sinks(&config);

    // Launch fsdr-hub binary with cfg-synthetic.toml
    let hub_bin = env!("CARGO_BIN_EXE_fsdr-hub");
    let hub_proc = Command::new(hub_bin)
        .args(["--config", config_path, "-v"])
        .spawn()
        .context("Failed to spawn fsdr-hub binary")?;

    fn clean_child_and_sinks(mut child: Child, config: &HubConfig) {
        #[cfg(unix)]
        {
            let pid = child.id().to_string();
            let _ = Command::new("kill").args(["-TERM", &pid]).status();
            for _ in 0..20 {
                if let Ok(Some(_)) = child.try_wait() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
        let _ = child.kill();
        let _ = child.wait();
        PulseAudioGuard::cleanup_config_sinks(config);
    }

    struct ProcGuard {
        child: Option<Child>,
        config: HubConfig,
    }
    impl Drop for ProcGuard {
        fn drop(&mut self) {
            if let Some(child) = self.child.take() {
                clean_child_and_sinks(child, &self.config);
            }
        }
    }
    let mut guard = ProcGuard {
        child: Some(hub_proc),
        config: config.clone(),
    };

    // Allow fsdr-hub server to start WebSocket listener and DSP flowgraph
    std::thread::sleep(Duration::from_millis(1500));

    // 3. Run Python E2E verification script
    let status = Command::new(python_bin)
        .arg("tests/test_vfo_retuning_e2e.py")
        .status()
        .context("Failed to run tests/test_vfo_retuning_e2e.py")?;

    // Cleanly terminate fsdr-hub and ensure only its sinks are unloaded
    if let Some(child) = guard.child.take() {
        clean_child_and_sinks(child, &config);
    }

    assert!(status.success(), "Python E2E retuning test failed!");
    Ok(())
}
