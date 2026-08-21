use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait LogPowerCmd<'i> {
    fn add_db(&self) -> Result<Option<&str>>;

    fn build_logpower_cf(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let add_db = self.add_db()?.unwrap_or("0.0");
        grc = grc
            .ensure_source(GrcItemType::C32)?
            .create_block_instance("logpower_cf")
            .with_parameter("add_db", add_db)
            .assert_output(GrcItemType::F32)
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> LogPowerCmd<'i> for Pair<'i, Rule> {
    fn add_db(&self) -> Result<Option<&'i str>> {
        let mut inner = self.clone().into_inner();
        if let Some(value) = inner.next() {
            let mut val_inner = value.clone().into_inner();
            if let Some(sub) = val_inner.next() {
                Ok(Some(sub.as_str()))
            } else {
                Ok(Some(value.as_str()))
            }
        } else {
            Ok(None)
        }
    }
}
