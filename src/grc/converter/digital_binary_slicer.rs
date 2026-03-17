use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use anyhow::{Result, Context};
use futuresdr::blocks::Apply;
use futuresdr::runtime::Flowgraph;

pub struct DigitalBinarySlicerConverter {}

impl BlockConverter for DigitalBinarySlicerConverter {
    fn convert(
        &self,
        _blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let blk = Apply::<_, f32, u8>::new(move |v: &f32| -> u8 { (*v).ge(&0.0f32).into() });
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk.into());
        let blk = Box::new(blk);
        Ok(blk)
    }
}
