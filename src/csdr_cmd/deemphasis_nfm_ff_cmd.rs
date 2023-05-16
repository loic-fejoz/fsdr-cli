use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use futuresdr::anyhow::Result;
use pest::iterators::Pair;

pub trait DeemphasisNfnCmd<'i> {
    fn rate(&self) -> Result<&str>;

    fn build_deemphasis_nfm(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let rate = self.rate()?;
        grc = grc
            .ensure_source(GrcItemType::F32)
            .create_block_instance("deemphasis_nfm_ff")
            .with_parameter("sample_rate", rate)
            .assert_output(GrcItemType::F32)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> DeemphasisNfnCmd<'i> for Pair<'i, Rule> {
    fn rate(&self) -> Result<&'i str> {
        if let Some(rate) = self.clone().into_inner().next() {
            Ok(rate.as_str())
        } else {
            Ok("48000")
        }
    }
}
