use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use fsdr_blocks::futuresdr::anyhow::Result;
use pest::iterators::Pair;

pub trait ThrottleCmd<'i> {
    fn rate(&self) -> Result<&str>;

    fn build_throttle(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let rate = self.rate()?;
        grc = grc
            .ensure_source(GrcItemType::F32)
            .create_block_instance("blocks_throttle")
            .with_parameter("samples_per_second", rate)
            .with_parameter("type", GrcItemType::F32.as_grc())
            .assert_output(GrcItemType::F32)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> ThrottleCmd<'i> for Pair<'i, Rule> {
    fn rate(&self) -> Result<&'i str> {
        if let Some(rate) = self.clone().into_inner().next() {
            Ok(rate.as_str())
        } else {
            Ok("48000")
        }
    }
}
