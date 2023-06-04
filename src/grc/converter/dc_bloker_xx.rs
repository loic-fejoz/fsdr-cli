use crate::blocks::DCBlocker;

use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use futuresdr::anyhow::Result;
use futuresdr::runtime::Flowgraph;

pub struct DcBlockerXx {}

impl BlockConverter for DcBlockerXx {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let min_bufsize = Grc2FutureSdr::parameter_as_f64(blk, "length", "32")? as usize;
        let blk = DCBlocker::<f32>::build(min_bufsize);
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
