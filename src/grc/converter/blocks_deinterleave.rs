use super::super::converter_helper::{BlockConverter, ConnectorAdapter};
use super::BlockInstance;
use anyhow::{bail, Result};
use fsdr_blocks::stream::Deinterleave;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::BlockId;
use futuresdr::runtime::Flowgraph;

pub struct DeinterleaveBlockConverter {}

impl BlockConverter for DeinterleaveBlockConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let item_type = blk
            .parameters
            .get("type")
            .expect("item type must be defined");
        let blk: Box<dyn ConnectorAdapter> = match &(item_type[..]) {
            "char" => {
                let blk = Deinterleave::<u8>::new();
                Box::new(DeinterleavePortAdapter {
                    blk: fg.add_block(blk).into(),
                })
            }
            "short" => {
                let blk = Deinterleave::<i16>::new();
                Box::new(DeinterleavePortAdapter {
                    blk: fg.add_block(blk).into(),
                })
            }
            "float" => {
                let blk = Deinterleave::<f32>::new();
                Box::new(DeinterleavePortAdapter {
                    blk: fg.add_block(blk).into(),
                })
            }
            "complex" => {
                let blk = Deinterleave::<Complex32>::new();
                Box::new(DeinterleavePortAdapter {
                    blk: fg.add_block(blk).into(),
                })
            }
            _ => bail!("Unhandled blocks_deinterleave Type {item_type}"),
        };
        Ok(blk)
    }
}

#[derive(Clone, Copy)]
pub struct DeinterleavePortAdapter {
    blk: BlockId,
}

impl ConnectorAdapter for DeinterleavePortAdapter {
    fn adapt_input_port(&self, port_name: &str) -> Result<(BlockId, &str)> {
        match port_name {
            "0" => Ok((self.blk, "in")),
            _ => bail!("Unknown input port name {port_name} for blocks_deinterleave "),
        }
    }

    fn adapt_output_port(&self, port_name: &str) -> Result<(BlockId, &str)> {
        match port_name {
            "0" => Ok((self.blk, "out0")),
            "1" => Ok((self.blk, "out1")),
            _ => bail!("Unknown output port name {port_name} for blocks_deinterleave "),
        }
    }
}
