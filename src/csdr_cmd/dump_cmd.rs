use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use futuresdr::anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait DumpCmd<'i> {
    fn block_name(&self) -> Result<&str>;
    fn input_type(&self) -> Result<GrcItemType>;

    fn build_dump(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let input_type = self.input_type()?;
        grc = grc
            .ensure_source(input_type)
            .create_block_instance(self.block_name()?)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> DumpCmd<'i> for Pair<'i, Rule> {
    fn block_name(&self) -> Result<&str> {
        Ok(self.as_str())
    }

    fn input_type(&self) -> Result<GrcItemType> {
        let input = self.as_str();
        match input {
            "dump_u8" => Ok(GrcItemType::U8),
            "dump_f" => Ok(GrcItemType::F32),
            _ => bail!("Unkown dump type: {input}"),
        }
    }
}
