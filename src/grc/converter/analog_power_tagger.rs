use super::super::converter_helper::{
    parse_db_or_linear, parse_duration_samples, BlockConverter, ConnectorAdapter,
    DefaultPortAdapter,
};
use super::{BlockInstance, Grc2FutureSdr};
use crate::blocks::PowerTagger;
use anyhow::{bail, Result};
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;

pub struct AnalogPowerTaggerConverter {}

impl BlockConverter for AnalogPowerTaggerConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let samp_rate = Grc2FutureSdr::parameter_as_f64(blk, "samp_rate", "2400000.0")?;

        let gen_threshold = blk.parameters.get("threshold");
        let gen_delay = blk.parameters.get("delay");

        let off_threshold = if let Some(t) = blk.parameters.get("off_threshold").or(gen_threshold) {
            Some(parse_db_or_linear(t)?)
        } else {
            None
        };

        let on_threshold = if let Some(t) = blk.parameters.get("on_threshold").or(gen_threshold) {
            Some(parse_db_or_linear(t)?)
        } else {
            None
        };

        let off_delay = if let Some(d) = blk.parameters.get("off_delay").or(gen_delay) {
            parse_duration_samples(d, samp_rate)?
        } else {
            parse_duration_samples("300ms", samp_rate)?
        };

        let on_delay = if let Some(d) = blk.parameters.get("on_delay").or(gen_delay) {
            parse_duration_samples(d, samp_rate)?
        } else {
            parse_duration_samples("300ms", samp_rate)?
        };

        let window_size = if let Some(w) = blk.parameters.get("window_size") {
            parse_duration_samples(w, samp_rate)?
        } else {
            parse_duration_samples("10ms", samp_rate)?
        };

        let off_tag = blk
            .parameters
            .get("off_tag")
            .map(|s| s.trim_matches('"').trim_matches('\'').to_string());
        let on_tag = blk
            .parameters
            .get("on_tag")
            .map(|s| s.trim_matches('"').trim_matches('\'').to_string());
        let debug = blk
            .parameters
            .get("debug")
            .map(|s| s == "true" || s == "1")
            .unwrap_or(false);

        let snr_off_ratio = if let Some(s) = blk.parameters.get("snr_off") {
            Some(parse_db_or_linear(s)?)
        } else {
            None
        };

        let snr_on_ratio = if let Some(s) = blk.parameters.get("snr_on") {
            Some(parse_db_or_linear(s)?)
        } else {
            None
        };

        let noise_alpha = if let Some(a) = blk.parameters.get("noise_alpha") {
            Some(super::super::converter_helper::eval_expr_str(a)?)
        } else {
            None
        };

        let item_type = blk.parameter_or("type", "ccc");
        let adapter: Box<dyn ConnectorAdapter> = match item_type {
            "ccc" | "c32" | "complex" => {
                let block = PowerTagger::<Complex32>::with_full_options(
                    off_threshold,
                    off_delay,
                    on_threshold,
                    on_delay,
                    window_size,
                    off_tag,
                    on_tag,
                    debug,
                    snr_off_ratio,
                    snr_on_ratio,
                    noise_alpha,
                );
                let id = fg.add(block)?.id();
                Box::new(DefaultPortAdapter::new(id))
            }
            "fff" | "f32" | "float" => {
                let block = PowerTagger::<f32>::with_full_options(
                    off_threshold,
                    off_delay,
                    on_threshold,
                    on_delay,
                    window_size,
                    off_tag,
                    on_tag,
                    debug,
                    snr_off_ratio,
                    snr_on_ratio,
                    noise_alpha,
                );
                let id = fg.add(block)?.id();
                Box::new(DefaultPortAdapter::new(id))
            }
            _ => bail!("analog_power_tagger: Unsupported type {item_type}"),
        };

        Ok(adapter)
    }
}
