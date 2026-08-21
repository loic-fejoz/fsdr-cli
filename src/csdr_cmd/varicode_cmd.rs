use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait VaricodeCmd<'i> {
    fn build_varicode_decoder(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::U8)?
            .create_block_instance("psk31_varicode_decoder_u8_u8")
            .assert_output(GrcItemType::U8)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_varicode_encoder(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::U8)?
            .create_block_instance("psk31_varicode_encoder_u8_u8")
            .assert_output(GrcItemType::U8)
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> VaricodeCmd<'i> for Pair<'i, Rule> {}
