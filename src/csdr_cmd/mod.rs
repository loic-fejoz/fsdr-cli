use crate::cmd_grammar::{CommandsParser, Rule};
use crate::grc::builder::{GraphLevel, GrcBuilder};
use crate::grc::{self, Grc};
use futuresdr::anyhow::{bail, Result};
use pest::iterators::Pair;
use pest::Parser;

use self::eval_cmd::EvalCmd;
use self::limit_cmd::LimitCmd;

pub mod eval_cmd;
mod limit_cmd;

pub trait AnyCmd<'i> {
    fn parse(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
}

impl<'i> AnyCmd<'i> for Pair<'i, Rule> {
    fn parse(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        match self.as_rule() {
            Rule::eval_cmd => {
                self.execute_eval()?;
                Ok(grc.clone())
            }
            Rule::limit_cmd => self.build_limit(grc),
            Rule::csdr_save_opt => Ok(grc.clone()),
            _ => {
                let rule = self.as_rule();
                todo!("unknown any cmd: {rule:?}");
            }
        }
    }
}

pub trait CsdrCmd<'i> {
    fn output(&self) -> Option<&'i str>;
    fn parse(&self) -> Result<Option<Grc>>;
}

impl<'i> CsdrCmd<'i> for Pair<'i, Rule> {
    fn output(&self) -> Option<&'i str> {
        let cmd = self.clone();
        let mut args = cmd.into_inner();
        if let Some(first_inner) = args.next() {
            match first_inner.as_rule() {
                Rule::csdr_save_opt => {
                    let filename = first_inner
                        .into_inner()
                        .next()
                        .expect("output filepath expected");
                    let filename = filename.as_str();
                    Some(filename)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn parse(&self) -> Result<Option<Grc>> {
        let mut grc_builder = GrcBuilder::new();
        for sub_cmd in self.clone().into_inner() {
            grc_builder = AnyCmd::parse(&sub_cmd, grc_builder)?;
        }
        grc_builder.ensure_sink();
        let grc = grc_builder.build()?;
        Ok(Some(grc))
    }
}

#[derive(Default)]
pub struct CsdrParser {}

impl CsdrParser {
    pub fn parse_command<'i>(cmd: impl Into<&'i str>) -> Result<Option<Grc>> {
        // let cmd = CommandsParser::parse_main(cmd.into())?;
        // CsdrCmd::parse(&cmd)

        let input = CommandsParser::parse(Rule::any_csdr_cmd, cmd.into())
            .expect("msg")
            .next()
            .expect("msg");
        let grc_builder = GrcBuilder::new();
        let mut grc_builder = AnyCmd::parse(&input, grc_builder)?;
        grc_builder.ensure_sink();
        let grc = grc_builder.build()?;
        Ok(Some(grc))
    }

    pub fn parse_multiple_commands<'i>(cmd: impl Into<&'i str>) -> Result<Option<Grc>> {
        let cmd = CommandsParser::parse_main(cmd.into())?;
        CsdrCmd::parse(&cmd)
    }
}
