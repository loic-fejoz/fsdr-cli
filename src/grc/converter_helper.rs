use crate::grc::backend::FsdrBackend;
use crate::iqengine_blockconverter::IQEngineOutputBlockConverter;

use super::BlockInstance;
use anyhow::{bail, Result};

/// Do the actual conversion from GNU Radio block description into
/// one or several FutureSDR block.
/// Return an helper that in case of hierarchical block know how to convert port name and block id
pub trait BlockConverter<B: FsdrBackend> {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>>;
}

pub trait MutBlockConverter<B: FsdrBackend> {
    fn convert(
        &mut self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>>;

    #[allow(dead_code)]
    fn downcast_iqengine(&self) -> Option<&IQEngineOutputBlockConverter> {
        None
    }
}

/// Convert GNU Radio port's name into actual FutureSDR block id and port name.
pub trait ConnectorAdapter<BlockRef: Clone> {
    /// Convert the name of a port into actual block id and port name
    fn adapt_input_port(&self, port_name: &str) -> Result<(BlockRef, &str)>;

    /// Convert the name of a port into actual block id and port name
    fn adapt_output_port(&self, port_name: &str) -> Result<(BlockRef, &str)>;
}

#[derive(Clone, Copy)]
pub struct DefaultPortAdapter<BlockRef: Clone> {
    blk: BlockRef,
}

impl<BlockRef: Clone> DefaultPortAdapter<BlockRef> {
    pub fn new(blk: BlockRef) -> DefaultPortAdapter<BlockRef> {
        DefaultPortAdapter { blk }
    }
}

impl<BlockRef: Clone> ConnectorAdapter<BlockRef> for DefaultPortAdapter<BlockRef> {
    fn adapt_input_port(&self, port_name: &str) -> Result<(BlockRef, &str)> {
        match port_name {
            "0" => Ok((self.blk.clone(), "input")),
            "in" | "input" => Ok((self.blk.clone(), "input")),
            _ => bail!("Unknown input port name {port_name}"),
        }
    }

    fn adapt_output_port(&self, port_name: &str) -> Result<(BlockRef, &str)> {
        match port_name {
            "0" => Ok((self.blk.clone(), "output")),
            "out" | "output" => Ok((self.blk.clone(), "output")),
            _ => bail!("Unknown output port name {port_name}"),
        }
    }
}

pub type BlockFactory<B> = Box<dyn FnOnce(&mut B) -> <B as FsdrBackend>::BlockRef>;

pub struct PredefinedBlockConverter<B: FsdrBackend> {
    value: Option<BlockFactory<B>>,
}

impl<B: FsdrBackend> PredefinedBlockConverter<B> {
    #[allow(dead_code)]
    pub fn new<F>(f: F) -> PredefinedBlockConverter<B>
    where
        F: FnOnce(&mut B) -> B::BlockRef + 'static,
    {
        PredefinedBlockConverter {
            value: Some(Box::new(f)),
        }
    }
}

impl<B: FsdrBackend> MutBlockConverter<B> for PredefinedBlockConverter<B> {
    fn convert(
        &mut self,
        _blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        if let Some(res) = self.value.take() {
            let blk = res(backend);
            let s: Box<dyn ConnectorAdapter<B::BlockRef>> = Box::new(DefaultPortAdapter::new(blk));
            return Ok(s);
        }
        bail!("Value already picked: probably too many time the same block type.")
    }
}
