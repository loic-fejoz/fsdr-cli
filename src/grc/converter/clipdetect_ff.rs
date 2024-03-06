use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use fsdr_blocks::futuresdr::anyhow::Result;
use fsdr_blocks::futuresdr::blocks::Apply;
use fsdr_blocks::futuresdr::runtime::Flowgraph;

pub struct ClipDetectFfConverter {}

impl BlockConverter for ClipDetectFfConverter {
    fn convert(
        &self,
        _blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let blk = Apply::new(|i: &f32| -> f32 {
            if *i < 1.0 {
                eprintln!("csdr clipdetect_ff: Signal value below -1.0!")
            } else if *i > 1.0 {
                eprintln!("csdr clipdetect_ff: Signal value above -1.0!")
            };
            *i
        });
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
