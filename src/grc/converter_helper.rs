use crate::iqengine_blockconverter::IQEngineOutputBlockConverter;

use super::BlockInstance;
use futuresdr::anyhow::{anyhow, bail, Result};
use futuresdr::runtime::{Block, Flowgraph};

/// Do the actual conversion from GNU Radio block description into
/// one or several FutureSDR block.
/// Return an helper that in case of hierarchical block know how to convert port name and block id
pub trait BlockConverter {
    fn convert(&self, blk: &BlockInstance, fg: &mut Flowgraph)
        -> Result<Box<dyn ConnectorAdapter>>;
}

pub trait MutBlockConverter {
    fn convert(&mut self, blk: &BlockInstance, fg: &mut Flowgraph)
        -> Result<Box<dyn ConnectorAdapter>>;

    fn downcast_iqengine(&self) -> Option<&IQEngineOutputBlockConverter> {
        None
    }
}

/// Convert GNU Radio port's name into actual FutureSDR block id and port name.
pub trait ConnectorAdapter {
    /// Convert the name of a port into actual block id and port name
    fn adapt_input_port(&self, port_name: &str) -> Result<(usize, &str)>;

    /// Convert the name of a port into actual block id and port name
    fn adapt_output_port(&self, port_name: &str) -> Result<(usize, &str)>;
}

#[derive(Clone, Copy)]
pub struct DefaultPortAdapter {
    blk: usize,
}

impl DefaultPortAdapter {
    pub fn new(blk: usize) -> DefaultPortAdapter {
        DefaultPortAdapter { blk }
    }
}

impl ConnectorAdapter for DefaultPortAdapter {
    fn adapt_input_port(&self, port_name: &str) -> Result<(usize, &str)> {
        match port_name {
            "0" => Ok((self.blk, "in")),
            "in" => Ok((self.blk, "in")),
            _ => bail!("Unknown input port name {port_name}"),
        }
    }

    fn adapt_output_port(&self, port_name: &str) -> Result<(usize, &str)> {
        match port_name {
            "0" => Ok((self.blk, "out")),
            "out" => Ok((self.blk, "out")),
            _ => bail!("Unknown output port name {port_name}"),
        }
    }
}

pub struct PredefinedBlockConverter {
    value: Option<Block>
}

impl PredefinedBlockConverter {
    pub fn new(blk: Block) -> PredefinedBlockConverter {
        PredefinedBlockConverter {
            value: Some(blk)
        }
    }
}

impl MutBlockConverter for PredefinedBlockConverter {
    fn convert(&mut self, _blk: &BlockInstance, fg: &mut Flowgraph)
        -> Result<Box<dyn ConnectorAdapter>> {
            if let Some(res) = self.value.take() {
                let blk = fg.add_block(res);
                let s: Box<dyn ConnectorAdapter> = Box::new(DefaultPortAdapter::new(blk));
                return Ok(s);
            }
            Err(anyhow!("Value already picked: probably too many time the same block type."))
    }
}