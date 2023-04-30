use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use futuresdr::anyhow::Result;
use pest::iterators::Pair;

pub trait FmDemodQuadriCmd<'i> {
    fn build_fm_demod_quadri(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc.clone();
        grc = grc
            .ensure_source(GrcItemType::C32)
            .create_block_instance("analog_quadrature_demod_cf")
            .with_parameter("gain", "1.0")
            .assert_output(GrcItemType::F32)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> FmDemodQuadriCmd<'i> for Pair<'i, Rule> {
}
