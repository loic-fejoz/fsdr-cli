use crate::blocks::OctaveComplex;

use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use futuresdr::anyhow::{anyhow, Result};
use futuresdr::runtime::Flowgraph;

pub struct OctaveComplexConverter {}

impl BlockConverter for OctaveComplexConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let samples_to_plot =
            Grc2FutureSdr::parameter_as_f64(blk, "samples_to_plot", "512")? as usize;
        let out_of_n_samples =
            Grc2FutureSdr::parameter_as_f64(blk, "out_of_n_samples", "2048")? as usize;
        if out_of_n_samples < samples_to_plot {
            return Err(anyhow!("out_of_n_samples should be < samples_to_plot"));
        }
        let blk = OctaveComplex::build(samples_to_plot, out_of_n_samples);
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
