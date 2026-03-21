use crate::grc::backend::FsdrBackend;
use crate::grc::converter_helper::{ConnectorAdapter, MutBlockConverter};
use crate::grc::BlockInstance;
use anyhow::{bail, Result};
use futuresdr::runtime::BlockId;

pub struct IQEngineOutputBlockConverter {
    pub blk: Option<BlockId>,
}

impl IQEngineOutputBlockConverter {
    pub fn new() -> IQEngineOutputBlockConverter {
        IQEngineOutputBlockConverter { blk: None }
    }
}

impl Default for IQEngineOutputBlockConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: FsdrBackend> MutBlockConverter<B> for IQEngineOutputBlockConverter {
    fn convert(
        &mut self,
        _blk: &BlockInstance,
        _backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        if let Some(blk) = self.blk {
            // Manual cast for PoC
            let blk_id: B::BlockRef = unsafe { std::mem::transmute_copy(&blk) };
            let s: Box<dyn ConnectorAdapter<B::BlockRef>> =
                Box::new(IQEngineConnectorAdapter { blk: blk_id });
            return Ok(s);
        }
        bail!("IQEngineOutputBlockConverter: block not set")
    }

    fn downcast_iqengine(&self) -> Option<&IQEngineOutputBlockConverter> {
        Some(self)
    }
}

pub struct IQEngineConnectorAdapter<BlockRef: Clone> {
    pub blk: BlockRef,
}

impl<BlockRef: Clone> ConnectorAdapter<BlockRef> for IQEngineConnectorAdapter<BlockRef> {
    fn adapt_input_port(&self, port_name: &str) -> Result<(BlockRef, &str)> {
        match port_name {
            "0" => Ok((self.blk.clone(), "input")),
            "in" | "input" => Ok((self.blk.clone(), "input")),
            _ => bail!("Unknown input port name {port_name}"),
        }
    }

    fn adapt_output_port(&self, _port_name: &str) -> Result<(BlockRef, &str)> {
        bail!("IQEngineOutputBlockConverter: no output port")
    }
}
