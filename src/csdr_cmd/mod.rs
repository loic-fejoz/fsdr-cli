use crate::cmd_grammar::{CommandsParser, Rule};
use crate::grc::builder::{GraphLevel, GrcBuilder};
use crate::grc::Grc;
use futuresdr::anyhow::{bail, Context, Result};
use pest::iterators::Pair;
use pest::Parser;

use self::agc_cmd::AgcCmd;
use self::amdemod_cmd::AmDemodCmd;
use self::bandpass_fir_fft_cmd::BandpassFirFftcmd;
use self::clipdetect_cmd::ClipDetectCmd;
use self::convert_cmd::ConvertCmd;
use self::deemphasis_nfm_ff_cmd::DeemphasisNfnCmd;
use self::deemphasis_wfm_ff_cmd::DeemphasisWfmCmd;
use self::dump_cmd::DumpCmd;
use self::eval_cmd::EvalCmd;
use self::fastdcblock_cmd::FastDCBlockCmd;
use self::fir_decimate_cmd::FirDecimateCmd;
use self::fmdemod_quadri_cmd::FmDemodQuadriCmd;
use self::fractional_decimator_cmd::FractionalDecimatorCmd;
use self::limit_cmd::LimitCmd;
use self::load_cmd::LoadCmd;
use self::octave_complex_cmd::OctaveComplexCmd;
use self::realpart_cmd::RealPartCmd;
use self::shift_addition_cmd::ShiftAdditionCmd;
use self::throttle_cmd::ThrottleCmd;

mod agc_cmd;
mod amdemod_cmd;
mod bandpass_fir_fft_cmd;
mod clipdetect_cmd;
mod convert_cmd;
mod deemphasis_nfm_ff_cmd;
mod deemphasis_wfm_ff_cmd;
mod dump_cmd;
pub mod eval_cmd;
mod fastdcblock_cmd;
mod fir_decimate_cmd;
mod fmdemod_quadri_cmd;
mod fractional_decimator_cmd;
mod limit_cmd;
mod load_cmd;
mod octave_complex_cmd;
mod realpart_cmd;
mod shift_addition_cmd;
mod throttle_cmd;

pub trait AnyCmd<'i> {
    fn parse(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
}

impl<'i> AnyCmd<'i> for Pair<'i, Rule> {
    fn parse(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        match self.as_rule() {
            Rule::agc_cmd => self.build_agc(grc),
            Rule::amdemod_cmd => self.build_amdemod(grc),
            Rule::bandpass_fir_fft_cc_cmd => self.build_bandpass_fir_fft_cc(grc),
            Rule::clipdetect_cmd => self.build_clipdetect(grc),
            Rule::convert_cmd => self.build_convert(grc),
            Rule::deemphasis_nfm_cmd => self.build_deemphasis_nfm(grc),
            Rule::deemphasis_wfm_cmd => self.build_deemphasis_wfm(grc),
            Rule::dump_cmd => self.build_dump(grc),
            Rule::eval_cmd => {
                self.execute_eval()?;
                Ok(grc)
            }
            Rule::fastdcblock_cmd => self.build_fastdcblock(grc),
            Rule::fractional_decimator_cmd => self.build_fractional_decimator(grc),
            Rule::fir_decimate_cmd => self.build_fir_decimate(grc),
            Rule::fmdemod_quadri_cmd => self.build_fm_demod_quadri(grc),
            Rule::limit_cmd => self.build_limit(grc),
            Rule::load_cmd => self.build_load(grc),
            Rule::octave_complex_cmd => self.build_octave_complex(grc),
            Rule::realpart_cmd => self.build_realpart(grc),
            Rule::shift_addition_cmd => self.build_shift_addition(grc),
            Rule::throttle_cmd => self.build_throttle(grc),

            Rule::csdr_save_opt => Ok(grc),
            _ => {
                let rule = self.as_rule();
                bail!("unknown any cmd: {rule:?}");
            }
        }
    }
}

pub trait CsdrCmd<'i> {
    fn output(&self) -> Option<&'i str>;
    fn parse(&self) -> Result<Option<Grc>>;
}

impl<'i> CsdrCmd<'i> for Pair<'i, Rule> {
    fn output(&self) -> Option<&'i str> {
        let cmd = self.clone();
        let mut args = cmd.into_inner();
        if let Some(first_inner) = args.next() {
            match first_inner.as_rule() {
                Rule::csdr_save_opt => {
                    let filename = first_inner
                        .into_inner()
                        .next()
                        .expect("output filepath expected");
                    let filename = filename.as_str();
                    Some(filename)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn parse(&self) -> Result<Option<Grc>> {
        let mut grc_builder = GrcBuilder::new();
        for sub_cmd in self.clone().into_inner() {
            grc_builder = AnyCmd::parse(&sub_cmd, grc_builder)?;
        }
        grc_builder.ensure_sink();
        let grc = grc_builder.build()?;
        Ok(Some(grc))
    }
}

#[derive(Default)]
pub struct CsdrParser {}

impl CsdrParser {
    pub fn parse_command<'i>(cmd: impl Into<&'i str>) -> Result<Option<Grc>> {
        // let cmd = CommandsParser::parse_main(cmd.into())?;
        // CsdrCmd::parse(&cmd)

        let input = CommandsParser::parse(Rule::any_csdr_cmd, cmd.into())?
            .next()
            .context("Parsing commands")?;
        let grc_builder = GrcBuilder::new();
        let mut grc_builder = AnyCmd::parse(&input, grc_builder)?;
        grc_builder.ensure_sink();
        let grc = grc_builder.build()?;
        Ok(Some(grc))
    }

    pub fn parse_multiple_commands<'i>(cmd: impl Into<&'i str>) -> Result<Option<Grc>> {
        let cmd = CommandsParser::parse_main(cmd.into())?;
        CsdrCmd::parse(&cmd)
    }
}
