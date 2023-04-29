use super::super::converter_helper::{BlockConverter, ConnectorAdapter};
use super::BlockInstance;
use fsdr_blocks::stream::Deinterleave;
use futuresdr::anyhow::{bail, Result};
use futuresdr::num_complex::Complex32;
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
        let blk = match &(item_type[..]) {
            "char" => Deinterleave::<u8>::new(),
            "short" => Deinterleave::<i16>::new(),
            "float" => Deinterleave::<f32>::new(),
            "complex" => Deinterleave::<Complex32>::new(),
            _ => bail!("Unhandled blocks_deinterleave Type {item_type}"),
        };
        let blk = fg.add_block(blk);
        let blk = DeinterleavePortAdapter { blk };
        let blk = Box::new(blk);
        Ok(blk)
    }
}

#[derive(Clone, Copy)]
pub struct DeinterleavePortAdapter {
    blk: usize,
}

impl ConnectorAdapter for DeinterleavePortAdapter {
    fn adapt_input_port(&self, port_name: &str) -> Result<(usize, &str)> {
        match port_name {
            "0" => Ok((self.blk, "in")),
            _ => bail!("Unknown input port name {port_name} for blocks_deinterleave "),
        }
    }

    fn adapt_output_port(&self, port_name: &str) -> Result<(usize, &str)> {
        match port_name {
            "0" => Ok((self.blk, "out0")),
            "1" => Ok((self.blk, "out1")),
            _ => bail!("Unknown output port name {port_name} for blocks_deinterleave "),
        }
    }
}
