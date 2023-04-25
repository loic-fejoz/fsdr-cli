use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder};
use crate::grc::Grc;
use futuresdr::anyhow::{bail, Result};
use pest::iterators::Pair;

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
            Rule::limit_cmd => {
                self.build_limit(grc)
            }
            Rule::csdr_save_opt => {
                Ok(grc.clone())
            }
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
