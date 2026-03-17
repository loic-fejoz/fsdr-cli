use crate::iqengine_blockconverter::IQEngineOutputBlockConverter;

use super::BlockInstance;
use anyhow::{anyhow, bail, Result};
use futuresdr::runtime::{BlockId, Flowgraph};

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
