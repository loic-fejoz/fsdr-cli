use std::env;

use futuresdr::anyhow::{bail, Result};
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;
mod grc;
use grc::Grc;
mod csdr;
use csdr::CsdrParser;
use grc::converter::Grc2FutureSdr;

fn usage() -> Result<Grc> {
    let msg = "Usage:\n\
    \tfsdr-cli  function_name <function_param1> <function_param2> [optional_param] ....\n\
    \tfsdr-cli \\\"csdr ... \\| csdr ....\\\" \n\
    \tfsdr-cli grc file.grc\n";
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
        return CsdrParser::parse_multiple_commands();
    } else {
        let args = env::args();
        let args = args.skip(1);
        return CsdrParser::parse_command(args.peekable());
    }
    todo!();
}

fn main() -> Result<()> {
    let fg = load()?;
    let fg: Flowgraph = Grc2FutureSdr::convert_grc(fg)?;
    Runtime::new().run(fg)?;
    Ok(())
}
