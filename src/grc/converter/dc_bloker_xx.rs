use crate::blocks::DCBlocker;
use crate::grc::backend::FsdrBackend;

use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{parameter_as_f64, BlockInstance};
use anyhow::Result;

pub struct DcBlockerXx {}

impl<B: FsdrBackend> BlockConverter<B> for DcBlockerXx {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let min_bufsize = parameter_as_f64(blk, "length", "32")? as usize;
        let blk = DCBlocker::<f32>::new(min_bufsize);
        let blk_ref = backend.add_block_runtime(blk)?;
        Ok(Box::new(DefaultPortAdapter::new(blk_ref)))
    }
}
