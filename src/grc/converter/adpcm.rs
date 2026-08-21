use crate::blocks::{AdpcmDecoderU8I16, AdpcmEncoderI16U8, CompressFftAdpcmFU8};
use crate::grc::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use crate::grc::BlockInstance;
use anyhow::{Context, Result};
use futuresdr::runtime::Flowgraph;

pub struct AdpcmEncoderConverter {}

impl BlockConverter for AdpcmEncoderConverter {
    fn convert(
        &self,
        _blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let blk = AdpcmEncoderI16U8::new();
        let id = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}

pub struct AdpcmDecoderConverter {}

impl BlockConverter for AdpcmDecoderConverter {
    fn convert(
        &self,
        _blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let blk = AdpcmDecoderU8I16::new();
        let id = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}

pub struct CompressFftAdpcmConverter {}

impl BlockConverter for CompressFftAdpcmConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let fft_size: usize = blk
            .parameters
            .get("fft_size")
            .context("missing fft_size parameter in compress_fft_adpcm_f_u8")?
            .parse()
            .context("failed to parse fft_size parameter")?;

        let blk = CompressFftAdpcmFU8::new(fft_size);
        let id = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
