use pest::iterators::Pair;
use pest::Parser;

use futuresdr::anyhow::Result;

#[derive(Parser)]
#[grammar = "src/cmd_line.pest"]
pub struct CommandsParser;

impl CommandsParser {
    pub fn parse_main(input: &str) -> Result<Pair<Rule>> {
        let input = CommandsParser::parse(Rule::main, input)
            .expect("Error while parsing:")
            .next()
            .expect("msg");
        //println!("input: {input:?}");
        Ok(input)
    }

    pub fn parse_expr<'i>(expr: impl Into<&'i str>) -> Result<Pair<'i, Rule>> {
        let expr = CommandsParser::parse(Rule::expr, expr.into())
            .expect("msg")
            .next()
            .expect("msg");
        //println!("input: {input:?}");
        Ok(expr)
    }
}
