use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use fsdr_blocks::futuresdr::anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait WeaverCmd<'i> {
    fn block_name(&self) -> Result<&str>;
    fn audio_rate(&self) -> Result<&str>;

    fn build_weaver(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let blk_name = self.block_name()?;
        let mut grc = grc;
        let audio_rate = self.audio_rate()?;
        grc = grc
            .ensure_source(GrcItemType::C32)
            .create_block_instance(blk_name)
            .with_parameter("audio_rate", audio_rate)
            .assert_output(GrcItemType::F32)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> WeaverCmd<'i> for Pair<'i, Rule> {
    fn block_name(&self) -> Result<&str> {
        match self.as_rule() {
            Rule::weaver_usb_cmd => Ok("weaver_usb_cf"),
            Rule::weaver_lsb_cmd => Ok("weaver_lsb_cf"),
            _ => bail!("Unknown weaver command"),
        }
    }

    fn audio_rate(&self) -> Result<&'i str> {
        if let Some(rate) = self.clone().into_inner().next() {
            Ok(rate.as_str())
        } else {
            bail!("missing mandatory <audio_rate> parameters for weaver_XXX_cc")
        }
    }
}
