use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use crate::blocks::CmdTrigger;
use anyhow::{bail, Result};
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;

pub struct BlocksCmdTriggerConverter {}

impl BlockConverter for BlocksCmdTriggerConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let samp_rate = Grc2FutureSdr::parameter_as_f64(blk, "samp_rate", "48000.0")?;
        let start_tag = blk
            .parameter_or("start_tag", "msgstart")
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();
        let end_tag = blk
            .parameter_or("end_tag", "msgend")
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();
        let cmd = blk
            .parameter_or("cmd", "./script.sh $input_file")
            .trim_matches('\'')
            .to_string();
        let debug = blk
            .parameters
            .get("debug")
            .map(|s| s == "true" || s == "1")
            .unwrap_or(false);

        let item_type = blk.parameter_or("type", "f32");
        let adapter: Box<dyn ConnectorAdapter> = match item_type {
            "fff" | "f32" | "float" => {
                let block =
                    CmdTrigger::<f32>::with_debug(start_tag, end_tag, cmd, samp_rate, debug);
                let id = fg.add(block)?.id();
                Box::new(DefaultPortAdapter::new(id))
            }
            "ccc" | "c32" | "complex" => {
                let block =
                    CmdTrigger::<Complex32>::with_debug(start_tag, end_tag, cmd, samp_rate, debug);
                let id = fg.add(block)?.id();
                Box::new(DefaultPortAdapter::new(id))
            }
            _ => bail!("blocks_cmd_trigger: Unsupported type {item_type}"),
        };

        Ok(adapter)
    }
}
