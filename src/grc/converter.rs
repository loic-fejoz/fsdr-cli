use crate::blocks::AudioSink;
use crate::blocks::DCBlocker;
use crate::blocks::OctaveComplex;
use crate::cmd_grammar::CommandsParser;
use crate::csdr_cmd::eval_cmd::EvalCmd;
use crate::grc::Grc;
use fsdr_blocks::math::FrequencyShifter;
use fsdr_blocks::stdinout::*;
use fsdr_blocks::stream::Deinterleave;
use fsdr_blocks::type_converters::*;
use futuresdr::anyhow::anyhow;
use futuresdr::anyhow::bail;
use futuresdr::anyhow::Context;
use futuresdr::anyhow::Result;
use futuresdr::blocks::ApplyNM;
use futuresdr::blocks::{
    AgcBuilder, Apply, Combine, FileSink, FileSource, FirBuilder, NullSink, Sink, Throttle,
};
use futuresdr::futuredsp::{firdes, windows};
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Block;
use futuresdr::runtime::Flowgraph;
use std::collections::BTreeMap;

use super::converter_helper::*;
use super::BlockInstance;
pub mod blocks_deinterleave;
use self::blocks_deinterleave::DeinterleaveBlockConverter;
pub mod realpart_cf;
use self::realpart_cf::RealpartCfConverter;
pub mod convert;
use self::convert::ConvertBlockConverter;
pub mod blocks_throttle;
use self::blocks_throttle::ThrottleConverter;
pub mod blocks_complex_to_mag;
use self::blocks_complex_to_mag::ComplexToMagConverter;

#[derive(Default, Clone)]
pub struct Grc2FutureSdr;

impl Grc2FutureSdr {
    fn block_converter(blk_def: &BlockInstance) -> Result<Box<dyn BlockConverter>> {
        let blk_type = &(blk_def.id[..]);
        let cvter: Box<dyn BlockConverter> = match blk_type {
            "blocks_deinterleave" => Box::new(DeinterleaveBlockConverter {}),
            "blocks_uchar_to_float"
            | "blocks_char_to_float"
            | "convert_s16_f"
            | "blocks_float_to_uchar"
            | "blocks_float_to_char"
            | "blocks_float_to_short"
            | "convert_ff_c"
            | "blocks_short_to_float" => Box::new(ConvertBlockConverter {}),
            "throttle_ff" => Box::new(ThrottleConverter {}),
            "realpart_cf"
            | "blocks_complex_to_real" => Box::new(RealpartCfConverter {}),
            "blocks_complex_to_mag" => Box::new(ComplexToMagConverter{}),
            _ => bail!("Unknown GNU Radio block {blk_type}"),
        };
        Ok(cvter)
    }

    pub fn convert_block(fg: &mut Flowgraph, blk: &BlockInstance) -> Result<Box<dyn ConnectorAdapter>> {
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

            fg.connect_stream(src_blk, src_port, tgt_blk, tgt_port).context("connecting {connection}")?;
        }
        Ok(fg)
    }

    // fn convert_grc_old(grc: Grc) -> Result<Flowgraph> {
    //     let mut fg = Flowgraph::new();
    //     let names: Vec<String> = grc.blocks.iter().map(|blk| blk.name.clone()).collect();
    //     let mut names_to_id = BTreeMap::<String, usize>::new();
    //     let mut names_to_block_type = BTreeMap::<String, String>::new();
    //     let fsdr_blocks: Vec<Option<usize>> = grc
    //         .blocks
    //         .iter()
    //         .map(|blk_def| {
    //             Self::convert_add_block(&mut fg, blk_def, &grc).expect("Invalid block definition")
    //         })
    //         .collect();
    //     for (name, idx) in names.iter().zip(fsdr_blocks.iter()) {
    //         if let Some(idx) = *idx {
    //             names_to_id.insert(name.clone(), idx);
    //         }
    //     }
    //     let blocks_type: Vec<String> = grc
    //         .blocks
    //         .iter()
    //         .map(|blk_def| blk_def.id.clone())
    //         .collect();
    //     for (name, block_type) in names.iter().zip(blocks_type.iter()) {
    //         names_to_block_type.insert(name.clone(), block_type.clone());
    //     }
    //     for connection in grc.connections {
    //         let src_name = connection[0].clone();
    //         let src_block = names_to_id.get(&src_name);

    //         if let Some(&src_block) = src_block {
    //             let tgt_name = connection[2].clone();
    //             let dst_block = names_to_id.get(&tgt_name);

    //             if let Some(&dst_block) = dst_block {
    //                 let src_port = Self::adapt_src_port(
    //                     names_to_block_type.get(src_name.as_str()),
    //                     &connection[1],
    //                 );
    //                 let dst_port = Self::adapt_dst_port(
    //                     names_to_block_type.get(tgt_name.as_str()),
    //                     &connection[3],
    //                 );
    //                 fg.connect_stream(src_block, src_port, dst_block, dst_port)?;
    //             }
    //         }
    //     }
    //     Ok(fg)
    // }

    // fn adapt_src_port<'a>(block_type: Option<&String>, port_out: &'a str) -> &'a str {
    //     if block_type.is_none() {
    //         return "out";
    //     }

    //     let block_type = block_type.expect("");
    //     // println!("adapt_src_port {block_type} {port_out}");
    //     match &block_type[..] {
    //         "blocks_deinterleave" => match port_out {
    //             "0" => "out0",
    //             "1" => "out1",
    //             _ => "out0",
    //         },
    //         _ => {
    //             if "0" == port_out {
    //                 "out"
    //             } else {
    //                 port_out
    //             }
    //         }
    //     }
    // }

    // fn adapt_dst_port<'a>(block_type: Option<&String>, port_in: &'a str) -> &'a str {
    //     if block_type.is_none() {
    //         return "in";
    //     }
    //     let block_type = block_type.expect("");
    //     // println!("adapt_dst_port {block_type} {port_in}");
    //     match &block_type[..] {
    //         "blocks_float_to_complex" => match port_in {
    //             "0" => "in0",
    //             "1" => "in1",
    //             _ => "in0",
    //         },
    //         _ => {
    //             if "0" == port_in {
    //                 "in"
    //             } else {
    //                 port_in
    //             }
    //         }
    //     }
    // }

    // fn convert_add_block(
    //     fg: &mut Flowgraph,
    //     blk_def: &BlockInstance,
    //     grc: &Grc,
    // ) -> Result<Option<usize>> {
    //     if "disabled".eq(blk_def.states.state.as_str()) {
    //         return Ok(None);
    //     }
    //     let block = Self::convert_block(blk_def, grc)?;
    //     if let Some(mut block) = block {
    //         block.set_instance_name(blk_def.name.clone());
    //         Ok(Some(fg.add_block(block)))
    //     } else {
    //         Ok(None)
    //     }
    // }

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

    // fn convert_block(blk_def: &BlockInstance, _grc: &Grc) -> Result<Option<Block>> {
    //     match &(blk_def.id[..]) {
    //         "realpart_cf" => {
    //             let realpart_blk = Apply::new(|i: &Complex32| -> f32 { i.re });
    //             Ok(Some(realpart_blk))
    //         }
    //         "analog_agc_xx" => {
    //             let reference = blk_def
    //                 .parameters
    //                 .get("reference")
    //                 .expect("reference must be defined")
    //                 .parse::<f32>()?;
    //             let max_gain = blk_def
    //                 .parameters
    //                 .get("max_gain")
    //                 .expect("max_gain must be defined")
    //                 .parse::<f32>()?;
    //             let rate = blk_def
    //                 .parameters
    //                 .get("rate")
    //                 .expect("rate must be defined")
    //                 .parse::<f32>()?;
    //             let item_type = blk_def
    //                 .parameters
    //                 .get("type")
    //                 .expect("type must be defined");

    //             let blk = match &(item_type[..]) {
    //                 "float" => AgcBuilder::<f32>::new()
    //                     .squelch(0.0)
    //                     .reference_power(reference)
    //                     .max_gain(max_gain)
    //                     .adjustment_rate(rate)
    //                     .build(),
    //                 _ => todo!("Unhandled analog_agc_xx Type {item_type}"),
    //             };
    //             Ok(Some(blk))
    //         }
    //         "analog_rail_ff" => {
    //             let low_threshold = Grc2FutureSdr::parameter_as_f32(blk_def, "lo", "-1.0")?;
    //             let max_threshold = Grc2FutureSdr::parameter_as_f32(blk_def, "hi", "1.0")?;
    //             let rail_blk =
    //                 Apply::new(move |i: &f32| -> f32 { i.max(low_threshold).min(max_threshold) });
    //             Ok(Some(rail_blk))
    //         }
    //         "blocks_complex_to_real" => {
    //             // TODO: should do an analysis on how many outputs are really used,
    //             // to know exactly what to generate
    //             let realpart_blk = Apply::new(|i: &Complex32| -> f32 { i.re });
    //             Ok(Some(realpart_blk))
    //         }
    //         "blocks_complex_to_mag" => {
    //             let blocks_complex_to_mag = Apply::new(|i: &Complex32| -> f32 { i.norm() });
    //             Ok(Some(blocks_complex_to_mag))
    //         }
    //         "clipdetect_ff" => {
    //             let blk = Apply::new(|i: &f32| -> f32 {
    //                 if *i < 1.0 {
    //                     eprintln!("csdr clipdetect_ff: Signal value below -1.0!")
    //                 } else if *i > 1.0 {
    //                     eprintln!("csdr clipdetect_ff: Signal value above -1.0!")
    //                 };
    //                 *i
    //             });
    //             Ok(Some(blk))
    //         }
    //         "convert_u8_f" => {
    //             let blk = TypeConvertersBuilder::scale_convert::<u8, f32>().build();
    //             Ok(Some(blk))
    //         }
    //         "convert_s8_f" => {
    //             let blk = TypeConvertersBuilder::scale_convert::<i8, f32>().build();
    //             Ok(Some(blk))
    //         }
    //         "convert_s16_f" => {
    //             let blk = TypeConvertersBuilder::scale_convert::<i16, f32>().build();
    //             Ok(Some(blk))
    //         }
    //         "convert_f_u8" => {
    //             let blk = TypeConvertersBuilder::lossy_scale_convert_f32_u8().build();
    //             Ok(Some(blk))
    //         }
    //         "convert_f_s8" => {
    //             let blk = TypeConvertersBuilder::lossy_scale_convert_f32_i8().build();
    //             Ok(Some(blk))
    //         }
    //         "convert_f_s16" => {
    //             let blk = TypeConvertersBuilder::lossy_scale_convert_f32_i16().build();
    //             Ok(Some(blk))
    //         }
    //         "dump_u8" => {
    //             let blk = Sink::new(|x: &u8| print!("{:02x} ", *x));
    //             Ok(Some(blk))
    //         }
    //         "dump_f" => {
    //             let blk = Sink::new(|x: &f32| print!("{:e} ", *x));
    //             Ok(Some(blk))
    //         }
    //         "blocks_file_source" => {
    //             let filename = blk_def
    //                 .parameters
    //                 .get("file")
    //                 .expect("filename must be defined");
    //             let item_type = blk_def
    //                 .parameters
    //                 .get("type")
    //                 .expect("item type must be defined");
    //             let _repeat = blk_def
    //                 .parameters
    //                 .get("repeat")
    //                 .unwrap_or(&"False".to_string());
    //             let filename = if "-" == filename {
    //                 "/proc/self/fd/0"
    //             } else {
    //                 filename
    //             };
    //             let blk = match &(item_type[..]) {
    //                 "u8" => FileSource::<u8>::new(filename, false),
    //                 "byte" => FileSource::<i8>::new(filename, false),
    //                 "f32" => FileSource::<f32>::new(filename, false),
    //                 "float" => FileSource::<f32>::new(filename, false),
    //                 "c32" => FileSource::<Complex32>::new(filename, false),
    //                 "complex" => FileSource::<Complex32>::new(filename, false),
    //                 _ => todo!("Unhandled FileSource Type {item_type}"),
    //             };
    //             Ok(Some(blk))
    //         }
    //         "blocks_file_sink" => {
    //             let filename = blk_def
    //                 .parameters
    //                 .get("file")
    //                 .expect("filename must be defined");
    //             let item_type = blk_def
    //                 .parameters
    //                 .get("type")
    //                 .expect("item type must be defined");
    //             if "-" == filename {
    //                 let blk = match &(item_type[..]) {
    //                     "u8" => StdInOutBuilder::<u8>::stdout().as_ne().build(),
    //                     "i16" => StdInOutBuilder::<i16>::stdout().as_ne().build(),
    //                     "short" => StdInOutBuilder::<i16>::stdout().as_ne().build(),
    //                     "f32" => StdInOutBuilder::<f32>::stdout().as_ne().build(),
    //                     "float" => StdInOutBuilder::<f32>::stdout().as_ne().build(),
    //                     "c32" => StdInOutBuilder::<Complex32>::stdout().as_ne().build(),
    //                     "complex" => StdInOutBuilder::<Complex32>::stdout().as_ne().build(),
    //                     _ => todo!("Unhandled FileSink Type {item_type}"),
    //                 };
    //                 Ok(Some(blk))
    //             } else {
    //                 let blk = match &(item_type[..]) {
    //                     "u8" => FileSink::<u8>::new(filename),
    //                     "i16" => FileSink::<i16>::new(filename),
    //                     "short" => FileSink::<i16>::new(filename),
    //                     "f32" => FileSink::<f32>::new(filename),
    //                     "float" => FileSink::<f32>::new(filename),
    //                     "c32" => FileSink::<Complex32>::new(filename),
    //                     "complex" => FileSink::<Complex32>::new(filename),
    //                     _ => todo!("Unhandled FileSink Type {item_type}"),
    //                 };
    //                 Ok(Some(blk))
    //             }
    //         }
    //         "dc_blocker_xx" => {
    //             let min_bufsize = "32".to_string();
    //             let min_bufsize = blk_def.parameters.get("length").or(Some(&min_bufsize));
    //             let min_bufsize = min_bufsize
    //                 .expect("")
    //                 .parse::<usize>()
    //                 .expect("invalid length");
    //             let dc_blocker = DCBlocker::<f32>::build(min_bufsize);
    //             Ok(Some(dc_blocker))
    //         }
    //         "analog_quadrature_demod_cf" => {
    //             let default_gain = "1.0".to_string();
    //             let gain = blk_def.parameters.get("gain").or(Some(&default_gain));
    //             let gain = gain.expect("").parse::<f32>().expect("invalid gain");
    //             // println!("gain: {gain}");
    //             let mut last = Complex32::new(0.0, 0.0); // store sample x[n-1]
    //             let demod = Apply::new(move |v: &Complex32| -> f32 {
    //                 let arg = (v * last.conj()).arg(); // Obtain phase of x[n] * conj(x[n-1])
    //                 last = *v;
    //                 arg * gain
    //             });
    //             Ok(Some(demod))
    //         }
    //         "analog_fm_deemph" => {
    //             let sample_rate = blk_def
    //                 .parameters
    //                 .get("samp_rate")
    //                 .expect("samp_rate must be defined");
    //             let sample_rate = sample_rate.parse::<f32>()?;
    //             let tau = blk_def
    //                 .parameters
    //                 .get("tau")
    //                 .expect("tau must be defined")
    //                 .parse::<f32>()?;
    //             let dt = 1.0 / sample_rate;
    //             let alpha = dt / (tau + dt);
    //             let mut last = 0.0; // store sample x[n-1]
    //             let blk = Apply::new(move |v: &f32| -> f32 {
    //                 let r = alpha * v + (1.0 - alpha) * last; //this is the simplest IIR LPF
    //                 last = r;
    //                 r
    //             });
    //             Ok(Some(blk))
    //         }
    //         "rational_resampler_xxx" => {
    //             let interp = blk_def
    //                 .parameters
    //                 .get("interp")
    //                 .expect("interp must be defined")
    //                 .parse::<usize>()?;
    //             let decim = blk_def
    //                 .parameters
    //                 .get("decim")
    //                 .expect("decim must be defined")
    //                 .parse::<usize>()?;
    //             let blk = FirBuilder::new_resampling::<f32, f32>(interp, decim);
    //             Ok(Some(blk))
    //         }
    //         "blocks_throttle" => {
    //             let rate = blk_def
    //                 .parameters
    //                 .get("samples_per_second")
    //                 .expect("samples_per_second must be defined for blocks_throttle")
    //                 .parse::<f64>()?;
    //             let item_type = blk_def
    //                 .parameters
    //                 .get("type")
    //                 .expect("item type must be defined");
    //             let blk = match &(item_type[..]) {
    //                 "char" => Throttle::<u8>::new(rate),
    //                 "short" => Throttle::<i16>::new(rate),
    //                 "float" => Throttle::<f32>::new(rate),
    //                 "complex" => Throttle::<Complex32>::new(rate),
    //                 _ => todo!("Unhandled blocks_throttle Type {item_type}"),
    //             };
    //             Ok(Some(blk))
    //         }
    //         "variable" => Ok(None),
    //         "convert_ff_c" => {
    //             let blk = ApplyNM::<_, _, _, 2, 1>::new(move |v: &[f32], d: &mut [Complex32]| {
    //                 d[0] = Complex32::new(v[0], v[1])
    //             });
    //             Ok(Some(blk))
    //         }
    //         "audio_sink" => {
    //             let rate = blk_def
    //                 .parameters
    //                 .get("samp_rate")
    //                 .expect("rate must be defined")
    //                 .parse::<u32>()?;
    //             let blk = AudioSink::new(rate, 1);
    //             Ok(Some(blk))
    //         }
    //         "blocks_add_const_vxx" => {
    //             let item_type = blk_def
    //                 .parameters
    //                 .get("type")
    //                 .expect("item type must be defined");
    //             let blk = match &(item_type[..]) {
    //                 "u8" => {
    //                     let constant = blk_def
    //                         .parameters
    //                         .get("const")
    //                         .expect("constant must be defined")
    //                         .parse::<u8>()?;
    //                     Apply::new(move |v: &u8| -> u8 { v + constant })
    //                 }
    //                 "float" => {
    //                     let constant = blk_def
    //                         .parameters
    //                         .get("const")
    //                         .expect("constant must be defined")
    //                         .parse::<f32>()?;
    //                     Apply::new(move |v: &f32| -> f32 { v + constant })
    //                 }
    //                 // "i16" => FileSink::<i16>::new(filename),
    //                 // "f32" => FileSink::<f32>::new(filename),
    //                 // "float" => FileSink::<f32>::new(filename),
    //                 // "c32" => FileSink::<Complex32>::new(filename),
    //                 // "complex" => FileSink::<Complex32>::new(filename),
    //                 _ => todo!("Unhandled blocks_add_const_vxx Type {item_type}"),
    //             };
    //             Ok(Some(blk))
    //         }
    //         "blocks_char_to_float" => {
    //             let scale = blk_def
    //                 .parameters
    //                 .get("scale")
    //                 .expect("scale must be defined")
    //                 .parse::<f32>()?;
    //             // println!("blocks_char_to_float scale: {scale}");
    //             let blk = Apply::new(move |v: &i8| -> f32 { (*v) as f32 * scale });
    //             Ok(Some(blk))
    //         }
    //         "blocks_float_to_short" => {
    //             let scale = blk_def
    //                 .parameters
    //                 .get("scale")
    //                 .expect("scale must be defined")
    //                 .parse::<f32>()?;
    //             // println!("blocks_float_to_short scale {scale}");
    //             let blk = Apply::new(move |v: &f32| -> i16 { (*v * scale) as i16 });
    //             Ok(Some(blk))
    //         }
    //         "blocks_deinterleave" => {
    //             let item_type = blk_def
    //                 .parameters
    //                 .get("type")
    //                 .expect("item type must be defined");
    //             let blk = match &(item_type[..]) {
    //                 "char" => Deinterleave::<u8>::new(),
    //                 "short" => Deinterleave::<i16>::new(),
    //                 "float" => Deinterleave::<f32>::new(),
    //                 "complex" => Deinterleave::<Complex32>::new(),
    //                 _ => todo!("Unhandled blocks_deinterleave Type {item_type}"),
    //             };
    //             Ok(Some(blk))
    //         }
    //         "blocks_float_to_complex" => {
    //             let blk =
    //                 Combine::new(|v1: &f32, v2: &f32| -> Complex32 { Complex32::new(*v1, *v2) });
    //             Ok(Some(blk))
    //         }
    //         "octave_complex_c" => {
    //             let samples_to_plot = blk_def
    //                 .parameters
    //                 .get("samples_to_plot")
    //                 .expect("samples_to_plot must be defined")
    //                 .parse::<usize>()?;
    //             let out_of_n_samples = blk_def
    //                 .parameters
    //                 .get("out_of_n_samples")
    //                 .expect("out_of_n_samples must be defined")
    //                 .parse::<usize>()?;
    //             if out_of_n_samples < samples_to_plot {
    //                 return Err(anyhow!("out_of_n_samples should be < samples_to_plot"));
    //             }
    //             let blk = OctaveComplex::build(samples_to_plot, out_of_n_samples);
    //             Ok(Some(blk))
    //         }
    //         "blocks_freqshift_cc" => {
    //             let sample_rate = blk_def
    //                 .parameters
    //                 .get("sample_rate")
    //                 .expect("sample_rate must be defined")
    //                 .parse::<f32>()?;
    //             let freq = blk_def
    //                 .parameters
    //                 .get("freq")
    //                 .expect("freq must be defined")
    //                 .parse::<f32>()?;
    //             let blk = FrequencyShifter::<Complex32>::new(freq, sample_rate);
    //             Ok(Some(blk))
    //         }
    //         "blocks_null_sink" => {
    //             let item_type = blk_def
    //                 .parameters
    //                 .get("type")
    //                 .expect("item type must be defined");
    //             let blk = match &(item_type[..]) {
    //                 "char" => NullSink::<u8>::new(),
    //                 "short" => NullSink::<i16>::new(),
    //                 "float" => NullSink::<f32>::new(),
    //                 "complex" => NullSink::<Complex32>::new(),
    //                 _ => todo!("Unhandled blocks_null_sink Type {item_type}"),
    //             };
    //             Ok(Some(blk))
    //         }
    //         "deemphasis_nfm_ff" => {
    //             let sample_rate = blk_def
    //                 .parameters
    //                 .get("sample_rate")
    //                 .expect("sample_rate must be defined")
    //                 .parse::<usize>()?;
    //             let blk = match sample_rate {
    //                 48000 => {
    //                     #[rustfmt::skip]
    //                     let taps = [0.00172568f32, 0.00179665, 0.00191952, 0.00205318, 0.00215178, 0.00217534, 0.00209924, 0.00192026, 0.00165789, 0.0013502, 0.00104545, 0.000790927, 0.000621911, 0.000553077, 0.000574554, 0.000653624, 0.000741816, 0.000785877, 0.000740151, 0.000577506, 0.000296217, -7.89273e-05, -0.0005017, -0.000914683, -0.00126243, -0.00150456, -0.00162564, -0.0016396, -0.00158725, -0.00152751, -0.00152401, -0.00163025, -0.00187658, -0.00226223, -0.00275443, -0.003295, -0.0038132, -0.00424193, -0.00453375, -0.00467274, -0.00467943, -0.00460728, -0.00453119, -0.00453056, -0.00467051, -0.00498574, -0.00547096, -0.00608027, -0.00673627, -0.00734698, -0.00782705, -0.00811841, -0.00820539, -0.00812057, -0.00793936, -0.00776415, -0.00770111, -0.00783479, -0.00820643, -0.00880131, -0.00954878, -0.0103356, -0.0110303, -0.011514, -0.0117094, -0.0116029, -0.0112526, -0.0107795, -0.010343, -0.0101053, -0.0101917, -0.0106561, -0.0114608, -0.0124761, -0.0135018, -0.0143081, -0.0146885, -0.0145126, -0.0137683, -0.0125796, -0.0111959, -0.00994914, -0.00918404, -0.00917447, -0.0100402, -0.0116822, -0.0137533, -0.0156723, -0.0166881, -0.0159848, -0.0128153, -0.00664117, 0.00274383, 0.0151313, 0.0298729, 0.0459219, 0.0619393, 0.076451, 0.0880348, 0.0955087, 0.098091, 0.0955087, 0.0880348, 0.076451, 0.0619393, 0.0459219, 0.0298729, 0.0151313, 0.00274383, -0.00664117, -0.0128153, -0.0159848, -0.0166881, -0.0156723, -0.0137533, -0.0116822, -0.0100402, -0.00917447, -0.00918404, -0.00994914, -0.0111959, -0.0125796, -0.0137683, -0.0145126, -0.0146885, -0.0143081, -0.0135018, -0.0124761, -0.0114608, -0.0106561, -0.0101917, -0.0101053, -0.010343, -0.0107795, -0.0112526, -0.0116029, -0.0117094, -0.011514, -0.0110303, -0.0103356, -0.00954878, -0.00880131, -0.00820643, -0.00783479, -0.00770111, -0.00776415, -0.00793936, -0.00812057, -0.00820539, -0.00811841, -0.00782705, -0.00734698, -0.00673627, -0.00608027, -0.00547096, -0.00498574, -0.00467051, -0.00453056, -0.00453119, -0.00460728, -0.00467943, -0.00467274, -0.00453375, -0.00424193, -0.0038132, -0.003295, -0.00275443, -0.00226223, -0.00187658, -0.00163025, -0.00152401, -0.00152751, -0.00158725, -0.0016396, -0.00162564, -0.00150456, -0.00126243, -0.000914683, -0.0005017, -7.89273e-05, 0.000296217, 0.000577506, 0.000740151, 0.000785877, 0.000741816, 0.000653624, 0.000574554, 0.000553077, 0.000621911, 0.000790927, 0.00104545, 0.0013502, 0.00165789, 0.00192026, 0.00209924, 0.00217534, 0.00215178, 0.00205318, 0.00191952, 0.00179665, 0.00172568];
    //                     FirBuilder::new::<f32, f32, f32, _>(taps)
    //                 }
    //                 8000 => {
    //                     #[rustfmt::skip]
    //                     let taps = [1.43777e+11f32, 1.45874e+11, -4.67746e+11, 9.98433e+10, -1.47835e+12, -3.78799e+11, -2.61333e+12, -1.07042e+12, -3.41242e+12, -1.57042e+12, -3.34195e+12, -1.4091e+12, -1.96864e+12, -2.26212e+11, 8.48259e+11, 2.04875e+12, 4.80451e+12, 5.06875e+12, 9.09434e+12, 8.04571e+12, 1.24874e+13, 9.85837e+12, 1.35433e+13, 9.28407e+12, 1.09287e+13, 5.30975e+12, 3.76762e+12, -2.54809e+12, -8.06152e+12, -1.39895e+13, -2.37664e+13, -2.77865e+13, -4.16745e+13, -4.16797e+13, -5.94708e+13, -5.17628e+13, -7.46014e+13, -4.66449e+13, -8.47575e+13, 1.51722e+14, 4.98196e+14, 1.51722e+14, -8.47575e+13, -4.66449e+13, -7.46014e+13, -5.17628e+13, -5.94708e+13, -4.16797e+13, -4.16745e+13, -2.77865e+13, -2.37664e+13, -1.39895e+13, -8.06152e+12, -2.54809e+12, 3.76762e+12, 5.30975e+12, 1.09287e+13, 9.28407e+12, 1.35433e+13, 9.85837e+12, 1.24874e+13, 8.04571e+12, 9.09434e+12, 5.06875e+12, 4.80451e+12, 2.04875e+12, 8.48259e+11, -2.26212e+11, -1.96864e+12, -1.4091e+12, -3.34195e+12, -1.57042e+12, -3.41242e+12, -1.07042e+12, -2.61333e+12, -3.78799e+11, -1.47835e+12, 9.98433e+10, -4.67746e+11, 1.45874e+11, 1.43777e+11];
    //                     FirBuilder::new::<f32, f32, f32, _>(taps)
    //                 }
    //                 44100 => {
    //                     #[rustfmt::skip]
    //                     let taps = [0.0025158f32, 0.00308564, 0.00365507, 0.00413598, 0.00446279, 0.00461162, 0.00460866, 0.00452474, 0.00445739, 0.00450444, 0.00473648, 0.0051757, 0.0057872, 0.00648603, 0.00715856, 0.00769296, 0.00801081, 0.00809096, 0.00797853, 0.00777577, 0.00761627, 0.00762871, 0.00789987, 0.00844699, 0.00920814, 0.0100543, 0.0108212, 0.0113537, 0.011551, 0.0113994, 0.0109834, 0.0104698, 0.0100665, 0.00996618, 0.0102884, 0.0110369, 0.0120856, 0.0131998, 0.0140907, 0.0144924, 0.0142417, 0.0133401, 0.0119771, 0.0105043, 0.00935909, 0.00895022, 0.00952985, 0.0110812, 0.0132522, 0.015359, 0.0164664, 0.0155409, 0.0116496, 0.00416925, -0.00703664, -0.021514, -0.0382135, -0.0555955, -0.0718318, -0.0850729, -0.0937334, -0.0967458, -0.0937334, -0.0850729, -0.0718318, -0.0555955, -0.0382135, -0.021514, -0.00703664, 0.00416925, 0.0116496, 0.0155409, 0.0164664, 0.015359, 0.0132522, 0.0110812, 0.00952985, 0.00895022, 0.00935909, 0.0105043, 0.0119771, 0.0133401, 0.0142417, 0.0144924, 0.0140907, 0.0131998, 0.0120856, 0.0110369, 0.0102884, 0.00996618, 0.0100665, 0.0104698, 0.0109834, 0.0113994, 0.011551, 0.0113537, 0.0108212, 0.0100543, 0.00920814, 0.00844699, 0.00789987, 0.00762871, 0.00761627, 0.00777577, 0.00797853, 0.00809096, 0.00801081, 0.00769296, 0.00715856, 0.00648603, 0.0057872, 0.0051757, 0.00473648, 0.00450444, 0.00445739, 0.00452474, 0.00460866, 0.00461162, 0.00446279, 0.00413598, 0.00365507, 0.00308564, 0.0025158];
    //                     FirBuilder::new::<f32, f32, f32, _>(taps)
    //                 }
    //                 11025 => {
    //                     #[rustfmt::skip]
    //                     let taps = [0.00113162f32, 0.000911207, 0.00173815, -0.000341385, -0.000849373, -0.00033066, -0.00290692, -0.00357326, -0.0031917, -0.00607078, -0.00659201, -0.00601551, -0.00886603, -0.00880243, -0.00759841, -0.0100344, -0.0088993, -0.00664423, -0.00835258, -0.00572919, -0.00214109, -0.00302443, 0.00132902, 0.00627003, 0.00596494, 0.0120731, 0.0180437, 0.0176243, 0.0253776, 0.0316572, 0.0298485, 0.0393389, 0.0446019, 0.0389943, 0.0516463, 0.0521951, 0.0350192, 0.0600945, 0.0163128, -0.217526, -0.378533, -0.217526, 0.0163128, 0.0600945, 0.0350192, 0.0521951, 0.0516463, 0.0389943, 0.0446019, 0.0393389, 0.0298485, 0.0316572, 0.0253776, 0.0176243, 0.0180437, 0.0120731, 0.00596494, 0.00627003, 0.00132902, -0.00302443, -0.00214109, -0.00572919, -0.00835258, -0.00664423, -0.0088993, -0.0100344, -0.00759841, -0.00880243, -0.00886603, -0.00601551, -0.00659201, -0.00607078, -0.0031917, -0.00357326, -0.00290692, -0.00033066, -0.000849373, -0.000341385, 0.00173815, 0.000911207, 0.00113162];
    //                     FirBuilder::new::<f32, f32, f32, _>(taps)
    //                 }
    //                 _ => todo!("Unhandled sample rate for deemphasis_nfm_ff. Must be one of 8000, 11025, 44100, 48000."),
    //             };
    //             Ok(Some(blk))
    //         }
    //         "fir_filter_xxx" => {
    //             let taps = blk_def
    //                 .parameters
    //                 .get("taps")
    //                 .expect("taps must be defined");
    //             let decimation = blk_def
    //                 .parameters
    //                 .get("decim")
    //                 .expect("decim must be defined")
    //                 .parse::<usize>()?;
    //             let item_type = blk_def
    //                 .parameters
    //                 .get("type")
    //                 .expect("type must be defined");
    //             let taps: Vec<f32> = if taps.is_empty() {
    //                 // This block definition was from csdr
    //                 let transition_bw = blk_def
    //                     .parameters
    //                     .get("transition_bw")
    //                     .expect("transition_bw must be defined")
    //                     .parse::<f64>()?;
    //                 let window = blk_def
    //                     .parameters
    //                     .get("window")
    //                     .expect("window must be defined");
    //                 let taps_length: usize = (4.0 / transition_bw) as usize;
    //                 let taps_length = taps_length + if taps_length % 2 == 0 { 1 } else { 0 };
    //                 assert!(taps_length % 2 == 1); //number of symmetric FIR filter taps should be odd

    //                 // Building firdes_lowpass_f(taps,taps_length,0.5/(float)factor,window);
    //                 let rect_win = match &window[..] {
    //                     "HAMMING" => windows::hamming(taps_length, false),
    //                     "BLACKMAN" => windows::blackman(taps_length, false),
    //                     //"KAISER" => windows::kaiser(taps_length, beta),
    //                     "HANN" => windows::hann(taps_length, false),
    //                     //"GAUSSIAN" => windows::gaussian(taps_length, alpha),
    //                     _ => todo!("Unknown fir_filter_xx window: {window}"),
    //                 };
    //                 let taps = firdes::lowpass::<f32>(transition_bw, rect_win.as_slice());
    //                 taps
    //             } else {
    //                 todo!("Unhandled fir_filter_xx taps definition")
    //             };
    //             let blk = match &(item_type[..]) {
    //                 "ccc" => FirBuilder::new_resampling_with_taps::<Complex32, Complex32, f32, _>(
    //                     1, decimation, taps,
    //                 ),
    //                 _ => todo!("Unhandled fir_filter_xx Type {item_type}"),
    //             };
    //             Ok(Some(blk))
    //         }
    //         "low_pass_filter" => {
    //             let beta = blk_def
    //                 .parameters
    //                 .get("beta")
    //                 .expect("beta must be defined")
    //                 .parse::<f64>();
    //             let cutoff_freq = blk_def
    //                 .parameters
    //                 .get("cutoff_freq")
    //                 .expect("cutoff_freq must be defined")
    //                 .parse::<f64>()?; // Cutoff frequency in Hz
    //             let decimation = blk_def
    //                 .parameters
    //                 .get("decim")
    //                 .expect("decim must be defined")
    //                 .parse::<usize>()?; // Decimation rate of filter
    //             let _gain = blk_def
    //                 .parameters
    //                 .get("gain")
    //                 .expect("gain must be defined")
    //                 .parse::<f32>()?;
    //             let interp = blk_def
    //                 .parameters
    //                 .get("interp")
    //                 .expect("interp must be defined")
    //                 .parse::<usize>()?;
    //             let sample_rate = blk_def
    //                 .parameters
    //                 .get("samp_rate")
    //                 .expect("samp_rate must be defined")
    //                 .parse::<f64>()?;
    //             let item_type = blk_def
    //                 .parameters
    //                 .get("type")
    //                 .expect("type must be defined");
    //             let _width = blk_def
    //                 .parameters
    //                 .get("width")
    //                 .expect("width must be defined")
    //                 .parse::<f64>()?; // Transition width between stop-band and pass-band in Hz
    //             let window = blk_def.parameters.get("win").expect("win must be defined");
    //             let transition_bw = cutoff_freq / sample_rate;
    //             let taps_length: usize = (4.0 / transition_bw) as usize;
    //             let taps_length = taps_length + if taps_length % 2 == 0 { 1 } else { 0 };
    //             assert!(taps_length % 2 == 1); //number of symmetric FIR filter taps should be odd
    //             let alpha = beta.clone();
    //             let rect_win = match &window[..] {
    //                 "window.WIN_HAMMING" => windows::hamming(taps_length, false),
    //                 "window.WIN_BLACKMAN" => windows::blackman(taps_length, false),
    //                 "window.WIN_KAISER" => {
    //                     windows::kaiser(taps_length, beta.expect("beta is mandatory for Kaiser"))
    //                 }
    //                 "window.WIN_HANN" => windows::hann(taps_length, false),
    //                 "window.WIN_GAUSSIAN" => windows::gaussian(
    //                     taps_length,
    //                     alpha.expect("alpha is mandatory for Gaussian"),
    //                 ),
    //                 _ => todo!("Unknown low_pass_filter window: {window}"),
    //             };
    //             let taps = firdes::lowpass::<f32>(transition_bw, rect_win.as_slice());
    //             let blk = match &(item_type[..]) {
    //                 "fir_filter_ccf" => {
    //                     FirBuilder::new_resampling_with_taps::<Complex32, Complex32, f32, _>(
    //                         interp, decimation, taps,
    //                     )
    //                 }
    //                 _ => todo!("Unhandled low_pass_filter Type {item_type}"),
    //             };
    //             Ok(Some(blk))
    //         }
    //         _ => {
    //             let unknow_block_type = blk_def.id.clone();
    //             todo!("unknow_block_type: {unknow_block_type}")
    //         }
    //     }
    // }
}
