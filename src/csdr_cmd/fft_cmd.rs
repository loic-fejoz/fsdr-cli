use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait FftCmd<'i> {
    fn fft_size(&self) -> Result<&str>;
    fn every_n_samples(&self) -> Result<&str>;
    fn window(&self) -> Result<Option<&str>>;

    fn build_fft_cc(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let fft_size = self.fft_size()?;
        let every_n_samples = self.every_n_samples()?;
        let window = self.window()?.unwrap_or("HAMMING");
        grc = grc
            .ensure_source(GrcItemType::C32)?
            .create_block_instance("fft_block")
            .with_parameter("type", "complex")
            .with_parameter("fft_size", fft_size)
            .with_parameter("every_n_samples", every_n_samples)
            .with_parameter("window", window)
            .assert_output(GrcItemType::C32)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_fft_fc(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let fft_size = self.fft_size()?;
        let every_n_samples = self.every_n_samples()?;
        let window = self.window()?.unwrap_or("HAMMING");
        grc = grc
            .ensure_source(GrcItemType::F32)?
            .create_block_instance("fft_block")
            .with_parameter("type", "float")
            .with_parameter("fft_size", fft_size)
            .with_parameter("every_n_samples", every_n_samples)
            .with_parameter("window", window)
            .assert_output(GrcItemType::C32)
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> FftCmd<'i> for Pair<'i, Rule> {
    fn fft_size(&self) -> Result<&'i str> {
        let mut inner = self.clone().into_inner();
        if let Some(value) = inner.next() {
            Ok(value.as_str())
        } else {
            bail!("missing mandatory <fft_size> parameter for fft")
        }
    }

    fn every_n_samples(&self) -> Result<&'i str> {
        let mut inner = self.clone().into_inner();
        inner.next();
        if let Some(value) = inner.next() {
            Ok(value.as_str())
        } else {
            bail!("missing mandatory <every_n_samples> parameter for fft")
        }
    }

    fn window(&self) -> Result<Option<&'i str>> {
        let mut inner = self.clone().into_inner();
        inner.next();
        inner.next();
        if let Some(value) = inner.next() {
            let s = value.as_str();
            if s.starts_with("--") {
                Ok(None)
            } else {
                Ok(Some(s))
            }
        } else {
            Ok(None)
        }
    }
}
