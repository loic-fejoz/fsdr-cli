use crate::cmd_grammar::CommandsParser;
use crate::csdr_cmd::eval_cmd::EvalCmd;
use crate::iqengine_blockconverter::IQEngineOutputBlockConverter;

use super::BlockInstance;
use anyhow::{anyhow, bail, Result};
use futuresdr::runtime::{BlockId, Flowgraph};

/// Evaluate a string as either a raw float or a parsed mathematical expression.
pub fn eval_expr_str(s: &str) -> Result<f32> {
    let trimmed = s.trim().trim_matches('"').trim_matches('\'');
    if let Ok(val) = trimmed.parse::<f32>() {
        return Ok(val);
    }
    let expr = CommandsParser::parse_expr(trimmed)?;
    EvalCmd::eval(&expr)
}

/// Parse a value in linear scale or in decibels (ending with `dB` or `db`).
pub fn parse_db_or_linear(raw: &str) -> Result<f32> {
    let s = raw.trim().trim_matches('"').trim_matches('\'');
    if let Some(stripped) = s.strip_suffix("dB").or_else(|| s.strip_suffix("db")) {
        let db = eval_expr_str(stripped)?;
        Ok(10.0f32.powf(db / 10.0))
    } else {
        eval_expr_str(s)
    }
}

/// Parse a duration in milliseconds (`ms`), seconds (`s`), or direct sample count.
pub fn parse_duration_samples(raw: &str, samp_rate: f64) -> Result<usize> {
    let s = raw.trim().trim_matches('"').trim_matches('\'');
    if let Some(stripped) = s.strip_suffix("ms") {
        let ms = eval_expr_str(stripped)? as f64;
        Ok(((ms / 1000.0) * samp_rate).round() as usize)
    } else if let Some(stripped) = s.strip_suffix('s') {
        let sec = eval_expr_str(stripped)? as f64;
        Ok((sec * samp_rate).round() as usize)
    } else {
        Ok(eval_expr_str(s)?.round() as usize)
    }
}

/// Do the actual conversion from GNU Radio block description into
/// one or several FutureSDR block.
/// Return an helper that in case of hierarchical block know how to convert port name and block id
pub trait BlockConverter {
    fn convert(&self, blk: &BlockInstance, fg: &mut Flowgraph)
        -> Result<Box<dyn ConnectorAdapter>>;
}

pub trait MutBlockConverter {
    fn convert(
        &mut self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>>;

    #[allow(dead_code)]
    fn downcast_iqengine(&self) -> Option<&IQEngineOutputBlockConverter> {
        None
    }
}

/// Convert GNU Radio port's name into actual FutureSDR block id and port name.
pub trait ConnectorAdapter {
    /// Convert the name of a port into actual block id and port name
    fn adapt_input_port(&self, port_name: &str) -> Result<(BlockId, &str)>;

    /// Convert the name of a port into actual block id and port name
    fn adapt_output_port(&self, port_name: &str) -> Result<(BlockId, &str)>;
}

#[derive(Clone, Copy)]
pub struct DefaultPortAdapter {
    blk: BlockId,
}

impl DefaultPortAdapter {
    pub fn new(blk: BlockId) -> DefaultPortAdapter {
        DefaultPortAdapter { blk }
    }
}

impl ConnectorAdapter for DefaultPortAdapter {
    fn adapt_input_port(&self, port_name: &str) -> Result<(BlockId, &str)> {
        match port_name {
            "0" => Ok((self.blk, "input")),
            "in" | "input" => Ok((self.blk, "input")),
            _ => bail!("Unknown input port name {port_name}"),
        }
    }

    fn adapt_output_port(&self, port_name: &str) -> Result<(BlockId, &str)> {
        match port_name {
            "0" => Ok((self.blk, "output")),
            "out" | "output" => Ok((self.blk, "output")),
            _ => bail!("Unknown output port name {port_name}"),
        }
    }
}

pub type BlockFactory = Box<dyn FnOnce(&mut Flowgraph) -> BlockId>;

pub struct PredefinedBlockConverter {
    value: Option<BlockFactory>,
}

impl PredefinedBlockConverter {
    #[allow(dead_code)]
    pub fn new<F>(f: F) -> PredefinedBlockConverter
    where
        F: FnOnce(&mut Flowgraph) -> BlockId + 'static,
    {
        PredefinedBlockConverter {
            value: Some(Box::new(f)),
        }
    }
}

impl MutBlockConverter for PredefinedBlockConverter {
    fn convert(
        &mut self,
        _blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        if let Some(res) = self.value.take() {
            let blk = res(fg);
            let s: Box<dyn ConnectorAdapter> = Box::new(DefaultPortAdapter::new(blk));
            return Ok(s);
        }
        Err(anyhow!(
            "Value already picked: probably too many time the same block type."
        ))
    }
}
