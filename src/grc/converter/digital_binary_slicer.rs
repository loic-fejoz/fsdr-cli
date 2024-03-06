use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use fsdr_blocks::futuresdr::anyhow::Result;
use fsdr_blocks::futuresdr::blocks::Apply;
use fsdr_blocks::futuresdr::runtime::Flowgraph;

pub struct DigitalBinarySlicerConverter {}

impl BlockConverter for DigitalBinarySlicerConverter {
    fn convert(
        &self,
        _blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let blk = Apply::new(move |v: &f32| -> u8 { (*v).ge(&0.0f32).into() });
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
