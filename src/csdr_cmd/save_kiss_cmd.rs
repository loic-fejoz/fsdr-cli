use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder};
use anyhow::{Context, Result};
use pest::iterators::Pair;

pub trait SaveKissCmd<'i> {
    fn filename(&self) -> Result<&str>;
    fn build_save_kiss(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
}

impl<'i> SaveKissCmd<'i> for Pair<'i, Rule> {
    fn filename(&self) -> Result<&'i str> {
        for arg in self.clone().into_inner() {
            if arg.as_rule() == Rule::load_param {
                return Ok(arg
                    .into_inner()
                    .next()
                    .context("filename expected")?
                    .as_str());
            }
        }
        Ok("-")
    }

    fn build_save_kiss(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let filename = self.filename()?;
        grc = grc
            .create_block_instance("satellites_kiss_file_sink")
            .with_parameter("file", filename)
            .push_and_link()?;
        Ok(grc)
    }
}
