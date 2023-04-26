use pest::iterators::Pair;
use pest::Parser;

use futuresdr::anyhow::{bail, Result};
use std::{
    f32::consts::{E, PI},
    num::ParseFloatError,
};

#[derive(Parser)]
#[grammar = "src/cmd_line.pest"]
pub struct CommandsParser;

impl CommandsParser {
    pub fn parse_main<'i>(input: &'i str) -> Result<Pair<'i, Rule>> {
        let input = CommandsParser::parse(Rule::main, &input)
            .expect("msg")
            .next()
            .expect("msg");
        println!("input: {input:?}");
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
