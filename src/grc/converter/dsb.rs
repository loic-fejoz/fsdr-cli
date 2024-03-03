use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use futuresdr::anyhow::Result;
use futuresdr::blocks::Apply;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;

pub struct DsbConverter {}

impl BlockConverter for DsbConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let q_value = Grc2FutureSdr::parameter_as_f32(blk, "q_value", "0.0")? as f32;
        let blk = Apply::new(move |v: &f32| -> Complex32 { Complex32::new(*v, q_value) });
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
