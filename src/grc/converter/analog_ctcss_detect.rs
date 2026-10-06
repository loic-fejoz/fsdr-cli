use super::super::converter_helper::{
    parse_duration_samples, BlockConverter, ConnectorAdapter, DefaultPortAdapter,
};
use super::{BlockInstance, Grc2FutureSdr};
use crate::blocks::CtcssDetect;
use anyhow::Result;
use futuresdr::runtime::Flowgraph;

pub struct AnalogCtcssDetectConverter {}

impl BlockConverter for AnalogCtcssDetectConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let samp_rate = Grc2FutureSdr::parameter_as_f64(blk, "samp_rate", "48000.0")?;
        let tone_freq = Grc2FutureSdr::parameter_as_f64(blk, "tone", "88.5")? as f32;
        let threshold = Grc2FutureSdr::parameter_as_f64(blk, "threshold", "0.01")? as f32;

        let duration_samples = if let Some(d) = blk.parameters.get("duration") {
            parse_duration_samples(d, samp_rate)?
        } else {
            parse_duration_samples("150ms", samp_rate)?
        };

        let tag_name = blk
            .parameters
            .get("tag")
            .cloned()
            .unwrap_or_else(|| "msgstart".to_string())
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();

        let debug = blk
            .parameters
            .get("debug")
            .map(|s| s == "true" || s == "1")
            .unwrap_or(false);

        let block = CtcssDetect::with_debug(
            samp_rate as f32,
            tone_freq,
            threshold,
            duration_samples,
            tag_name,
            debug,
        );
        let id = fg.add(block)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
