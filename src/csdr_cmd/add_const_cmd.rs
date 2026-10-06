use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait AddConstCmd<'i> {
    fn real_offset(&self) -> Result<Option<&str>>;
    fn imag_offset(&self) -> Result<Option<&str>>;

    fn build_add_const(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let real = self.real_offset()?.unwrap_or("0.0");
        let imag = self.imag_offset()?.unwrap_or("0.0");
        grc = grc
            .ensure_source(GrcItemType::C32)?
            .create_block_instance("add_const_cc")
            .with_parameter("real", real)
            .with_parameter("imag", imag)
            .assert_output(GrcItemType::C32)
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> AddConstCmd<'i> for Pair<'i, Rule> {
    fn real_offset(&self) -> Result<Option<&'i str>> {
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

    fn imag_offset(&self) -> Result<Option<&'i str>> {
        let mut inner = self.clone().into_inner();
        let _ = inner.next(); // skip real
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
