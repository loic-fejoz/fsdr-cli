use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use fsdr_blocks::math::FrequencyShifter;
use fsdr_blocks::futuresdr::anyhow::Result;
use fsdr_blocks::futuresdr::num_complex::Complex32;
use fsdr_blocks::futuresdr::runtime::Flowgraph;

pub struct FreqShiftCcConverter {}

impl BlockConverter for FreqShiftCcConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let sample_rate = Grc2FutureSdr::parameter_as_f64(blk, "sample_rate", "48000")? as f32;
        let freq = Grc2FutureSdr::parameter_as_f64(blk, "freq", "1.0")? as f32;
        let blk = FrequencyShifter::<Complex32>::new(freq, sample_rate);
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
