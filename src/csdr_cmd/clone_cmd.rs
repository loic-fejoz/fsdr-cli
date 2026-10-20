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
        let item_type = grc.last_output_type().unwrap_or(GrcItemType::F32);
        grc = grc
            .ensure_source(item_type)?
            .create_block_instance("blocks_null_sink")
            .with_parameter("type", item_type.as_grc())
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> CloneCmd<'i> for Pair<'i, Rule> {}
