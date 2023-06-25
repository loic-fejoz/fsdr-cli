use crate::cmd_grammar::Rule;
use crate::grc::builder::GrcItemType;
use futuresdr::anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait CsdrCmd<'i> {
    fn arg<const N: usize>(&self, msg: impl Into<&'static str>) -> Result<&'i str>;
    fn type_arg<const N: usize>(&self, msg: impl Into<&'static str>) -> Result<GrcItemType>;
    fn optional_arg<const N: usize>(&self) -> Result<Option<&'i str>>;
}

impl<'i> CsdrCmd<'i> for Pair<'i, Rule> {
    fn arg<const N: usize>(&self, msg: impl Into<&'static str>) -> Result<&'i str> {
        let mut args = self.clone().into_inner();
        for _ in 1..N {
            args.next();
        }
        if let Some(value) = args.next() {
            Ok(value.as_str())
        } else {
            bail!(msg.into())
        }
    }

    fn optional_arg<const N: usize>(&self) -> Result<Option<&'i str>> {
        let mut args = self.clone().into_inner();
        for _ in 1..N {
            args.next();
        }
        if let Some(value) = args.next() {
            Ok(Some(value.as_str()))
        } else {
            Ok(None)
        }
    }

    fn type_arg<const N: usize>(&self, msg: impl Into<&'static str>) -> Result<GrcItemType> {
        let arg = self.arg::<N>(msg)?;
        match arg {
            "u8" => Ok(GrcItemType::U8),
            "f" => Ok(GrcItemType::F32),
            "c" => Ok(GrcItemType::C32),
            _ => bail!("Unkown item type: {arg}"),
        }
    }
}
