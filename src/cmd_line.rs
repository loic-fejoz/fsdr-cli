use crate::cmd_grammar::Rule;
use pest::iterators::Pair;

pub trait HighLevelCmdLine<'i> {
    fn is_grc_cmd(&self) -> bool;
    fn is_csdr_cmd(&self) -> bool;
    fn is_help_cmd(&self) -> bool;
    fn as_grc_cmd(&self) -> Option<&Pair<'i, Rule>>;
    fn as_csdr_cmd(&self) -> Option<&Pair<'i, Rule>>;
}

impl<'i> HighLevelCmdLine<'i> for Pair<'i, Rule> {
    fn is_grc_cmd(&self) -> bool {
        match self.as_rule() {
            Rule::grc_cmd => true,
            _ => false,
        }
    }

    fn as_grc_cmd(&self) -> Option<&Self> {
        match self.as_rule() {
            Rule::grc_cmd => Some(self),
            _ => None,
        }
    }

    fn is_csdr_cmd(&self) -> bool {
        match self.as_rule() {
            Rule::csdr_cmd => true,
            _ => false,
        }
    }

    fn as_csdr_cmd(&self) -> Option<&Self> {
        match self.as_rule() {
            Rule::csdr_cmd => Some(self),
            _ => None,
        }
    }

    fn is_help_cmd(&self) -> bool {
        match self.as_rule() {
            Rule::help_cmd => true,
            _ => false,
        }
    }
}
