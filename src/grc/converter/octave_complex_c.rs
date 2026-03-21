use crate::blocks::OctaveComplex;
use crate::grc::backend::FsdrBackend;

use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{parameter_as_f64, BlockInstance};
use anyhow::{anyhow, Result};

pub struct OctaveComplexConverter {}

impl<B: FsdrBackend> BlockConverter<B> for OctaveComplexConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let samples_to_plot = parameter_as_f64(blk, "samples_to_plot", "512")? as usize;
        let out_of_n_samples = parameter_as_f64(blk, "out_of_n_samples", "2048")? as usize;
        if out_of_n_samples < samples_to_plot {
            return Err(anyhow!("out_of_n_samples should be < samples_to_plot"));
        }
        let blk: OctaveComplex = OctaveComplex::new(samples_to_plot, out_of_n_samples);
        let blk_ref = backend.add_block_runtime(blk)?;
        Ok(Box::new(DefaultPortAdapter::new(blk_ref)))
    }
}
