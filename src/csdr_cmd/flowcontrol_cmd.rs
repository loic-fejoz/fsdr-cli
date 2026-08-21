use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait FlowcontrolCmd<'i> {
    fn data_rate(&self) -> Result<&str>;

    fn build_flowcontrol(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let rate = self.data_rate()?;
        grc = grc
            .ensure_source(GrcItemType::F32)?
            .create_block_instance("blocks_throttle")
            .with_parameter("samples_per_second", rate)
            .with_parameter("type", GrcItemType::F32.as_grc())
            .assert_output(GrcItemType::F32)
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> FlowcontrolCmd<'i> for Pair<'i, Rule> {
    fn data_rate(&self) -> Result<&'i str> {
        let mut inner = self.clone().into_inner();
        if let Some(rate) = inner.next() {
            Ok(rate.as_str())
        } else {
            bail!("missing mandatory <data_rate> parameter for flowcontrol")
        }
    }
}
