use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use crate::blocks::{QuadratureDemodAlgo, QuadratureDemodCf};
use anyhow::{bail, Result};
use futuresdr::runtime::Flowgraph;

pub struct AnalogQuadratureDemoConverter {}

impl BlockConverter for AnalogQuadratureDemoConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let gain = Grc2FutureSdr::parameter_as_f64(blk, "gain", "1.0")? as f32;
        let algo_str = blk.parameter_or("algorithm", "quadri");
        let algo = match algo_str {
            "quadri" => QuadratureDemodAlgo::Quadri,
            "atan" => QuadratureDemodAlgo::Atan,
            _ => bail!("analog_quadrature_demod: Unknown algorithm: {algo_str}"),
        };

        let block = QuadratureDemodCf::with_algo(gain, algo);
        let id = fg.add(block)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
