use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait YesCmd<'i> {
    fn value(&self) -> Result<Option<&str>>;

    fn build_yes_f(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let val = self.value()?.unwrap_or("1.0");
        grc = grc
            .create_block_instance("yes_f")
            .with_parameter("value", val)
            .assert_output(GrcItemType::F32)
            .push()?;
        Ok(grc)
    }
}

impl<'i> YesCmd<'i> for Pair<'i, Rule> {
    fn value(&self) -> Result<Option<&'i str>> {
        let mut inner = self.clone().into_inner();
        if let Some(val) = inner.next() {
            let mut val_inner = val.clone().into_inner();
            if let Some(sub) = val_inner.next() {
                Ok(Some(sub.as_str()))
            } else {
                Ok(Some(val.as_str()))
            }
        } else {
            Ok(None)
        }
    }
}
