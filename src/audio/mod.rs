use anyhow::{Context, Result};
use futuresdr::tracing::{info, warn};
use std::process::Command;
use std::sync::{Arc, Mutex};

use crate::config::HubConfig;

#[derive(Debug, Default)]
pub struct PulseAudioGuard {
    created_modules: Vec<u32>,
}

impl PulseAudioGuard {
    pub fn new() -> Self {
        Self {
            created_modules: Vec::new(),
        }
    }

    /// Check if pactl command is available
    pub fn is_pactl_available() -> bool {
        Command::new("pactl")
            .arg("--version")
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }

    /// Load a null sink via pactl
    pub fn load_null_sink(
        &mut self,
        sink_name: &str,
        description: Option<&str>,
    ) -> Result<Option<u32>> {
        if !Self::is_pactl_available() {
            warn!("pactl is not available on this system. Skipping virtual null sink creation.");
            return Ok(None);
        }

        let desc = description.unwrap_or(sink_name);
        // PipeWire / PulseAudio pactl parses sink_properties with quotes: device.description='...'
        let prop = format!("device.description='{}'", desc);

        let output = Command::new("pactl")
            .arg("load-module")
            .arg("module-null-sink")
            .arg(format!("sink_name={}", sink_name))
            .arg(format!("sink_properties={}", prop))
            .output()
            .with_context(|| format!("Failed to execute pactl load-module for {}", sink_name))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("pactl load-module failed for {}: {}", sink_name, stderr);
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let id_str = stdout.trim();
        if let Ok(id) = id_str.parse::<u32>() {
            info!(
                "Created PulseAudio null sink '{}' (module ID: {})",
                sink_name, id
            );
            self.created_modules.push(id);
            Ok(Some(id))
        } else {
            warn!("Could not parse module ID from pactl output: '{}'", id_str);
            Ok(None)
        }
    }

    /// Unload a module via pactl
    pub fn unload_module(&mut self, id: u32) -> Result<()> {
        if !Self::is_pactl_available() {
            return Ok(());
        }

        let status = Command::new("pactl")
            .arg("unload-module")
            .arg(id.to_string())
            .status();

        match status {
            Ok(s) if s.success() => {
                info!("Unloaded PulseAudio module ID: {}", id);
            }
            Ok(s) => {
                warn!("pactl unload-module {} exited with {}", id, s);
            }
            Err(e) => {
                warn!("Failed to execute pactl unload-module {}: {}", id, e);
            }
        }
        Ok(())
    }

    /// Unload an existing null sink matching the specified sink name, if present.
    pub fn unload_sink_by_name(sink_name: &str) -> Result<bool> {
        if !Self::is_pactl_available() {
            return Ok(false);
        }

        let output = Command::new("pactl")
            .arg("list")
            .arg("modules")
            .output()
            .context("Failed to list pactl modules")?;

        if !output.status.success() {
            return Ok(false);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut current_id: Option<u32> = None;
        let mut unlinked = false;

        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Module #") {
                if let Some(id_str) = trimmed.strip_prefix("Module #") {
                    current_id = id_str.trim().parse::<u32>().ok();
                }
            } else if trimmed.contains(&format!("sink_name={}", sink_name)) {
                if let Some(id) = current_id {
                    let _ = Command::new("pactl")
                        .args(["unload-module", &id.to_string()])
                        .status();
                    info!(
                        "Unloaded existing null sink '{}' (module ID: {})",
                        sink_name, id
                    );
                    unlinked = true;
                    current_id = None;
                }
            }
        }

        Ok(unlinked)
    }

    /// Cleanup all sinks defined in a HubConfig by name
    pub fn cleanup_config_sinks(config: &HubConfig) {
        for channel in &config.channels {
            if let Some(audio) = &channel.audio {
                if let Some(sink_name) = &audio.sink_name {
                    let _ = Self::unload_sink_by_name(sink_name);
                }
                if let Some(source_name) = &audio.source_name {
                    let _ = Self::unload_sink_by_name(source_name);
                }
            }
        }
    }

    /// Cleanup all loaded modules
    pub fn cleanup(&mut self) {
        let modules: Vec<u32> = std::mem::take(&mut self.created_modules);
        for id in modules {
            let _ = self.unload_module(id);
        }
    }

    /// Set up virtual endpoints from a HubConfig
    pub fn setup_from_config(config: &HubConfig) -> Result<Self> {
        let mut guard = Self::new();
        for channel in &config.channels {
            if let Some(audio) = &channel.audio {
                if let Some(sink_name) = &audio.sink_name {
                    // Unload previous module if already present with same sink_name
                    let _ = Self::unload_sink_by_name(sink_name);
                    let _ = guard.load_null_sink(sink_name, Some(sink_name))?;
                }
                if let Some(source_name) = &audio.source_name {
                    let _ = Self::unload_sink_by_name(source_name);
                    let _ = guard.load_null_sink(source_name, Some(source_name))?;
                }
            }
        }
        Ok(guard)
    }

    /// Spawn a signal handler that ensures cleanup on Ctrl+C / SIGINT
    pub fn spawn_signal_cleanup(guard: Arc<Mutex<Self>>) {
        tokio::spawn(async move {
            if tokio::signal::ctrl_c().await.is_ok() {
                info!("Received termination signal, cleaning up audio endpoints...");
                if let Ok(mut g) = guard.lock() {
                    g.cleanup();
                }
                std::process::exit(0);
            }
        });
    }
}

impl Drop for PulseAudioGuard {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guard_lifecycle() {
        if !PulseAudioGuard::is_pactl_available() {
            eprintln!("pactl not available, skipping test");
            return;
        }

        let mut guard = PulseAudioGuard::new();
        let mod_id = guard
            .load_null_sink("fsdr_test_sink", Some("FSDR Test Sink"))
            .unwrap();

        if let Some(id) = mod_id {
            assert!(guard.created_modules.contains(&id));
            guard.cleanup();
            assert!(guard.created_modules.is_empty());
        }
    }
}
