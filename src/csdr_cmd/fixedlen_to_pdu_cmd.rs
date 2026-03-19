use crate::cmd_grammar::Rule;
use crate::csdr_cmd::eval_cmd::EvalCmd;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::{Context, Result};
use pest::iterators::Pair;

pub trait FixedlenToPduCmd<'i> {
    fn packet_len(&self) -> Result<String>;
    fn syncword_tag(&self) -> Result<Option<String>>;
    fn build_fixedlen_to_pdu(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
}

impl<'i> FixedlenToPduCmd<'i> for Pair<'i, Rule> {
    fn packet_len(&self) -> Result<String> {
        self.clone()
            .into_inner()
            .next()
            .context("packet_len expression expected")?
            .eval()
            .map(|v| v.to_string())
    }

    fn syncword_tag(&self) -> Result<Option<String>> {
        let mut inner = self.clone().into_inner();
        inner.next(); // skip packet_len
        if let Some(tag) = inner.next() {
            Ok(Some(tag.as_str().to_string()))
        } else {
            Ok(None)
        }
    }

    fn build_fixedlen_to_pdu(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let packet_len = self.packet_len()?;
        let syncword_tag = self.syncword_tag()?.unwrap_or_else(|| "".to_string());

        grc = grc
            .ensure_source(GrcItemType::U8)?
            .create_block_instance("satellites_fixedlen_to_pdu")
            .with_parameter("type", "byte")
            .with_parameter("packet_len", &packet_len)
            .with_parameter("syncword_tag", &syncword_tag)
            .with_parameter("pack", "False")
            .with_parameter("packet_len_tag_key", "\"\"")
            .push_and_link()?;
        Ok(grc)
    }
}
