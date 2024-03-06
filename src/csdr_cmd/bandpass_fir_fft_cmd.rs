use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use fsdr_blocks::futuresdr::anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait BandpassFirFftcmd<'i> {
    fn low_cut(&self) -> Result<&str>;
    fn high_cut(&self) -> Result<&str>;
    fn bandwidth(&self) -> Result<Option<&str>>;
    fn window(&self) -> Result<Option<&str>>;

    fn build_bandpass_fir_fft_cc(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let low_cut = self.low_cut()?;
        let high_cut = self.high_cut()?;
        let transition_bw = self.bandwidth()?.unwrap_or("0.05");
        let window = self.window()?.unwrap_or("HAMMING");
        grc = grc
            .ensure_source(GrcItemType::C32)
            .create_block_instance("band_pass_filter")
            .with_parameter("beta", "6.76") // only for Kaiser window
            .with_parameter("decim", "1")
            .with_parameter("gain", "1")
            .with_parameter("high_cutoff_freq", high_cut)
            .with_parameter("interp", "1")
            .with_parameter("low_cutoff_freq", low_cut)
            .with_parameter("samp_rate", "1")
            .with_parameter("type", "fir_filter_ccc")
            .with_parameter("width", transition_bw)
            .with_parameter("win", format!("window.WIN_{window}"))
            .assert_output(GrcItemType::C32)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> BandpassFirFftcmd<'i> for Pair<'i, Rule> {
    fn low_cut(&self) -> Result<&'i str> {
        if let Some(value) = self.clone().into_inner().next() {
            Ok(value.as_str())
        } else {
            bail!("missing mandatory <low_cut> parameters for bandpass_fir_fft_cc")
        }
    }

    fn high_cut(&self) -> Result<&'i str> {
        let mut inner = self.clone().into_inner();
        inner.next();
        if let Some(value) = inner.next() {
            Ok(value.as_str())
        } else {
            bail!("missing mandatory <high_cut> parameters for bandpass_fir_fft_cc")
        }
    }

    fn bandwidth(&self) -> Result<Option<&'i str>> {
        let mut inner = self.clone().into_inner();
        inner.next();
        inner.next();
        if let Some(value) = inner.next() {
            Ok(Some(value.as_str()))
        } else {
            Ok(None)
        }
    }

    fn window(&self) -> Result<Option<&'i str>> {
        let mut inner = self.clone().into_inner();
        inner.next();
        inner.next();
        inner.next();
        if let Some(value) = inner.next() {
            Ok(Some(value.as_str()))
        } else {
            Ok(None)
        }
    }
}
