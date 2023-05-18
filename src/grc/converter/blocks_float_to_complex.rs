use super::super::converter_helper::{BlockConverter, ConnectorAdapter};
use super::BlockInstance;
use futuresdr::anyhow::{bail, Result};
use futuresdr::blocks::Combine;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;

#[derive(Clone, Copy)]
pub struct FloatToComplexPortAdapter {
    blk: usize,
}

impl FloatToComplexPortAdapter {
    pub fn new(blk: usize) -> FloatToComplexPortAdapter {
        FloatToComplexPortAdapter { blk }
    }
}

impl ConnectorAdapter for FloatToComplexPortAdapter {
    fn adapt_input_port(&self, port_name: &str) -> Result<(usize, &str)> {
        match port_name {
            "0" => Ok((self.blk, "in0")),
            "1" => Ok((self.blk, "in1")),
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

pub struct FloatToComplexConverter {}

impl BlockConverter for FloatToComplexConverter {
    fn convert(
        &self,
        _blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let blk = Combine::new(|v1: &f32, v2: &f32| -> Complex32 { Complex32::new(*v1, *v2) });
        let blk = fg.add_block(blk);
        let blk = FloatToComplexPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
