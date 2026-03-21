use crate::cmd_grammar::Rule;
use pest::iterators::Pair;

pub trait GrcCmd<'i> {
    fn filename(&self) -> &'i str;
}

impl<'i> GrcCmd<'i> for Pair<'i, Rule> {
    fn filename(&self) -> &'i str {
        self.clone()
            .into_inner()
            .find(|p| matches!(p.as_rule(), Rule::filepath))
            .expect("missing filepath to GNU Radio flowgraph")
            .as_str()
    }
}
