use super::super::converter_helper::{
    parse_duration_samples, BlockConverter, ConnectorAdapter, DefaultPortAdapter,
};
use super::{BlockInstance, Grc2FutureSdr};
use crate::blocks::TimerTagger;
use anyhow::{bail, Result};
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;

pub struct BlocksTimerTaggerConverter {}

impl BlockConverter for BlocksTimerTaggerConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let samp_rate = Grc2FutureSdr::parameter_as_f64(blk, "samp_rate", "48000.0")?;
        let duration_str = blk.parameter_or("duration", "30s");
        let duration_samples = parse_duration_samples(duration_str, samp_rate)?;

        let start_tag = blk
            .parameter_or("start", "msgstart")
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();
        let end_tag = blk
            .parameter_or("end", "msgend")
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();

        let debug = blk
            .parameters
            .get("debug")
            .map(|s| s == "true" || s == "1")
            .unwrap_or(false);

        let item_type = blk.parameter_or("type", "fff");
        let adapter: Box<dyn ConnectorAdapter> = match item_type {
            "fff" | "f32" | "float" => {
                let block =
                    TimerTagger::<f32>::with_debug(start_tag, end_tag, duration_samples, debug);
                let id = fg.add(block)?.id();
                Box::new(DefaultPortAdapter::new(id))
            }
            "ccc" | "c32" | "complex" => {
                let block = TimerTagger::<Complex32>::with_debug(
                    start_tag,
                    end_tag,
                    duration_samples,
                    debug,
                );
                let id = fg.add(block)?.id();
                Box::new(DefaultPortAdapter::new(id))
            }
            _ => bail!("blocks_timer_tagger: Unsupported type {item_type}"),
        };

        Ok(adapter)
    }
}
