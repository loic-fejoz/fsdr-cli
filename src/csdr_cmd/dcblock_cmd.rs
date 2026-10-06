use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait DcBlockCmd<'i> {
    fn r_param(&self) -> Result<Option<&str>>;

    fn build_dcblock_ff(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let r = self.r_param()?.unwrap_or("0.999");
        grc = grc
            .ensure_source(GrcItemType::F32)?
            .create_block_instance("dcblock_ff")
            .with_parameter("r", r)
            .assert_output(GrcItemType::F32)
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> DcBlockCmd<'i> for Pair<'i, Rule> {
    fn r_param(&self) -> Result<Option<&'i str>> {
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
