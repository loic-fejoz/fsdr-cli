use crate::cmd_grammar::CommandsParser;
use crate::csdr_cmd::eval_cmd::EvalCmd;
use crate::grc::Grc;
use futuresdr::anyhow::bail;
use futuresdr::anyhow::Context;
use futuresdr::anyhow::Result;
use futuresdr::runtime::Flowgraph;
use std::collections::BTreeMap;

use super::converter_helper::*;
pub mod analog_agc_xx;
use self::analog_agc_xx::AnalogAgcXxConverter;
pub mod analog_fm_deemph;
use self::analog_fm_deemph::AnalogFmDeemphConverter;
pub mod analog_nfm_deemph;
use self::analog_nfm_deemph::DeemphasisNfmConverter;
pub mod analog_rail_ff;
use self::analog_rail_ff::AnalogRailFfConverter;
pub mod audio_sink;
use self::audio_sink::AudioSinkConverter;
use super::BlockInstance;
pub mod band_pass_filter;
use self::band_pass_filter::BandPassFilterConverter;
pub mod blocks_add_const_vxx;
use self::blocks_add_const_vxx::AddConstVxConverter;
pub mod blocks_deinterleave;
use self::blocks_deinterleave::DeinterleaveBlockConverter;
pub mod blocks_complex_to_real;
use self::blocks_complex_to_real::RealpartCfConverter;
pub mod convert;
use self::convert::ConvertBlockConverter;
pub mod blocks_file_sink;
use self::blocks_file_sink::FileSinkConverter;
pub mod blocks_file_source;
use self::blocks_file_source::FileSourceConverter;
pub mod blocks_float_to_complex;
use self::blocks_float_to_complex::FloatToComplexConverter;
pub mod blocks_freqshift_cc;
use self::blocks_freqshift_cc::FreqShiftCcConverter;
pub mod blocks_multiply_const_vxx;
use self::blocks_multiply_const_vxx::MulConstVxConverter;
pub mod blocks_null_sink;
use self::blocks_null_sink::NullSinkConverter;
pub mod blocks_throttle;
use self::blocks_throttle::ThrottleConverter;
pub mod blocks_complex_to_mag;
use self::blocks_complex_to_mag::ComplexToMagConverter;
pub mod clipdetect_ff;
use self::clipdetect_ff::ClipDetectFfConverter;
pub mod dc_bloker_xx;
use self::dc_bloker_xx::DcBlockerXx;
pub mod dump;
use self::dump::DumpConverter;
pub mod fir_filter_xx;
use self::fir_filter_xx::FirFilterXxConverter;
pub mod analog_quadrature_demod;
use self::analog_quadrature_demod::AnalogQuadratureDemoConverter;
pub mod low_pass_filter;
use self::low_pass_filter::LowPassFilterConverter;
pub mod octave_complex_c;
use self::octave_complex_c::OctaveComplexConverter;
pub mod rational_resampler_xxx;
use self::rational_resampler_xxx::RationalResamplerXxConverter;
pub mod weaver_ssb;
use self::weaver_ssb::WeaverSsbConverter;

#[derive(Default, Clone)]
pub struct Grc2FutureSdr;

impl Grc2FutureSdr {
    fn block_converter(blk_def: &BlockInstance) -> Result<Box<dyn BlockConverter>> {
        let blk_type = &(blk_def.id[..]);
        let cvter: Box<dyn BlockConverter> = match blk_type {
            "analog_agc_xx" => Box::new(AnalogAgcXxConverter {}),
            "analog_quadrature_demod_cf" => Box::new(AnalogQuadratureDemoConverter {}),
            "analog_rail_ff" => Box::new(AnalogRailFfConverter {}),
            "band_pass_filter" => Box::new(BandPassFilterConverter {}),
            "audio_sink" => Box::new(AudioSinkConverter {}),
            "blocks_add_const_vxx" => Box::new(AddConstVxConverter {}),
            "blocks_deinterleave" => Box::new(DeinterleaveBlockConverter {}),
            "blocks_file_sink" => Box::new(FileSinkConverter {}),
            "blocks_file_source" => Box::new(FileSourceConverter {}),
            "blocks_float_to_complex" => Box::new(FloatToComplexConverter {}),
            "blocks_freqshift_cc" => Box::new(FreqShiftCcConverter {}),
            "blocks_multiply_const_vxx" => Box::new(MulConstVxConverter {}),
            "blocks_uchar_to_float"
            | "blocks_char_to_float"
            | "convert_s16_f"
            | "blocks_float_to_uchar"
            | "blocks_float_to_char"
            | "blocks_float_to_short"
            | "convert_ff_c"
            | "blocks_short_to_float" => Box::new(ConvertBlockConverter {}),
            "blocks_null_sink" => Box::new(NullSinkConverter {}),
            "dump_u8" | "dump_f" => Box::new(DumpConverter {}),
            "throttle_ff" | "blocks_throttle" => Box::new(ThrottleConverter {}),
            "realpart_cf" | "blocks_complex_to_real" => Box::new(RealpartCfConverter {}),
            "blocks_complex_to_mag" => Box::new(ComplexToMagConverter {}),
            "clipdetect_ff" => Box::new(ClipDetectFfConverter {}),
            "dc_blocker_xx" => Box::new(DcBlockerXx {}),
            "deemphasis_nfm_ff" | "analog_nfm_deemph" => Box::new(DeemphasisNfmConverter {}),
            "analog_fm_deemph" => Box::new(AnalogFmDeemphConverter {}),
            "fir_filter_xxx" => Box::new(FirFilterXxConverter {}),
            "low_pass_filter" => Box::new(LowPassFilterConverter {}),
            "octave_complex_c" => Box::new(OctaveComplexConverter {}),
            "rational_resampler_xxx" => Box::new(RationalResamplerXxConverter {}),
            "weaver_usb_cf" | "weaver_lsb_cf" => Box::new(WeaverSsbConverter {}),
            _ => bail!("Unknown GNU Radio block {blk_type}"),
        };
        Ok(cvter)
    }

    #[allow(dead_code)]
    pub fn convert_block(
        fg: &mut Flowgraph,
        blk: &BlockInstance,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        Grc2FutureSdr::block_converter(blk)?.convert(blk, fg)
    }

    pub fn convert_grc(grc: Grc) -> Result<Flowgraph> {
        let mut fg = Flowgraph::new();
        let fsdr_blocks = grc
            .blocks
            .iter()
            .map(|blk| -> Result<Box<dyn ConnectorAdapter>> {
                Grc2FutureSdr::block_converter(blk)?.convert(blk, &mut fg)
            });
        let names: Vec<String> = grc.blocks.iter().map(|blk| blk.name.clone()).collect();
        let mut names_to_adapter = BTreeMap::<String, Box<dyn ConnectorAdapter>>::new();

        for (name, adapter) in names.iter().zip(fsdr_blocks) {
            let adapter = adapter?;
            names_to_adapter.insert(name.clone(), adapter);
        }
        for connection in grc.connections {
            let src_blk = connection[0].clone();
            let src_blk = names_to_adapter
                .get(&src_blk)
                .context("unfound source block: {src_blk}")?;
            let src_port = connection[1].clone();
            let (src_blk, src_port) = src_blk.adapt_output_port(&src_port)?;

            let tgt_blk = connection[2].clone();
            let tgt_blk = names_to_adapter
                .get(&tgt_blk)
                .context("unfound target block: {tgt_blk}")?;
            let tgt_port = connection[3].clone();
            let (tgt_blk, tgt_port) = tgt_blk.adapt_input_port(&tgt_port)?;

            fg.connect_stream(src_blk, src_port, tgt_blk, tgt_port)
                .context("connecting {connection}")?;
        }
        Ok(fg)
    }

    fn parameter_as_f32<'i>(
        blk_def: &'i BlockInstance,
        key: &'i str,
        default_value: impl Into<&'i str>,
    ) -> Result<f32> {
        let expr = blk_def.parameter_or(key, default_value);
        let expr = CommandsParser::parse_expr(expr)?;
        EvalCmd::eval(&expr)
    }

    fn parameter_as_f64<'i>(
        blk_def: &'i BlockInstance,
        key: &'i str,
        default_value: impl Into<&'i str>,
    ) -> Result<f64> {
        let value = Grc2FutureSdr::parameter_as_f32(blk_def, key, default_value)?;
        Ok(value as f64)
    }
}
