use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use crate::blocks::FftExchangeSidesFf;
use anyhow::Result;
use futuresdr::runtime::Flowgraph;

pub struct FftExchangeSidesFfConverter {}

impl BlockConverter for FftExchangeSidesFfConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let fft_size = Grc2FutureSdr::parameter_as_f64(blk, "fft_size", "512")? as usize;
        let block = FftExchangeSidesFf::new(fft_size);
        let id = fg.add(block)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
