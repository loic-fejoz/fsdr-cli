use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use crate::blocks::{FftCc, FftFc};
use anyhow::{bail, Result};
use futuresdr::runtime::Flowgraph;

pub struct FftBlockConverter {}

impl BlockConverter for FftBlockConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let item_type = blk.parameter_or("type", "complex");
        let fft_size = Grc2FutureSdr::parameter_as_f64(blk, "fft_size", "512")? as usize;
        let default_every = fft_size.to_string();
        let every_n_samples =
            Grc2FutureSdr::parameter_as_f64(blk, "every_n_samples", &default_every[..])? as usize;
        let window = blk.parameter_or("window", "HAMMING");

        let adapter: Box<dyn ConnectorAdapter> = match item_type {
            "complex" | "c" | "c32" => {
                let block = FftCc::new(fft_size, every_n_samples, window);
                let id = fg.add(block)?.id();
                Box::new(DefaultPortAdapter::new(id))
            }
            "float" | "f" | "f32" => {
                let block = FftFc::new(fft_size, every_n_samples, window);
                let id = fg.add(block)?.id();
                Box::new(DefaultPortAdapter::new(id))
            }
            _ => bail!("Unsupported type for fft: {item_type}"),
        };

        Ok(adapter)
    }
}
