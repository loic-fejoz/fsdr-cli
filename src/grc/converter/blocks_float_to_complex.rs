use super::super::converter_helper::{BlockConverter, ConnectorAdapter};
use super::BlockInstance;
use crate::grc::backend::FsdrBackend;
use anyhow::{bail, Result};
use futuresdr::blocks::Combine;
use futuresdr::num_complex::Complex32;

#[derive(Clone, Copy)]
pub struct FloatToComplexPortAdapter<BlockRef: Clone> {
    blk: BlockRef,
}

impl<BlockRef: Clone> FloatToComplexPortAdapter<BlockRef> {
    pub fn new(blk: BlockRef) -> FloatToComplexPortAdapter<BlockRef> {
        FloatToComplexPortAdapter { blk }
    }
}

impl<BlockRef: Clone> ConnectorAdapter<BlockRef> for FloatToComplexPortAdapter<BlockRef> {
    fn adapt_input_port(&self, port_name: &str) -> Result<(BlockRef, &str)> {
        match port_name {
            "0" | "in0" => Ok((self.blk.clone(), "in0")),
            "1" | "in1" => Ok((self.blk.clone(), "in1")),
            _ => bail!("Unknown input port name {port_name}"),
        }
    }

    fn adapt_output_port(&self, port_name: &str) -> Result<(BlockRef, &str)> {
        match port_name {
            "0" | "out" | "output" => Ok((self.blk.clone(), "out")),
            _ => bail!("Unknown output port name {port_name}"),
        }
    }
}

pub struct FloatToComplexConverter {}

impl<B: FsdrBackend> BlockConverter<B> for FloatToComplexConverter {
    fn convert(
        &self,
        _blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let blk: Combine<_, f32, f32, Complex32> =
            Combine::new(|v1: &f32, v2: &f32| -> Complex32 { Complex32::new(*v1, *v2) });
        let blk_ref = backend.add_block_runtime(blk)?;
        Ok(Box::new(FloatToComplexPortAdapter::new(blk_ref)))
    }
}
