use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait FftExchangeSidesCmd<'i> {
    fn fft_size(&self) -> Result<&str>;

    fn build_fft_exchange_sides_ff(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let fft_size = self.fft_size()?;
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
    fn fft_size(&self) -> Result<&'i str> {
        let mut inner = self.clone().into_inner();
        if let Some(value) = inner.next() {
            Ok(value.as_str())
        } else {
            bail!("missing mandatory <fft_size> parameter for fft_exchange_sides_ff")
        }
    }
}
