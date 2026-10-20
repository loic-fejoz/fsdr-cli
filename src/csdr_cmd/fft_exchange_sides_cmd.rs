use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::Result;
use pest::iterators::Pair;

pub trait FftExchangeSidesCmd<'i> {
    fn explicit_fft_size(&self) -> Option<&'i str>;

    fn build_fft_exchange_sides_ff(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let fft_size = if let Some(sz) = self.explicit_fft_size() {
            sz.to_string()
        } else if let Some(prev) = grc
            .blocks()
            .iter()
            .rev()
            .find(|b| b.parameters.contains_key("fft_size"))
        {
            prev.parameters.get("fft_size").unwrap().clone()
        } else {
            "4096".to_string()
        };

        grc = grc
            .ensure_source(GrcItemType::F32)?
            .create_block_instance("fft_exchange_sides_ff")
            .with_parameter("fft_size", fft_size)
            .assert_output(GrcItemType::F32)
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> FftExchangeSidesCmd<'i> for Pair<'i, Rule> {
    fn explicit_fft_size(&self) -> Option<&'i str> {
        let mut inner = self.clone().into_inner();
        inner.next().map(|v| v.as_str())
    }
}
