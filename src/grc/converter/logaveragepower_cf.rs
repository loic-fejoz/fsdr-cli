use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use crate::blocks::LogAveragePowerCf;
use anyhow::Result;
use futuresdr::runtime::Flowgraph;

pub struct LogAveragePowerCfConverter {}

impl BlockConverter for LogAveragePowerCfConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let fft_size = Grc2FutureSdr::parameter_as_f64(blk, "fft_size", "512")? as usize;
        let avg_number = Grc2FutureSdr::parameter_as_f64(blk, "avg_number", "10")? as usize;
        let add_db = Grc2FutureSdr::parameter_as_f64(blk, "add_db", "0.0")? as f32;
        let block = LogAveragePowerCf::new(fft_size, avg_number, add_db);
        let id = fg.add(block)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
