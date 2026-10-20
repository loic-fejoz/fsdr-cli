use anyhow::{bail, Context, Result};
use fsdr_cli::cmd_grammar::{self, Rule};
use fsdr_cli::cmd_line::HighLevelCmdLine;
use fsdr_cli::csdr_cmd::CsdrCmd;
use fsdr_cli::grc::converter::Grc2FutureSdr;
use fsdr_cli::grc::{Grc, GrcParser};
use fsdr_cli::grc_cmd::GrcCmd;
use fsdr_cli::iqengine_cmd::IQEngineCmd;
use futuresdr::runtime::{Flowgraph, Runtime};
use itertools::join;
use pest::error::ErrorVariant;
use std::{eprintln, println};

fn usage() -> Result<Grc> {
    let msg = "Usage:\n\
    \tfsdr-cli grc file.grc\n\
    \tfsdr-cli iqengine [conf.yml]\n\
    \tfsdr-cli  function_name <function_param1> <function_param2> [optional_param] ....\n\
    \tfsdr-cli \"csdr ... \\| [csdr] ....\" \n\
    \tfsdr-cli \"csdr ... ! [csdr] ....\" \n\
\n";
    bail!(msg);
}

fn main() -> Result<()> {
    let mut input = std::env::args();
    input.next(); // skip binary name

    // Get back all the arguments as a big one command line ready to be parsed
    let one_liner = join(input, " ");
    let one_liner = one_liner.trim();
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
                            let marker = "-".repeat(x.max(1) - 1) + "^";
                            eprintln!("\x1b[93m{marker}\x1b[0m");
                        }
                        pest::error::InputLocation::Span(range) => {
                            let marker = " ".repeat(range.0) + &("^".repeat(range.1 - range.0));
                            eprintln!("\x1b[93m{marker}\x1b[0m");
                        }
                    }
                    let positives = join(positives.iter().map(|r| format!("{r:?}")), " ");
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
    if let Some(iqengine_cmd) = input.as_iqengine_cmd() {
        #[cfg(not(feature = "iqengine"))]
        {
            bail!("iqengine feature not available. Please download another version.");
        }
        #[cfg(feature = "iqengine")]
        {
            let filename = iqengine_cmd.iqengine_configuration();
            return fsdr_cli::iqengine_plugin::start_iqengine_daemon(filename);
        }
    } else if let Some(grc_cmd) = input.as_grc_cmd() {
        let filename = grc_cmd.filename();
        fg = Some(GrcParser::load(filename)?);
    } else if let Some(csdr_cmd) = input.as_csdr_cmd() {
        fg = csdr_cmd.parse()?;
        if fg.is_none() {
            // happen for command like eval
            return Ok(());
        }

        if let Some(output) = csdr_cmd.output()? {
            let fg = fg.context("Failed to get flowgraph for saving")?;
            GrcParser::save(output, &fg).context("failed to save GRC file")?;
            println!("Flowgraph saved into {output:?}");
            return Ok(());
        }
    }

    let fg = fg.context("No flowgraph was defined. Please check your command line arguments.")?;
    let fg: Flowgraph = Grc2FutureSdr::new().convert_grc(fg)?;
    Runtime::new().run(fg)?;
    Ok(())
}
