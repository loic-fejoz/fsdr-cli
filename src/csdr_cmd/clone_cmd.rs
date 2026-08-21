use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait CloneCmd<'i> {
    fn build_clone(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        // clone and through are transparent passthrough in the flowgraph
        Ok(grc)
    }

    fn build_none(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::F32)?
            .create_block_instance("blocks_null_sink")
            .with_parameter("type", GrcItemType::F32.as_grc())
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> CloneCmd<'i> for Pair<'i, Rule> {}
