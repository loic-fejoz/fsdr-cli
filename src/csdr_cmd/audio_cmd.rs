use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use futuresdr::anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait AudioCmd<'i> {
    fn audio_rate(&self) -> Result<&str>;
    fn num_inputs(&self) -> Result<Option<&str>>;

    fn build_audio_sink(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let audio_rate = self.audio_rate()?;
        let num_inputs = self.num_inputs()?.or(Some("1")).expect("");
        grc = grc
            .ensure_source(GrcItemType::F32)
            .create_block_instance("audio_sink")
            .with_parameter("samp_rate", audio_rate)
            .with_parameter("num_inputs", num_inputs)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> AudioCmd<'i> for Pair<'i, Rule> {
    fn audio_rate(&self) -> Result<&'i str> {
        if let Some(value) = self.clone().into_inner().next() {
            Ok(value.as_str())
        } else {
            bail!("missing mandatory <audio_rate> parameters for audio")
        }
    }

    fn num_inputs(&self) -> Result<Option<&str>> {
        let mut inner = self.clone().into_inner();
        inner.next();
        if let Some(value) = inner.next() {
            Ok(Some(value.as_str()))
        } else {
            Ok(None)
        }
    }
}
