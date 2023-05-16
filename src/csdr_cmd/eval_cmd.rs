use crate::cmd_grammar::Rule;
use crate::grc::Grc;
use futuresdr::anyhow::Result;
use pest::iterators::Pair;
use std::f32::consts::{E, PI};

pub trait EvalCmd<'i> {
    fn eval(&self) -> Result<f32>;
    fn execute_eval(&self) -> Result<Option<Grc>>;
}

impl<'i> EvalCmd<'i> for Pair<'i, Rule> {
    fn execute_eval(&self) -> Result<Option<Grc>> {
        let expr = self.clone().into_inner().next().expect("msg");
        let result = expr.eval()?;
        println!("{result}");
        Ok(None)
    }

    fn eval(&self) -> Result<f32> {
        match self.as_rule() {
            Rule::ident => match self.as_str() {
                "pi" => Ok(PI),
                "e" => Ok(E),
                "nan" => Ok(f32::NAN),
                "inf" => Ok(f32::INFINITY),
                "neg_inf" => Ok(f32::NEG_INFINITY),
                "tau" => Ok(2.0 * PI),
                _ => {
                    todo!("Unknown identifier {self:?}")
                }
            },
            Rule::number => {
                let mut input = self.as_str().to_string();
                let mut multiplier = 1.0;
                if input.contains('K') {
                    multiplier = 1e3;
                    input = input.replace('K', ".");
                } else if input.contains('M') {
                    multiplier = 1e6;
                    input = input.replace('M', ".");
                } else if input.contains('G') {
                    multiplier = 1e9;
                    input = input.replace('G', ".");
                }
                Ok(multiplier * input.replace('_', "").parse::<f32>()?)
            }
            Rule::func_call => {
                let mut it = self.clone().into_inner();
                let func_name = it.next().expect("func_name");
                let func_name = func_name.as_str();
                let arg1 = it.next().expect("arg1");
                let arg1 = (arg1).eval()?;
                match func_name {
                    "sqrt" => Ok(arg1.sqrt()),
                    _ => {
                        todo!()
                    }
                }
            }
            Rule::term => {
                let mut it = self.clone().into_inner();
                let subterm = it.next().expect("subterm");
                let r = (subterm).eval()?;
                Ok(r)
            }
            Rule::expr1 => {
                let mut it = self.clone().into_inner();
                let expr3 = it.next().expect("expr3");
                let mut r = (expr3).eval()?;
                while let Some(operator) = it.next() {
                    let right = it.next().expect("right expr expected");
                    let right = (right).eval()?;
                    match operator.as_rule() {
                        Rule::addition => {
                            r += right;
                        }
                        Rule::minus => {
                            r -= right;
                        }
                        _ => {
                            todo!()
                        }
                    }
                }
                Ok(r)
            }
            Rule::expr2 => {
                let mut it = self.clone().into_inner();
                let expr3 = it.next().expect("expr3");
                let mut r = (expr3).eval()?;
                while let Some(operator) = it.next() {
                    let right = it.next().expect("right expr expected");
                    let right = (right).eval()?;
                    match operator.as_rule() {
                        Rule::multiply => {
                            r *= right;
                        }
                        Rule::division => {
                            r /= right;
                        }
                        Rule::modulus => {
                            r %= right;
                        }
                        _ => {
                            todo!()
                        }
                    }
                }
                Ok(r)
            }
            Rule::expr3 => {
                let mut it = self.clone().into_inner();
                let expr3 = it.next().expect("expr3");
                let mut r = (expr3).eval()?;
                while let Some(operator) = it.next() {
                    let right = it.next().expect("right expr expected");
                    let right = (right).eval()?;
                    match operator.as_rule() {
                        Rule::exponentiation => {
                            r = r.powf(right);
                        }
                        _ => {
                            todo!()
                        }
                    }
                }
                Ok(r)
            }
            Rule::expr4 => {
                let mut it = self.clone().into_inner();
                let first = it.next().expect("expr4");
                match first.as_rule() {
                    Rule::minus => {
                        let term = it.next().expect("expr4");
                        Ok(-1.0 * (term).eval()?)
                    }
                    Rule::term => first.eval(),
                    _ => {
                        todo!()
                    }
                }
            }
            _ => {
                let rule = self.as_rule();
                todo!("unknown: {rule:?}");
            }
        }
    }
}
