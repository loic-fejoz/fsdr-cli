use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use crate::blocks::LogPowerCf;
use anyhow::Result;
use futuresdr::runtime::Flowgraph;

pub struct LogPowerCfConverter {}

impl BlockConverter for LogPowerCfConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let add_db = Grc2FutureSdr::parameter_as_f64(blk, "add_db", "0.0")? as f32;
        let block = LogPowerCf::new(add_db);
        let id = fg.add(block)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
