use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait FixedAmplitudeCmd<'i> {
    fn amplitude(&self) -> Result<Option<&str>>;

    fn build_fixed_amplitude(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let amp = self.amplitude()?.unwrap_or("1.0");
        grc = grc
            .ensure_source(GrcItemType::C32)?
            .create_block_instance("fixed_amplitude_cc")
            .with_parameter("amplitude", amp)
            .assert_output(GrcItemType::C32)
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> FixedAmplitudeCmd<'i> for Pair<'i, Rule> {
    fn amplitude(&self) -> Result<Option<&'i str>> {
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
