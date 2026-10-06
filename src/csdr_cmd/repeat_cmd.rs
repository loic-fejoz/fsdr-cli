use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait RepeatCmd<'i> {
    fn repeat_count(&self) -> Result<&str>;

    fn build_repeat_u8(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let repeat = self.repeat_count()?;
        grc = grc
            .ensure_source(GrcItemType::U8)?
            .create_block_instance("repeat_u8")
            .with_parameter("repeat", repeat)
            .assert_output(GrcItemType::U8)
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> RepeatCmd<'i> for Pair<'i, Rule> {
    fn repeat_count(&self) -> Result<&'i str> {
        let mut inner = self.clone().into_inner();
        if let Some(count) = inner.next() {
            Ok(count.as_str())
        } else {
            bail!("missing mandatory <repeat> parameter for repeat_u8")
        }
    }
}
