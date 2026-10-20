use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait HubBlocksCmd<'i> {
    fn build_pulse_source(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
    fn build_pulse_sink(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
    fn build_weaver_usb_fc(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
    fn build_pulse_shaping_filter(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>>;
    fn build_firdes_pulse_shaping(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>>;
    fn build_qpsk_modulator(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
    fn build_oqpsk_modulator(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
    fn build_qpsk_demod(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
    fn build_oqpsk_demod(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
    fn build_qpsk_costas_loop(&self, grc: GrcBuilder<GraphLevel>)
        -> Result<GrcBuilder<GraphLevel>>;
    fn build_oqpsk_costas_loop(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>>;
    fn build_pdu_deframing_g3ruh(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>>;
    fn build_pdu_framing_g3ruh(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>>;
    fn build_afsk1200_demod(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
    fn build_afsk1200_mod(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
    fn build_hdlc_deframing(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
    fn build_hdlc_framing(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
    fn build_fir_interpolate(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
    fn build_bokeh_spectrum_sink(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>>;
    fn build_synthetic_sdr_source(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>>;
}

impl<'i> HubBlocksCmd<'i> for Pair<'i, Rule> {
    fn build_pulse_source(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let dev_name = self
            .clone()
            .into_inner()
            .next()
            .map(|p| p.as_str())
            .unwrap_or("default");
        let dev_name = dev_name.trim_matches('"').trim_matches('\'');
        let mut grc = grc;
        grc = grc
            .create_block_instance("pulse_source")
            .with_parameter("device_name", dev_name)
            .assert_output(GrcItemType::F32)
            .push()?;
        Ok(grc)
    }

    fn build_pulse_sink(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let dev_name = self
            .clone()
            .into_inner()
            .next()
            .map(|p| p.as_str())
            .unwrap_or("default");
        let dev_name = dev_name.trim_matches('"').trim_matches('\'');
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::F32)?
            .create_block_instance("pulse_sink")
            .with_parameter("device_name", dev_name)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_weaver_usb_fc(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let audio_rate = if let Some(rate) = self.clone().into_inner().next() {
            rate.as_str()
        } else {
            bail!("missing mandatory <audio_rate> for weaver_usb_fc");
        };
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::F32)?
            .create_block_instance("weaver_usb_fc")
            .with_parameter("audio_rate", audio_rate)
            .assert_output(GrcItemType::C32)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_pulse_shaping_filter(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let mut inner = self.clone().into_inner();
        let filter_type = inner.next().map(|p| p.as_str()).unwrap_or("RRC");
        let alpha = inner.next().map(|p| p.as_str()).unwrap_or("0.35");
        let sps = inner.next().map(|p| p.as_str()).unwrap_or("4");

        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::C32)?
            .create_block_instance("pulse_shaping_filter_cc")
            .with_parameter("type", filter_type)
            .with_parameter("alpha", alpha)
            .with_parameter("sps", sps)
            .assert_output(GrcItemType::C32)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_firdes_pulse_shaping(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        self.build_pulse_shaping_filter(grc)
    }

    fn build_qpsk_modulator(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::U8)?
            .create_block_instance("qpsk_modulator_u8_c")
            .assert_output(GrcItemType::C32)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_oqpsk_modulator(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::U8)?
            .create_block_instance("oqpsk_modulator_u8_c")
            .assert_output(GrcItemType::C32)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_qpsk_demod(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::C32)?
            .create_block_instance("qpsk_demod_c_u8")
            .assert_output(GrcItemType::U8)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_oqpsk_demod(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::C32)?
            .create_block_instance("oqpsk_demod_c_u8")
            .assert_output(GrcItemType::U8)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_qpsk_costas_loop(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let mut inner = self.clone().into_inner();
        let loop_bw = inner.next().map(|p| p.as_str()).unwrap_or("0.02");

        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::C32)?
            .create_block_instance("qpsk_costas_loop_cc")
            .with_parameter("w", loop_bw)
            .assert_output(GrcItemType::C32)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_oqpsk_costas_loop(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let mut inner = self.clone().into_inner();
        let loop_bw = inner.next().map(|p| p.as_str()).unwrap_or("0.02");

        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::C32)?
            .create_block_instance("oqpsk_costas_loop_cc")
            .with_parameter("w", loop_bw)
            .assert_output(GrcItemType::C32)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_pdu_deframing_g3ruh(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::U8)?
            .create_block_instance("pdu_deframing_g3ruh")
            .assert_output(GrcItemType::Pdu)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_pdu_framing_g3ruh(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .create_block_instance("pdu_framing_g3ruh")
            .assert_output(GrcItemType::U8)
            .push()?;
        Ok(grc)
    }

    fn build_afsk1200_demod(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let sample_rate = self
            .clone()
            .into_inner()
            .next()
            .map(|p| p.as_str())
            .unwrap_or("48000");
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::F32)?
            .create_block_instance("afsk1200_demod_f_u8")
            .with_parameter("sample_rate", sample_rate)
            .assert_output(GrcItemType::U8)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_afsk1200_mod(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let sample_rate = self
            .clone()
            .into_inner()
            .next()
            .map(|p| p.as_str())
            .unwrap_or("48000");
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::U8)?
            .create_block_instance("afsk1200_mod_u8_f")
            .with_parameter("sample_rate", sample_rate)
            .assert_output(GrcItemType::F32)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_hdlc_deframing(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::U8)?
            .create_block_instance("hdlc_deframing_u8_pdu")
            .assert_output(GrcItemType::Pdu)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_hdlc_framing(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .create_block_instance("hdlc_framing_pdu_u8")
            .assert_output(GrcItemType::U8)
            .push()?;
        Ok(grc)
    }

    fn build_fir_interpolate(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut inner = self.clone().into_inner();
        let interp = inner.next().map(|p| p.as_str()).unwrap_or("1");
        let bandwidth = inner.next().map(|p| p.as_str()).unwrap_or("0.05");
        let window = inner.next().map(|p| p.as_str()).unwrap_or("HAMMING");

        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::C32)?
            .create_block_instance("fir_interpolate_cc")
            .with_parameter("interp", interp)
            .with_parameter("transition_bw", bandwidth)
            .with_parameter("window", window)
            .assert_output(GrcItemType::C32)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_bokeh_spectrum_sink(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let mut inner = self.clone().into_inner();
        let fft_size = inner.next().map(|p| p.as_str()).unwrap_or("4096");
        let fps = inner.next().map(|p| p.as_str()).unwrap_or("20");

        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::F32)?
            .create_block_instance("bokeh_spectrum_sink")
            .with_parameter("fft_size", fft_size)
            .with_parameter("fps", fps)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_synthetic_sdr_source(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let sample_rate = self
            .clone()
            .into_inner()
            .next()
            .map(|p| p.as_str())
            .unwrap_or("2000000");

        let mut grc = grc;
        grc = grc
            .create_block_instance("synthetic_sdr_source")
            .with_parameter("sample_rate", sample_rate)
            .assert_output(GrcItemType::C32)
            .push()?;
        Ok(grc)
    }
}

#[cfg(test)]
mod tests {
    use crate::cmd_grammar::CommandsParser;
    use crate::cmd_line::HighLevelCmdLine;
    use crate::config::HubConfig;
    use crate::csdr_cmd::CsdrCmd;

    #[test]
    fn test_parse_pipelines_from_all_configs() {
        let configs = [
            "examples/cfg-2m.toml",
            "examples/cfg-qo100.toml",
            "examples/cfg-qo100-mixed.toml",
            "examples/cfg-20m.toml",
        ];
        for cfg_path in configs {
            let config = HubConfig::load_from_file(cfg_path)
                .unwrap_or_else(|e| panic!("Failed to load {}: {:?}", cfg_path, e));

            for ch in &config.channels {
                if let Some(ref rx) = ch.rx {
                    if let Some(ref pipeline) = rx.pipeline {
                        let normalized = HubConfig::normalize_pipeline(pipeline);
                        let cmd_str = format!("csdr {}", normalized);
                        let parsed = CommandsParser::parse_main(&cmd_str).unwrap_or_else(|e| {
                            panic!(
                                "{}: channel {} RX failed parse: {:?}\nCommand: {}",
                                cfg_path, ch.id, e, cmd_str
                            )
                        });
                        let csdr_cmd = parsed.as_csdr_cmd().expect("must be csdr_cmd");
                        let grc_res = csdr_cmd.parse().unwrap_or_else(|e| {
                            panic!(
                                "{}: channel {} RX failed GRC build: {:?}\nCommand: {}",
                                cfg_path, ch.id, e, cmd_str
                            )
                        });
                        assert!(grc_res.is_some());
                    }
                }

                if let Some(ref tx) = ch.tx {
                    if tx.enabled.unwrap_or(true) {
                        if let Some(ref pipeline) = tx.pipeline {
                            let normalized = HubConfig::normalize_pipeline(pipeline);
                            let cmd_str = format!("csdr {}", normalized);
                            let parsed = CommandsParser::parse_main(&cmd_str).unwrap_or_else(|e| {
                                panic!(
                                    "{}: channel {} TX failed parse: {:?}\nCommand: {}",
                                    cfg_path, ch.id, e, cmd_str
                                )
                            });
                            let csdr_cmd = parsed.as_csdr_cmd().expect("must be csdr_cmd");
                            let grc_res = csdr_cmd.parse().unwrap_or_else(|e| {
                                panic!(
                                    "{}: channel {} TX failed GRC build: {:?}\nCommand: {}",
                                    cfg_path, ch.id, e, cmd_str
                                )
                            });
                            assert!(grc_res.is_some());
                        }
                    }
                }
            }

            if let Some(ref telemetry) = config.telemetry {
                if let Some(ref bokeh) = telemetry.bokeh {
                    for probe in &bokeh.probes {
                        if let Some(ref pipeline) = probe.pipeline {
                            let normalized = HubConfig::normalize_pipeline(pipeline);
                            let cmd_str = format!("csdr {}", normalized);
                            let parsed = CommandsParser::parse_main(&cmd_str).unwrap_or_else(|e| {
                                panic!(
                                    "{}: probe {} failed parse: {:?}\nCommand: {}",
                                    cfg_path, probe.name, e, cmd_str
                                )
                            });
                            let csdr_cmd = parsed.as_csdr_cmd().expect("must be csdr_cmd");
                            let grc_res = csdr_cmd.parse().unwrap_or_else(|e| {
                                panic!(
                                    "{}: probe {} failed GRC build: {:?}\nCommand: {}",
                                    cfg_path, probe.name, e, cmd_str
                                )
                            });
                            assert!(grc_res.is_some());
                        }
                    }
                }
            }
        }
    }
}
