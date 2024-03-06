use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use fsdr_blocks::futuresdr::anyhow::Result;
use pest::iterators::Pair;

pub trait LimitCmd<'i> {
    fn max_amplitude(&self) -> Result<&str>;

    fn build_limit(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let max_amplitude = self.max_amplitude()?;
        grc = grc
            .ensure_source(GrcItemType::F32)
            .create_block_instance("analog_rail_ff")
            .with_parameter("lo", format!("-1.0*({max_amplitude})"))
            .with_parameter("hi", max_amplitude.to_string())
            .assert_output(GrcItemType::F32)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> LimitCmd<'i> for Pair<'i, Rule> {
    fn max_amplitude(&self) -> Result<&'i str> {
        if let Some(max_amplitude) = self.clone().into_inner().next() {
            Ok(max_amplitude.as_str())
        } else {
            Ok("1.0")
        }
    }
}
