use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use crate::blocks::CatServer;
use anyhow::Result;
use futuresdr::runtime::Flowgraph;

pub struct CatServerConverter {}

impl BlockConverter for CatServerConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let port = Grc2FutureSdr::parameter_as_f64(blk, "port", "4532")? as u16;
        let rx_freq = Grc2FutureSdr::parameter_as_f64(blk, "rx_freq", "144500000")? as u64;
        let tx_freq = Grc2FutureSdr::parameter_as_f64(blk, "tx_freq", "144500000")? as u64;
        let ctcss = Grc2FutureSdr::parameter_as_f64(blk, "ctcss_tone", "885")? as i32;
        let blk = CatServer::new(port, rx_freq, tx_freq, ctcss)?;
        let blk = fg.add(blk)?.id();
        let adapter = DefaultPortAdapter::new(blk);
        Ok(Box::new(adapter))
    }
}
