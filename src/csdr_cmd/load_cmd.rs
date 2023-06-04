use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use futuresdr::anyhow::{bail, Context, Result};
use pest::iterators::Pair;

pub trait LoadCmd<'i> {
    fn block_name(&self) -> Result<&str>;
    fn input_type(&self) -> Result<GrcItemType>;
    fn filename(&self) -> Result<&str>;

    fn build_load(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let input_type = self.input_type()?;
        let filename = self.filename()?;
        grc = grc
            .create_block_instance("blocks_file_source")
            .with_parameter("type", input_type.as_grc())
            .with_parameter("file", filename)
            .with_parameter("repeat", "false")
            .assert_output(input_type)
            .push();
        Ok(grc)
    }
}

impl<'i> LoadCmd<'i> for Pair<'i, Rule> {
    fn block_name(&self) -> Result<&str> {
        let cmd = self.clone();
        let mut args = cmd.into_inner();
        let first = args.next().expect("");
        Ok(first.as_str())
    }

    fn input_type(&self) -> Result<GrcItemType> {
        let input = self.block_name()?;
        match input {
            "u8" => Ok(GrcItemType::U8),
            "f" => Ok(GrcItemType::F32),
            "c" => Ok(GrcItemType::C32),
            _ => bail!("Unkown load type: {input}"),
        }
    }

    fn filename(&self) -> Result<&'i str> {
        for arg in self.clone().into_inner() {
            if arg.as_rule() == Rule::load_param {
                return Ok(arg
                    .into_inner()
                    .next()
                    .context("filename expected")?
                    .as_str());
            }
        }
        Ok("-")
    }
}
