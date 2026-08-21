use crate::cmd_grammar::{CommandsParser, Rule};
use crate::grc::builder::{GraphLevel, GrcBuilder};
use crate::grc::Grc;
use anyhow::{bail, Context, Result};
use pest::iterators::Pair;
use pest::Parser;

use self::add_dcoffset_cmd::AddDcOffsetCmd;
use self::agc_cmd::AgcCmd;
use self::amdemod_cmd::AmDemodCmd;
use self::audio_cmd::AudioCmd;
use self::bandpass_fir_fft_cmd::BandpassFirFftcmd;
use self::binary_slicer::BinarySlicerCmd;
use self::cat_server_cmd::CatServerCmd;
use self::clipdetect_cmd::ClipDetectCmd;
use self::clone_cmd::CloneCmd;
use self::convert_cmd::ConvertCmd;
use self::costas_loop_cmd::CostasLoopCmd;
use self::ctcss_gen_cmd::CtcssGenCmd;
use self::dbpsk_cmd::DBPskCmd;
use self::dcblock_cmd::DcBlockCmd;
use self::decimating_shift_addition_cmd::DecimatingShiftAdditionCmd;
use self::deemphasis_nfm_ff_cmd::DeemphasisNfnCmd;
use self::deemphasis_wfm_ff_cmd::DeemphasisWfmCmd;
use self::dsb_cmd::DsbCmd;
use self::dump_cmd::DumpCmd;
use self::eval_cmd::EvalCmd;
use self::fastdcblock_cmd::FastDCBlockCmd;
use self::fft_cmd::FftCmd;
use self::fft_exchange_sides_cmd::FftExchangeSidesCmd;
use self::fir_decimate_cmd::FirDecimateCmd;
use self::fixedlen_to_pdu_cmd::FixedlenToPduCmd;
use self::flowcontrol_cmd::FlowcontrolCmd;
use self::fmdemod_quadri_cmd::FmDemodQuadriCmd;
use self::fractional_decimator_cmd::FractionalDecimatorCmd;
use self::gain_cmd::GainCmd;
use self::limit_cmd::LimitCmd;
use self::load_cmd::LoadCmd;
use self::load_kiss_cmd::LoadKissCmd;
use self::logaveragepower_cmd::LogAveragePowerCmd;
use self::logpower_cmd::LogPowerCmd;
use self::mono2stereo_cmd::Mono2StereoCmd;
use self::octave_complex_cmd::OctaveComplexCmd;
use self::pack_bits_cmd::PackBitsCmd;
use self::pattern_search_cmd::PatternSearchCmd;
use self::rational_resampler_cmd::RationalResamplerCmd;
use self::realpart_cmd::RealPartCmd;
use self::repeat_cmd::RepeatCmd;
use self::save_kiss_cmd::SaveKissCmd;
use self::shift_addition_cmd::ShiftAdditionCmd;
use self::tcp_kiss_client_cmd::TcpKissClientCmd;
use self::tcp_kiss_server_cmd::TcpKissServerCmd;
use self::throttle_cmd::ThrottleCmd;
use self::timing_recovery_cmd::TimingRecoveryCmd;
use self::varicode_cmd::VaricodeCmd;
use self::weaver_cmd::WeaverCmd;

mod add_dcoffset_cmd;
mod agc_cmd;
mod amdemod_cmd;
mod audio_cmd;
mod bandpass_fir_fft_cmd;
mod binary_slicer;
mod cat_server_cmd;
mod clipdetect_cmd;
mod clone_cmd;
mod convert_cmd;
mod costas_loop_cmd;
mod ctcss_gen_cmd;
mod dbpsk_cmd;
mod dcblock_cmd;
mod decimating_shift_addition_cmd;
mod deemphasis_nfm_ff_cmd;
mod deemphasis_wfm_ff_cmd;
mod dsb_cmd;
mod dump_cmd;
pub mod eval_cmd;
mod fastdcblock_cmd;
mod fft_cmd;
mod fft_exchange_sides_cmd;
mod fir_decimate_cmd;
mod fixedlen_to_pdu_cmd;
mod flowcontrol_cmd;
mod fmdemod_quadri_cmd;
mod fractional_decimator_cmd;
mod gain_cmd;
mod limit_cmd;
mod load_cmd;
mod load_kiss_cmd;
mod logaveragepower_cmd;
mod logpower_cmd;
mod mono2stereo_cmd;
mod octave_complex_cmd;
mod pack_bits_cmd;
mod pattern_search_cmd;
mod rational_resampler_cmd;
mod realpart_cmd;
mod repeat_cmd;
mod save_kiss_cmd;
mod shift_addition_cmd;
mod tcp_kiss_client_cmd;
mod tcp_kiss_server_cmd;
mod throttle_cmd;
mod timing_recovery_cmd;
mod varicode_cmd;
mod weaver_cmd;

pub trait AnyCmd<'i> {
    fn parse(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>>;
}

impl<'i> AnyCmd<'i> for Pair<'i, Rule> {
    fn parse(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        match self.as_rule() {
            Rule::add_dcoffset_cc_cmd => self.build_add_dcoffset_cc(grc),
            Rule::agc_cmd => self.build_agc(grc),
            Rule::amdemod_cmd => self.build_amdemod(grc),
            Rule::audio_cmd => self.build_audio_sink(grc),
            Rule::bandpass_fir_fft_cc_cmd => self.build_bandpass_fir_fft_cc(grc),
            Rule::binary_slicer_cmd => self.build_binary_slicer(grc),
            Rule::bpsk_costas_loop_cmd | Rule::pll_cmd => self.build_costas_loop(grc),
            Rule::cat_server_cmd => CatServerCmd::build_cat_server(self, grc),
            Rule::clone_cmd => self.build_clone(grc),
            Rule::ctcss_gen_cmd => CtcssGenCmd::build_ctcss_gen(self, grc),
            Rule::clipdetect_cmd => self.build_clipdetect(grc),
            Rule::convert_cmd => self.build_convert(grc),
            Rule::dbpsk_cmd => self.build_dbpsk_decoder(grc),
            Rule::dcblock_cmd => self.build_dcblock_ff(grc),
            Rule::decimating_shift_addition_cmd => self.build_decimating_shift_addition_cc(grc),
            Rule::deemphasis_nfm_cmd => self.build_deemphasis_nfm(grc),
            Rule::deemphasis_wfm_cmd => self.build_deemphasis_wfm(grc),
            Rule::dsb_cmd => self.build_dsb(grc),
            Rule::dump_cmd => self.build_dump(grc),
            Rule::eval_cmd => {
                self.execute_eval()?;
                Ok(grc)
            }
            Rule::fastdcblock_cmd => self.build_fastdcblock(grc),
            Rule::fft_cc_cmd => self.build_fft_cc(grc),
            Rule::fft_fc_cmd => self.build_fft_fc(grc),
            Rule::fft_exchange_sides_ff_cmd => self.build_fft_exchange_sides_ff(grc),
            Rule::flowcontrol_cmd => self.build_flowcontrol(grc),
            Rule::fractional_decimator_cmd => self.build_fractional_decimator(grc),
            Rule::fir_decimate_cmd => self.build_fir_decimate(grc),
            Rule::fmdemod_quadri_cmd => self.build_fm_demod_quadri(grc),
            Rule::fmdemod_atan_cmd => self.build_fm_demod_atan(grc),
            Rule::gain_cmd => self.build_gain(grc),
            Rule::limit_cmd => self.build_limit(grc),
            Rule::load_cmd => self.build_load(grc),
            Rule::load_kiss_cmd => self.build_load_kiss(grc),
            Rule::logaveragepower_cf_cmd => self.build_logaveragepower_cf(grc),
            Rule::logpower_cf_cmd => self.build_logpower_cf(grc),
            Rule::fixedlen_to_pdu_cmd => self.build_fixedlen_to_pdu(grc),
            Rule::mono2stereo_s16_cmd => self.build_mono2stereo_s16(grc),
            Rule::none_cmd => self.build_none(grc),
            Rule::save_kiss_cmd => self.build_save_kiss(grc),
            Rule::tcp_kiss_server_cmd => self.build_tcp_kiss_server(grc),
            Rule::tcp_kiss_client_cmd => self.build_tcp_kiss_client(grc),
            Rule::octave_complex_cmd => self.build_octave_complex(grc),
            Rule::pack_bits_cmd => self.build_pack_bits(grc),
            Rule::pattern_search_cmd => self.build_pattern_search(grc),
            Rule::psk31_varicode_decoder_cmd => self.build_varicode_decoder(grc),
            Rule::psk31_varicode_encoder_cmd => self.build_varicode_encoder(grc),
            Rule::rational_resampler_cmd => self.build_rational_resampler(grc),
            Rule::realpart_cmd => self.build_realpart(grc),
            Rule::repeat_u8_cmd => self.build_repeat_u8(grc),
            Rule::shift_addition_cmd => self.build_shift_addition(grc),
            Rule::throttle_cmd => self.build_throttle(grc),
            Rule::timing_recovery_cmd => self.build_timing_recovery(grc),
            Rule::weaver_lsb_cmd | Rule::weaver_usb_cmd => self.build_weaver(grc),

            Rule::csdr_save_opt => Ok(grc),
            _ => {
                let rule = self.as_rule();
                bail!("unknown any cmd: {rule:?}");
            }
        }
    }
}

pub trait CsdrCmd<'i> {
    fn output(&self) -> Result<Option<&'i str>>;
    fn parse(&self) -> Result<Option<Grc>>;
}

impl<'i> CsdrCmd<'i> for Pair<'i, Rule> {
    fn output(&self) -> Result<Option<&'i str>> {
        let cmd = self.clone();
        let mut args = cmd.into_inner();
        if let Some(first_inner) = args.next() {
            match first_inner.as_rule() {
                Rule::csdr_save_opt => {
                    let filename = first_inner
                        .into_inner()
                        .next()
                        .context("output filepath expected")?;
                    let filename = filename.as_str();
                    Ok(Some(filename))
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    fn parse(&self) -> Result<Option<Grc>> {
        let mut grc_builder = GrcBuilder::new();
        if self.as_rule() == Rule::csdr_cmd {
            for sub_cmd in self.clone().into_inner() {
                grc_builder = AnyCmd::parse(&sub_cmd, grc_builder)?;
            }
        } else {
            grc_builder = AnyCmd::parse(self, grc_builder)?;
        }
        grc_builder.ensure_sink()?;
        let grc = grc_builder.build()?;
        Ok(Some(grc))
    }
}

#[derive(Default)]
pub struct CsdrParser {}

impl CsdrParser {
    pub fn parse_command(cmd: &str) -> Result<Option<Grc>> {
        // let cmd = CommandsParser::parse_main(cmd.into())?;
        // CsdrCmd::parse(&cmd)

        let input = CommandsParser::parse(Rule::any_csdr_cmd, cmd)?
            .next()
            .context("Parsing commands")?;
        let grc_builder = GrcBuilder::new();
        let mut grc_builder = AnyCmd::parse(&input, grc_builder)?;
        grc_builder.ensure_sink()?;
        let grc = grc_builder.build()?;
        Ok(Some(grc))
    }

    pub fn parse_multiple_commands(cmd: &str) -> Result<Option<Grc>> {
        let cmd = CommandsParser::parse_main(cmd)?;
        CsdrCmd::parse(&cmd)
    }
}
