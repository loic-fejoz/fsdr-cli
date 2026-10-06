use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait DifferentialCodingCmd<'i> {
    fn build_differential_encoder(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::U8)?
            .create_block_instance("differential_encoder_u8_u8")
            .assert_output(GrcItemType::U8)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_differential_decoder(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::U8)?
            .create_block_instance("differential_decoder_u8_u8")
            .assert_output(GrcItemType::U8)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_invert_u8(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::U8)?
            .create_block_instance("invert_u8_u8")
            .assert_output(GrcItemType::U8)
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> DifferentialCodingCmd<'i> for Pair<'i, Rule> {}
