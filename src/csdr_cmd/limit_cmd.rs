use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use futuresdr::anyhow::{bail, Result};
use pest::iterators::Pair;
use std::{
    env,
    f32::consts::{E, PI},
    num::ParseFloatError,
};

pub trait LimitCmd<'i> {
    fn max_amplitude(&self) -> Result<f32>;
    fn build_limit(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
}

impl<'i> LimitCmd<'i> for Pair<'i, Rule> {
    fn max_amplitude(&self) -> Result<f32> {
        Ok(1.0f32)
    }

    fn build_limit(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc.clone();
        let max_amplitude = self.max_amplitude()?;
        grc = grc
            .ensure_source(GrcItemType::F32)
            .create_block_instance("analog_rail_ff")
                .with_parameter("lo", format!("-{max_amplitude}"))
                .with_parameter("hi", format!("{max_amplitude}"))
                .assert_output(GrcItemType::F32)
                .push_and_link();
        Ok(grc)
    }
}
