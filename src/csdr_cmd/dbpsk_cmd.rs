use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait DBPskCmd<'i> {
    fn build_dbpsk_decoder(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::C32)?
            .create_block_instance("dbpsk_decoder_c_u8")
            .assert_output(GrcItemType::U8)
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> DBPskCmd<'i> for Pair<'i, Rule> {}
