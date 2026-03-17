use crate::cmd_grammar::Rule;
use pest::iterators::Pair;

pub trait IQEngineCmd<'i> {
    fn iqengine_configuration(&self) -> Option<&'i str>;
}

impl<'i> IQEngineCmd<'i> for Pair<'i, Rule> {
    fn iqengine_configuration(&self) -> Option<&'i str> {
        let cmd = self.clone();
        let mut args = cmd.into_inner();
        if let Some(first) = args.next() {
            match first.as_rule() {
                Rule::filepath => Some(first.as_str()),
                _ => None,
            }
        } else {
            None
        }
    }
}
