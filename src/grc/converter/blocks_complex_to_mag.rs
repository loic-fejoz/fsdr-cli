use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use futuresdr::anyhow::Result;
use futuresdr::blocks::Apply;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;

pub struct ComplexToMagConverter {}

impl BlockConverter for ComplexToMagConverter {
    fn convert(
        &self,
        _blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let blk = Apply::new(|i: &Complex32| -> f32 { i.norm() });
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
