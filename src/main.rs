#[macro_use]
pub extern crate async_trait;
#[macro_use]
extern crate pest_derive;

use cmd_grammar::Rule;
use pest::error::ErrorVariant;
use std::{eprintln, println};

use self::grc::GrcParser;
use cmd_line::HighLevelCmdLine;
use futuresdr::anyhow::{bail, Ok, Result};
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;
mod grc;
use grc::Grc;
// mod csdr;
// use csdr::CsdrParser;
use grc::converter::Grc2FutureSdr;
pub mod blocks;
pub mod cmd_line;
pub mod csdr_cmd;
use crate::csdr_cmd::CsdrCmd;
pub mod cmd_grammar;
pub mod grc_cmd;
use grc_cmd::GrcCmd;

fn usage() -> Result<Grc> {
    let msg = "Usage:\n\
    \tfsdr-cli grc file.grc
    \tfsdr-cli  function_name <function_param1> <function_param2> [optional_param] ....\n\
    \tfsdr-cli \"csdr ... \\| [csdr] ....\" \n\
    \tfsdr-cli \"csdr ... ! [csdr] ....\" \n\
\n";
    bail!(msg);
}

fn join(iter: impl Iterator<Item = String>) -> String {
    iter.fold(String::new(), |mut a, b| {
        a.reserve(b.len() + 1);
        a.push(' ');
        a.push_str(&b);
        a
    })
}

fn main() -> Result<()> {
    let mut input = std::env::args();
    input.next(); // skip binary name

    // Get back all the arguments as a big one command line ready to be parsed
    let one_liner = join(input);
    // let one_liner = input.fold(String::new(), |mut a, b| {
    //     a.reserve(b.len() + 1);
    //     a.push(' ');
    //     a.push_str(&b);
    //     a
    // });
    let one_liner = one_liner.trim();
    //println!("actual input: '{input}'");
    let input = cmd_grammar::CommandsParser::parse_main(one_liner);

    if let Err(err) = input {
        match err.downcast_ref::<pest::error::Error<Rule>>() {
            Some(err) => match &err.variant {
                ErrorVariant::ParsingError {
                    positives,
                    negatives: _,
                } => {
                    eprintln!("\x1b[0;31mParsing error:\x1b[0m");
                    eprintln!("{one_liner}");
                    match err.location {
                        pest::error::InputLocation::Pos(x) => {
                            let marker = "-".repeat(x - 1) + "^";
                            eprintln!("\x1b[93m{marker}\x1b[0m");
                        }
                        pest::error::InputLocation::Span(range) => {
                            let marker = " ".repeat(range.0) + &("^".repeat(range.1 - range.0));
                            eprintln!("\x1b[93m{marker}\x1b[0m");
                        }
                    }
                    let positives = join(positives.iter().map(|r| format!("{r:?}")));
                    eprintln!("help: Expecting one of{positives}");
                }
                ErrorVariant::CustomError { message: _ } => {
                    eprintln!("{err}");
                }
            },
            None => {
                eprintln!("{err}");
            }
        }
        std::process::exit(1);
    }
    let input = input?;

    if input.is_help_cmd() {
        usage()?;
        return Ok(());
    }
    let mut fg: Option<Grc> = None;
    if let Some(grc_cmd) = input.as_grc_cmd() {
        let filename = grc_cmd.filename();
        // println!("Loading {filename}...");
        fg = Some(grc::GrcParser::load(filename)?);
    } else if let Some(csdr_cmd) = input.as_csdr_cmd() {
        fg = csdr_cmd.parse()?;
        if fg.is_none() {
            // happen for command like eval
            return Ok(());
        }

        if let Some(output) = csdr_cmd.output() {
            let fg = fg.expect("");
            GrcParser::save(output, &fg);
            println!("Flowgraph saved into {output:?}");
            return Ok(());
        }
    }

    let fg = fg.expect("undefined flowgraph");
    let fg: Flowgraph = Grc2FutureSdr::convert_grc(fg)?;
    Runtime::new().run(fg)?;
    Ok(())
}
