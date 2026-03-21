use super::super::converter_helper::{BlockConverter, ConnectorAdapter};
use super::BlockInstance;
use crate::grc::backend::FsdrBackend;
use anyhow::{bail, Result};
use fsdr_blocks::stream::Deinterleave;
use futuresdr::num_complex::Complex32;

pub struct DeinterleaveBlockConverter {}

impl<B: FsdrBackend> BlockConverter<B> for DeinterleaveBlockConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let item_type = blk.parameter_or("type", "float");

        let adapter: Box<dyn ConnectorAdapter<B::BlockRef>> = match &item_type[..] {
            "float" => {
                let blk = Deinterleave::<f32>::new();
                let blk_ref = backend.add_block_runtime(blk)?;
                Box::new(DeinterleavePortAdapter { blk: blk_ref })
            }
            "complex" => {
                let blk = Deinterleave::<Complex32>::new();
                let blk_ref = backend.add_block_runtime(blk)?;
                Box::new(DeinterleavePortAdapter { blk: blk_ref })
            }
            "short" => {
                let blk = Deinterleave::<i16>::new();
                let blk_ref = backend.add_block_runtime(blk)?;
                Box::new(DeinterleavePortAdapter { blk: blk_ref })
            }
            "byte" => {
                let blk = Deinterleave::<u8>::new();
                let blk_ref = backend.add_block_runtime(blk)?;
                Box::new(DeinterleavePortAdapter { blk: blk_ref })
            }
            _ => bail!("blocks_deinterleave: Unhandled type {item_type}"),
        };
        Ok(adapter)
    }
}

pub struct DeinterleavePortAdapter<BlockRef: Clone> {
    pub blk: BlockRef,
}

impl<BlockRef: Clone> ConnectorAdapter<BlockRef> for DeinterleavePortAdapter<BlockRef> {
    fn adapt_input_port(&self, port_name: &str) -> Result<(BlockRef, &str)> {
        match port_name {
            "0" => Ok((self.blk.clone(), "input")),
            _ => bail!("Unknown input port name {port_name}"),
        }
    }

    fn adapt_output_port(&self, port_name: &str) -> Result<(BlockRef, &str)> {
        match port_name {
            "0" => Ok((self.blk.clone(), "output0")),
            "1" => Ok((self.blk.clone(), "output1")),
            _ => bail!("Unknown output port name {port_name}"),
        }
    }
}
