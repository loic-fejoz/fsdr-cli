use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait Mono2StereoCmd<'i> {
    fn build_mono2stereo_s16(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::S16)?
            .create_block_instance("mono2stereo_s16")
            .assert_output(GrcItemType::S16)
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> Mono2StereoCmd<'i> for Pair<'i, Rule> {}
