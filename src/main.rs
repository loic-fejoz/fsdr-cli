#[macro_use]
pub extern crate async_trait;
#[macro_use]
extern crate pest_derive;

use std::env;

use cmd_line::HighLevelCmdLine;
use self::grc::GrcParser;
use futuresdr::anyhow::{bail, Ok, Result};
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;
mod grc;
use grc::Grc;
mod csdr;
use csdr::CsdrParser;
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

fn load() -> Result<Grc> {
    let arg_count = env::args().count();
    if arg_count <= 1 {
        usage()?;
    }
    let mut args = env::args();
    let _cmd_line = args.next().expect("");
    let first_arg = args.next().expect("");
    if "grc" == first_arg {
        if arg_count <= 2 {
            usage()?;
        } else {
            let filename = args.next().expect("");
            return grc::GrcParser::load(filename);
        }
    } else if "--help" == first_arg || "help" == first_arg {
        if arg_count <= 2 {
            usage()?;
        } else {
            let cmd_name = args.next().expect("");
            todo!("Display help for {cmd_name}");
        }
    } else if "csdr" == first_arg {
        return CsdrParser::parse_multiple_commands(&mut args.peekable());
    } else {
        let args = env::args();
        let args = args.skip(1);
        return CsdrParser::parse_command(&mut args.peekable());
    }
    todo!();
}

fn main() -> Result<()> {
    let mut input = std::env::args();
    input.next(); // skip binary name
    let input: String = input.collect();
    let input = input.as_str();
    let input = cmd_grammar::CommandsParser::parse_main(input)?;

    if input.is_help_cmd() {
        usage()?;
        return Ok(());
    }
    let mut fg: Option<Grc> = None;
    if let Some(grc_cmd) = input.as_grc_cmd() {
        let filename = grc_cmd.filename();
        fg = Some(grc::GrcParser::load(filename)?);
    } else if let Some(csdr_cmd) = input.as_csdr_cmd() {
        fg = csdr_cmd.parse()?;
        if fg.is_none() { // happen for command like eval
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
