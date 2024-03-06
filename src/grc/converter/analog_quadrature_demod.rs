use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use fsdr_blocks::futuresdr::anyhow::Result;
use fsdr_blocks::futuresdr::blocks::Apply;
use fsdr_blocks::futuresdr::num_complex::Complex32;
use fsdr_blocks::futuresdr::runtime::Flowgraph;

pub struct AnalogQuadratureDemoConverter {}

impl BlockConverter for AnalogQuadratureDemoConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let gain = Grc2FutureSdr::parameter_as_f64(blk, "gain", "1.0")? as f32;
        let mut last = Complex32::new(0.0, 0.0); // store sample x[n-1]
        let blk = Apply::new(move |v: &Complex32| -> f32 {
            let arg = (v * last.conj()).arg(); // Obtain phase of x[n] * conj(x[n-1])
            last = *v;
            arg * gain
        });
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
