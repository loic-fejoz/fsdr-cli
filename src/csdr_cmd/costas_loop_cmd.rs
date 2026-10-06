use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait CostasLoopCmd<'i> {
    fn loop_bw(&self) -> Result<Option<&str>>;
    fn damping(&self) -> Result<Option<&str>>;

    fn build_costas_loop(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let loop_bw = self.loop_bw()?.unwrap_or("0.05");
        let damping = self.damping()?.unwrap_or("0.707");

        grc = grc
            .ensure_source(GrcItemType::C32)?
            .create_block_instance("bpsk_costas_loop_cc")
            .with_parameter("loop_bw", loop_bw)
            .with_parameter("damping", damping)
            .assert_output(GrcItemType::C32)
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> CostasLoopCmd<'i> for Pair<'i, Rule> {
    fn loop_bw(&self) -> Result<Option<&'i str>> {
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

    fn damping(&self) -> Result<Option<&'i str>> {
        let mut inner = self.clone().into_inner();
        let _ = inner.next(); // skip loop_bw
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
