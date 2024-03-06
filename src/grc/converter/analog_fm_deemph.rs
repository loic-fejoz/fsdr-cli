use fsdr_blocks::futuresdr::blocks::Apply;

use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use fsdr_blocks::futuresdr::anyhow::Result;
use fsdr_blocks::futuresdr::runtime::Flowgraph;

pub struct AnalogFmDeemphConverter {}

impl BlockConverter for AnalogFmDeemphConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let sample_rate = Grc2FutureSdr::parameter_as_f64(blk, "samp_rate", "48000")? as f32;
        let tau = Grc2FutureSdr::parameter_as_f64(blk, "tau", "50e-6")? as f32;
        let dt = 1.0 / sample_rate;
        let alpha = dt / (tau + dt);
        let mut last = 0.0; // store sample x[n-1]
        let blk = Apply::new(move |v: &f32| -> f32 {
            let r = alpha * v + (1.0 - alpha) * last; //this is the simplest IIR LPF
            last = r;
            r
        });
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
