use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait AddDcOffsetCmd<'i> {
    fn offset_re(&self) -> Result<Option<&str>>;
    fn offset_im(&self) -> Result<Option<&str>>;

    fn build_add_dcoffset_cc(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let mut builder = grc
            .ensure_source(GrcItemType::C32)?
            .create_block_instance("add_dcoffset_cc");
        if let Some(re) = self.offset_re()? {
            builder.with_parameter("offset_re", re);
            if let Some(im) = self.offset_im()? {
                builder.with_parameter("offset_im", im);
            }
        }
        grc = builder.assert_output(GrcItemType::C32).push_and_link()?;
        Ok(grc)
    }
}

impl<'i> AddDcOffsetCmd<'i> for Pair<'i, Rule> {
    fn offset_re(&self) -> Result<Option<&'i str>> {
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

    fn offset_im(&self) -> Result<Option<&'i str>> {
        let mut inner = self.clone().into_inner();
        inner.next();
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
