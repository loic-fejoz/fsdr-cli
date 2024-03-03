use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use futuresdr::anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait PatternSearchCmd<'i> {
    fn values_after(&self) -> Result<&'i str>;

    fn pattern_values(&self) -> Result<Vec<&'i str>>;

    fn build_pattern_search(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::U8)
            .create_block_instance("pattern_search")
            .with_parameter("values_after", self.values_after()?)
            .with_parameter("pattern_values", self.pattern_values()?.join(","))
            .assert_output(GrcItemType::U8)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> PatternSearchCmd<'i> for Pair<'i, Rule> {
    fn values_after(&self) -> Result<&'i str> {
        let mut inner = self.clone().into_inner();
        if let Some(value) = inner.next() {
            Ok(value.as_str())
        } else {
            bail!("missing mandatory <values_after> parameters for pattern_search_u8_u8")
        }
    }

    fn pattern_values(&self) -> Result<Vec<&'i str>> {
        let mut inner = self.clone().into_inner();
        inner.next();
        let preamble_data: Vec<&'i str> = inner.map(|x| x.as_str()).collect();
        if preamble_data.is_empty() {
            bail!("missing mandatory <pattern_values> parameters for pattern_search_u8_u8")
        } else {
            Ok(preamble_data)
        }
    }
}
