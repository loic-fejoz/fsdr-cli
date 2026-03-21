//! This module contains converters that map GNU Radio Companion (GRC) block definitions
//! to FutureSDR kernels.

use crate::cmd_grammar::CommandsParser;
use crate::csdr_cmd::eval_cmd::EvalCmd;
use crate::grc::backend::{FsdrBackend, RuntimeBackend};
use crate::grc::Grc;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use futuresdr::runtime::Flowgraph;
use std::collections::BTreeMap;
use std::collections::HashMap;

use super::converter_helper::*;
pub mod analog_agc_xx;
pub mod audio_sink;
use super::BlockInstance;
pub mod blocks_multiply_const_vxx;
pub mod fir_filter_xx;

pub mod analog_fm_deemph;
pub mod analog_nfm_deemph;
pub mod analog_quadrature_demod;
pub mod analog_rail_ff;
pub mod band_pass_filter;
pub mod blocks_add_const_vxx;
pub mod blocks_complex_to_mag;
pub mod blocks_complex_to_real;
pub mod blocks_deinterleave;
pub mod blocks_file_sink;
pub mod blocks_file_source;
pub mod blocks_float_to_complex;
pub mod blocks_freqshift_cc;
pub mod blocks_null_sink;
pub mod blocks_pack_k_bits;
pub mod blocks_throttle;
pub mod clipdetect_ff;
pub mod convert;
pub mod dc_bloker_xx;
pub mod digital_binary_slicer;
pub mod dsb;
pub mod dump;
pub mod low_pass_filter;
pub mod octave_complex_c;
pub mod pattern_search;
pub mod rational_resampler_xxx;
pub mod satellites_fixedlen_to_pdu;
pub mod satellites_kiss_client_source;
pub mod satellites_kiss_file_sink;
pub mod satellites_kiss_file_source;
pub mod satellites_kiss_server_sink;
pub mod timing_recovery;
pub mod weaver_ssb;

#[derive(Default)]
pub struct Grc2FutureSdr<B: FsdrBackend> {
    specific_converter: HashMap<String, Box<dyn MutBlockConverter<B>>>,
}

impl<B: FsdrBackend> Grc2FutureSdr<B> {
    pub fn new() -> Grc2FutureSdr<B> {
        Grc2FutureSdr {
            specific_converter: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn take(
        &mut self,
        k: &str,
    ) -> std::option::Option<Box<dyn MutBlockConverter<B> + 'static>> {
        self.specific_converter.remove(k)
    }

    #[allow(dead_code)]
    pub fn with_blocktype_conversion(
        &mut self,
        blocktype: impl ToString,
        f: Box<dyn MutBlockConverter<B>>,
    ) {
        self.specific_converter.insert(blocktype.to_string(), f);
    }

    fn block_converter(blk_def: &BlockInstance) -> Result<Box<dyn BlockConverter<B>>> {
        let blk_type = &(blk_def.id[..]);
        let cvter: Box<dyn BlockConverter<B>> = match blk_type {
            "analog_agc_xx" => Box::new(self::analog_agc_xx::AnalogAgcXxConverter {}),
            "analog_quadrature_demod_cf" => {
                Box::new(self::analog_quadrature_demod::AnalogQuadratureDemoConverter {})
            }
            "analog_rail_ff" => Box::new(self::analog_rail_ff::AnalogRailFfConverter {}),
            "band_pass_filter" => Box::new(self::band_pass_filter::BandPassFilterConverter {}),
            "audio_sink" => Box::new(self::audio_sink::AudioSinkConverter {}),
            "blocks_add_const_vxx" => Box::new(self::blocks_add_const_vxx::AddConstVxConverter {}),
            "blocks_deinterleave" => {
                Box::new(self::blocks_deinterleave::DeinterleaveBlockConverter {})
            }
            "digital_binary_slicer_fb" => {
                Box::new(self::digital_binary_slicer::DigitalBinarySlicerConverter {})
            }
            "dsb" => Box::new(self::dsb::DsbConverter {}),
            "blocks_file_sink" => Box::new(self::blocks_file_sink::FileSinkConverter {}),
            "blocks_file_source" => Box::new(self::blocks_file_source::FileSourceConverter {}),
            "blocks_float_to_complex" => {
                Box::new(self::blocks_float_to_complex::FloatToComplexConverter {})
            }
            "blocks_freqshift_cc" => Box::new(self::blocks_freqshift_cc::FreqShiftCcConverter {}),
            "blocks_multiply_const_vxx" => {
                Box::new(self::blocks_multiply_const_vxx::MulConstVxConverter {})
            }
            "blocks_uchar_to_float"
            | "blocks_char_to_float"
            | "convert_s16_f"
            | "blocks_float_to_uchar"
            | "blocks_float_to_char"
            | "blocks_float_to_short"
            | "convert_ff_c"
            | "blocks_short_to_float" => Box::new(self::convert::ConvertBlockConverter {}),
            "blocks_null_sink" => Box::new(self::blocks_null_sink::NullSinkConverter {}),
            "dump_u8" | "dump_f" | "dump_c" => Box::new(self::dump::DumpConverter {}),
            "throttle_ff" | "blocks_throttle" => {
                Box::new(self::blocks_throttle::ThrottleConverter {})
            }
            "realpart_cf" | "blocks_complex_to_real" => {
                Box::new(self::blocks_complex_to_real::RealpartCfConverter {})
            }
            "blocks_complex_to_mag" => {
                Box::new(self::blocks_complex_to_mag::ComplexToMagConverter {})
            }
            "clipdetect_ff" => Box::new(self::clipdetect_ff::ClipDetectFfConverter {}),
            "dc_blocker_xx" => Box::new(self::dc_bloker_xx::DcBlockerXx {}),
            "deemphasis_nfm_ff" | "analog_nfm_deemph" => {
                Box::new(self::analog_nfm_deemph::DeemphasisNfmConverter {})
            }
            "analog_fm_deemph" => Box::new(self::analog_fm_deemph::AnalogFmDeemphConverter {}),
            "fir_filter_xxx" => Box::new(self::fir_filter_xx::FirFilterXxConverter {}),
            "low_pass_filter" => Box::new(self::low_pass_filter::LowPassFilterConverter {}),
            "octave_complex_c" => Box::new(self::octave_complex_c::OctaveComplexConverter {}),
            "blocks_pack_k_bits_bb" => Box::new(self::blocks_pack_k_bits::PackBitsConverter {}),
            "pattern_search" => Box::new(self::pattern_search::PatternSearchConverter {}),
            "rational_resampler_xxx" => {
                Box::new(self::rational_resampler_xxx::RationalResamplerXxConverter {})
            }
            "satellites_kiss_file_source" => {
                Box::new(self::satellites_kiss_file_source::SatellitesKissFileSourceConverter {})
            }
            "satellites_fixedlen_to_pdu" => {
                Box::new(self::satellites_fixedlen_to_pdu::SatellitesFixedlenToPduConverter {})
            }
            "satellites_kiss_file_sink" => {
                Box::new(self::satellites_kiss_file_sink::SatellitesKissFileSinkConverter {})
            }
            "satellites_kiss_server_sink" => {
                Box::new(self::satellites_kiss_server_sink::SatellitesKissServerSinkConverter {})
            }
            "satellites_kiss_client_source" => Box::new(
                self::satellites_kiss_client_source::SatellitesKissClientSourceConverter {},
            ),
            "timing_recovery" => Box::new(self::timing_recovery::TimingRecoveryConverter {}),
            "weaver_usb_cf" | "weaver_lsb_cf" => Box::new(self::weaver_ssb::WeaverSsbConverter {}),
            _ => bail!("Unknown GNU Radio block {blk_type}"),
        };
        Ok(cvter)
    }

    pub fn convert_block(
        &mut self,
        backend: &mut B,
        blk: &BlockInstance,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let cvter = self.specific_converter.get_mut(&blk.id);
        if let Some(cvter) = cvter {
            cvter.convert(blk, backend)
        } else {
            Grc2FutureSdr::<B>::block_converter(blk)?.convert(blk, backend)
        }
    }

    pub fn convert_grc(&mut self, backend: &mut B, grc: Grc) -> Result<()> {
        let mut names_to_adapter =
            BTreeMap::<String, Box<dyn ConnectorAdapter<B::BlockRef>>>::new();

        for blk in &grc.blocks {
            let adapter = self.convert_block(backend, blk)?;
            names_to_adapter.insert(blk.name.clone(), adapter);
        }

        for connection in grc.connections {
            let src_blk_name = connection[0].clone();
            let src_adapter = names_to_adapter
                .get(&src_blk_name)
                .context(format!("unfound source block: {}", src_blk_name))?;
            let src_port = connection[1].clone();
            let (src_blk, src_port) = src_adapter.adapt_output_port(&src_port)?;

            let tgt_blk_name = connection[2].clone();
            let tgt_adapter = names_to_adapter
                .get(&tgt_blk_name)
                .context(format!("unfound target block: {}", tgt_blk_name))?;
            let tgt_port = connection[3].clone();
            let (tgt_blk, tgt_port) = tgt_adapter.adapt_input_port(&tgt_port)?;

            backend.connect(&src_blk, src_port, &tgt_blk, tgt_port)?;
        }
        Ok(())
    }
}

pub fn parameter_as_f32<'i>(
    blk_def: &'i BlockInstance,
    key: &'i str,
    default_value: impl Into<&'i str>,
) -> Result<f32> {
    let expr = blk_def.parameter_or(key, default_value);
    let expr = CommandsParser::parse_expr(expr)?;
    EvalCmd::eval(&expr)
}

pub fn parameter_as_f64<'i>(
    blk_def: &'i BlockInstance,
    key: &'i str,
    default_value: impl Into<&'i str>,
) -> Result<f64> {
    let value = parameter_as_f32(blk_def, key, default_value)?;
    Ok(value as f64)
}

// Re-export convert_grc as a standalone function for now
pub fn convert_grc_runtime(grc: Grc) -> Result<Flowgraph> {
    let mut fg = Flowgraph::new();
    {
        let mut backend = RuntimeBackend { fg: &mut fg };
        let mut converter = Grc2FutureSdr::<RuntimeBackend>::new();
        converter.convert_grc(&mut backend, grc)?;
    }
    Ok(fg)
}

pub fn convert_grc_to_rust(grc: Grc) -> Result<String> {
    let mut backend = crate::grc::backend::CodegenBackend::new();
    let mut converter = Grc2FutureSdr::<crate::grc::backend::CodegenBackend>::new();
    converter.convert_grc(&mut backend, grc)?;

    let tokens = backend.tokens;
    let final_code = quote::quote! {
        extern crate fsdr_cli;
        use futuresdr::runtime::Flowgraph;
        use futuresdr::runtime::Runtime;
        use futuresdr::blocks::{Apply, FirBuilder, Filter, FileSource, Sink, ApplyNM};
        use futuresdr::num_complex::Complex32;
        use fsdr_cli::blocks::kiss_file_sink::KissFileSink;
        use fsdr_cli::blocks::fixedlen_to_pdu::FixedlenToPdu;
        use fsdr_cli::blocks::pattern_search::PatternSearch;

        fn main() {
            #tokens
            Runtime::new().run(fg).unwrap();
        }
    };

    let mut code_str = final_code.to_string();
    // Path translation for generated code
    code_str = code_str.replace("crate :: blocks ::", "fsdr_cli :: blocks ::");

    Ok(code_str)
}
