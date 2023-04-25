use crate::cmd_grammar::Rule;
use pest::iterators::Pair;

pub trait GrcCmd<'i> {
    fn filename(&self) -> &'i str;
}

impl<'i> GrcCmd<'i> for Pair<'i, Rule> {
    fn filename(&self) -> &'i str {
        let cmd = self.clone();
        let mut args = cmd.into_inner();
        let first = args.next().expect("missig filepath to GNU Radio flowgraph");
        match first.as_rule() {
            Rule::filepath => first.as_str(),
            _ => todo!(),
        }
    }
}
