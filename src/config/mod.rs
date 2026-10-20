use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubConfig {
    pub hardware: Option<HardwareSection>,
    pub beacon_tracker: Option<BeaconTrackerConfig>,
    #[serde(default)]
    pub channels: Vec<ChannelConfig>,
    pub telemetry: Option<TelemetryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareSection {
    pub sdr: Option<HardwareConfig>,
    pub pluto: Option<HardwareConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HardwareConfig {
    pub device: Option<String>,
    pub uri: Option<String>,
    pub sample_rate: Option<u64>,
    pub rx_freq: Option<u64>,
    pub rx_gain: Option<f64>,
    pub tx_freq: Option<u64>,
    pub tx_gain: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BeaconTrackerConfig {
    #[serde(default)]
    pub enabled: bool,
    pub cat_port: Option<u16>,
    pub beacon_freq: Option<u64>,
    pub beacon_freq_offset: Option<i64>,
    pub mode: Option<String>,
    pub max_drift_hz: Option<f64>,
    pub holdover_on_loss: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub id: String,
    #[serde(rename = "type")]
    pub channel_type: String,
    pub cat_port: Option<u16>,
    pub frequency: Option<u64>,
    pub frequency_offset: Option<i64>,
    pub frequency_rx: Option<u64>,
    pub frequency_tx: Option<u64>,
    #[serde(default = "default_priority")]
    pub priority: u8,
    pub cat_rx: Option<CatRxConfig>,
    pub cat_tx: Option<CatTxConfig>,
    pub audio: Option<AudioChannelConfig>,
    pub kiss: Option<KissChannelConfig>,
    pub rx: Option<PipelineChannelConfig>,
    pub tx: Option<PipelineChannelConfig>,
}

fn default_priority() -> u8 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatRxConfig {
    #[serde(rename = "type")]
    pub cat_type: Option<String>,
    pub listen_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatTxConfig {
    #[serde(rename = "type")]
    pub cat_type: Option<String>,
    pub listen_port: Option<u16>,
    pub target: Option<String>,
    pub lo_multiplier: Option<i64>,
    pub lo_offset: Option<i64>,
    pub track_rx_tuning: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioChannelConfig {
    pub sink_name: Option<String>,
    pub source_name: Option<String>,
    pub sample_rate: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KissChannelConfig {
    pub tcp_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineChannelConfig {
    pub pipeline: Option<String>,
    #[serde(default = "default_enabled")]
    pub enabled: Option<bool>,
}

fn default_enabled() -> Option<bool> {
    Some(true)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    pub bokeh: Option<TelemetryBokehConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryBokehConfig {
    pub listen_addr: Option<String>,
    pub fps: Option<u32>,
    #[serde(default)]
    pub probes: Vec<BokehProbeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BokehProbeConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub probe_type: Option<String>,
    pub source_node: Option<String>,
    pub fft_size: Option<usize>,
    pub averaging: Option<usize>,
    pub pipeline: Option<String>,
}

impl HubConfig {
    pub fn load_from_str(content: &str) -> Result<Self> {
        let config: HubConfig = toml::from_str(content)?;
        Ok(config)
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file {:?}", path.as_ref()))?;
        Self::load_from_str(&content)
    }

    pub fn get_hardware(&self) -> HardwareConfig {
        if let Some(hw) = &self.hardware {
            if let Some(sdr) = &hw.sdr {
                return sdr.clone();
            }
            if let Some(pluto) = &hw.pluto {
                return pluto.clone();
            }
        }
        HardwareConfig::default()
    }

    pub fn build_env_vars(
        &self,
        channel_id: &str,
        delta_f_beacon: i64,
    ) -> Result<HashMap<String, f32>> {
        let mut vars = HashMap::new();
        let hw = self.get_hardware();
        let sample_rate = hw.sample_rate.unwrap_or(1_000_000) as f32;
        let rx_center = hw.rx_freq.unwrap_or(0);
        let tx_center = hw.tx_freq.unwrap_or(0);

        vars.insert("sample_rate".to_string(), sample_rate);

        let channel = self
            .channels
            .iter()
            .find(|c| c.id == channel_id)
            .with_context(|| format!("Channel not found: {}", channel_id))?;

        // Calculate offset, offset_rx, offset_tx
        let mut offset = 0.0f32;
        let mut offset_rx = 0.0f32;
        let mut offset_tx = 0.0f32;

        if let Some(off) = channel.frequency_offset {
            offset = off as f32;
            offset_rx = off as f32;
            offset_tx = off as f32;
        } else if let Some(freq) = channel.frequency {
            let diff = (freq as i64) - (rx_center as i64) - delta_f_beacon;
            offset = diff as f32;
            offset_rx = diff as f32;
            offset_tx = ((channel.frequency_tx.unwrap_or(freq) as i64) - (tx_center as i64)) as f32;
        }

        if let Some(freq_rx) = channel.frequency_rx {
            let diff = (freq_rx as i64) - (rx_center as i64) - delta_f_beacon;
            offset_rx = diff as f32;
            if offset == 0.0 {
                offset = offset_rx;
            }
        }

        if let Some(freq_tx) = channel.frequency_tx {
            let diff = (freq_tx as i64) - (tx_center as i64);
            offset_tx = diff as f32;
            if offset == 0.0 && channel.frequency_rx.is_none() {
                offset = offset_tx;
            }
        }

        vars.insert("offset".to_string(), offset);
        vars.insert("offset_rx".to_string(), offset_rx);
        vars.insert("offset_tx".to_string(), offset_tx);
        vars.insert("delta_f_beacon".to_string(), delta_f_beacon as f32);

        Ok(vars)
    }

    /// Normalizes template expressions in pipeline string:
    /// Replaces `{offset}` -> `offset`, `{offset_rx}` -> `offset_rx`, etc.
    /// Handles backslashes and multiple lines.
    pub fn normalize_pipeline(pipeline_raw: &str) -> String {
        let lines: Vec<&str> = pipeline_raw
            .lines()
            .map(|l| l.trim().trim_end_matches('\\').trim())
            .filter(|l| !l.is_empty())
            .collect();
        let joined = lines.join(" ");
        joined
            .replace("{offset}", "offset")
            .replace("{offset_rx}", "offset_rx")
            .replace("{offset_tx}", "offset_tx")
            .replace("{delta_f_beacon}", "delta_f_beacon")
    }

    /// Normalizes and substitutes numeric values for template variables:
    /// Replaces `{offset}` with its evaluated value (e.g. `-126000.0`), etc.
    pub fn normalize_pipeline_with_vars(pipeline_raw: &str, vars: &HashMap<String, f32>) -> String {
        let lines: Vec<&str> = pipeline_raw
            .lines()
            .map(|l| l.trim().trim_end_matches('\\').trim())
            .filter(|l| !l.is_empty())
            .collect();
        let mut joined = lines.join(" ");
        for (var_name, val) in vars {
            let key = format!("{{{}}}", var_name);
            let val_str = format!("{:.3}", val);
            joined = joined.replace(&key, &val_str);
            // Also replace non-bracketed if bare variable name
            joined = joined.replace(var_name, &val_str);
        }
        joined
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_all_sample_configs() {
        let cfg_2m =
            HubConfig::load_from_file("examples/cfg-2m.toml").expect("cfg-2m.toml should parse");
        assert_eq!(cfg_2m.channels.len(), 4);
        let vars_ft8 = cfg_2m.build_env_vars("vhf_ft8", 0).unwrap();
        // 144_174_000 - 145_000_000 = -826_000
        assert_eq!(*vars_ft8.get("offset").unwrap(), -826000.0);

        let cfg_qo100 = HubConfig::load_from_file("examples/cfg-qo100.toml")
            .expect("cfg-qo100.toml should parse");
        assert_eq!(cfg_qo100.channels.len(), 3);
        let vars_qo = cfg_qo100.build_env_vars("ft8", 0).unwrap();
        assert_eq!(*vars_qo.get("offset").unwrap(), -140000.0);

        let cfg_mixed = HubConfig::load_from_file("examples/cfg-qo100-mixed.toml")
            .expect("cfg-qo100-mixed.toml should parse");
        assert_eq!(cfg_mixed.channels.len(), 2);
    }

    #[test]
    fn test_parse_pipelines_from_all_configs() {
        use crate::csdr_cmd::CsdrParser;

        let configs = [
            "examples/cfg-2m.toml",
            "examples/cfg-qo100.toml",
            "examples/cfg-qo100-mixed.toml",
            "examples/cfg-20m.toml",
        ];
        for path in configs {
            let cfg = HubConfig::load_from_file(path).expect("failed to load config");
            for ch in &cfg.channels {
                let _vars = cfg
                    .build_env_vars(&ch.id, 0)
                    .expect("build_env_vars failed");
                if let Some(ref rx) = ch.rx {
                    if let Some(ref pipe) = rx.pipeline {
                        let norm = HubConfig::normalize_pipeline(pipe);
                        let grc = CsdrParser::parse_multiple_commands(&norm).unwrap_or_else(|e| {
                            panic!(
                                "Channel {} rx pipeline failed in {}: {}\nPipeline: {}",
                                ch.id, path, e, norm
                            );
                        });
                        assert!(grc.is_some());
                    }
                }
                if let Some(ref tx) = ch.tx {
                    if let Some(ref pipe) = tx.pipeline {
                        let norm = HubConfig::normalize_pipeline(pipe);
                        let grc = CsdrParser::parse_multiple_commands(&norm).unwrap_or_else(|e| {
                            panic!(
                                "Channel {} tx pipeline failed in {}: {}\nPipeline: {}",
                                ch.id, path, e, norm
                            );
                        });
                        assert!(grc.is_some());
                    }
                }
            }
        }
    }
}
