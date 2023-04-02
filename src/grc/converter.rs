use crate::blocks::AudioSink;
use crate::blocks::DCBlocker;
use crate::blocks::OctaveComplex;
use crate::grc::Grc;
use fsdr_blocks::math::FrequencyShifter;
use fsdr_blocks::stdinout::*;
use fsdr_blocks::stream::Deinterleave;
use fsdr_blocks::type_converters::*;
use futuresdr::anyhow::anyhow;
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

use super::BlockInstance;

#[derive(Default, Clone)]
pub struct Grc2FutureSdr;

impl Grc2FutureSdr {
    pub fn convert_grc(grc: Grc) -> Result<Flowgraph> {
        let mut fg = Flowgraph::new();
        let names: Vec<String> = grc.blocks.iter().map(|blk| blk.name.clone()).collect();
        let mut names_to_id = BTreeMap::<String, usize>::new();
        let mut names_to_block_type = BTreeMap::<String, String>::new();
        let fsdr_blocks: Vec<Option<usize>> = grc
            .blocks
            .iter()
            .map(|blk_def| {
                Self::convert_add_block(&mut fg, blk_def, &grc).expect("Invalid block definition")
            })
            .collect();
        for (name, idx) in names.iter().zip(fsdr_blocks.iter()) {
            if let Some(idx) = *idx {
                names_to_id.insert(name.clone(), idx);
            }
        }
        let blocks_type: Vec<String> = grc
            .blocks
            .iter()
            .map(|blk_def| blk_def.id.clone())
            .collect();
        for (name, block_type) in names.iter().zip(blocks_type.iter()) {
            names_to_block_type.insert(name.clone(), block_type.clone());
        }
        for connection in grc.connections {
            let src_name = connection[0].clone();
            let src_block = names_to_id.get(&src_name);

            if let Some(&src_block) = src_block {
                let tgt_name = connection[2].clone();
                let dst_block = names_to_id.get(&tgt_name);

                if let Some(&dst_block) = dst_block {
                    let src_port = Self::adapt_src_port(
                        names_to_block_type.get(src_name.as_str()),
                        &connection[1],
                    );
                    let dst_port = Self::adapt_dst_port(
                        names_to_block_type.get(tgt_name.as_str()),
                        &connection[3],
                    );
                    fg.connect_stream(src_block, src_port, dst_block, dst_port)?;
                }
            }
        }
        Ok(fg)
    }

    fn adapt_src_port<'a>(block_type: Option<&String>, port_out: &'a str) -> &'a str {
        if block_type.is_none() {
            return "out";
        }

        let block_type = block_type.expect("");
        // println!("adapt_src_port {block_type} {port_out}");
        match &block_type[..] {
            "blocks_deinterleave" => match port_out {
                "0" => "out0",
                "1" => "out1",
                _ => "out0",
            },
            _ => {
                if "0" == port_out {
                    "out"
                } else {
                    port_out
                }
            }
        }
    }

    fn adapt_dst_port<'a>(block_type: Option<&String>, port_in: &'a str) -> &'a str {
        if block_type.is_none() {
            return "in";
        }
        let block_type = block_type.expect("");
        // println!("adapt_dst_port {block_type} {port_in}");
        match &block_type[..] {
            "blocks_float_to_complex" => match port_in {
                "0" => "in0",
                "1" => "in1",
                _ => "in0",
            },
            _ => {
                if "0" == port_in {
                    "in"
                } else {
                    port_in
                }
            }
        }
    }

    pub fn convert_add_block(
        fg: &mut Flowgraph,
        blk_def: &BlockInstance,
        grc: &Grc,
    ) -> Result<Option<usize>> {
        if "disabled".eq(blk_def.states.state.as_str()) {
            return Ok(None);
        }
        let block = Self::convert_block(blk_def, grc)?;
        if let Some(mut block) = block {
            block.set_instance_name(blk_def.name.clone());
            Ok(Some(fg.add_block(block)))
        } else {
            Ok(None)
        }
    }

    fn convert_block(blk_def: &BlockInstance, _grc: &Grc) -> Result<Option<Block>> {
        match &(blk_def.id[..]) {
            "realpart_cf" => {
                let realpart_blk = Apply::new(|i: &Complex32| -> f32 { i.re });
                Ok(Some(realpart_blk))
            }
            "analog_agc_xx" => {
                let reference = blk_def
                    .parameters
                    .get("reference")
                    .expect("reference must be defined")
                    .parse::<f32>()?;
                let max_gain = blk_def
                    .parameters
                    .get("max_gain")
                    .expect("max_gain must be defined")
                    .parse::<f32>()?;
                let rate = blk_def
                    .parameters
                    .get("rate")
                    .expect("rate must be defined")
                    .parse::<f32>()?;
                let item_type = blk_def
                    .parameters
                    .get("type")
                    .expect("type must be defined");

                let blk = match &(item_type[..]) {
                    "float" => AgcBuilder::<f32>::new()
                        .squelch(0.0)
                        .reference_power(reference)
                        .max_gain(max_gain)
                        .adjustment_rate(rate)
                        .build(),
                    _ => todo!("Unhandled analog_agc_xx Type {item_type}"),
                };
                Ok(Some(blk))
            }
            "analog_rail_ff" => {
                let default_low_threshold = "-1.0".to_string();
                let low_threshold = blk_def
                    .parameters
                    .get("lo")
                    .or(Some(&default_low_threshold));
                let low_threshold = low_threshold
                    .expect("")
                    .parse::<f32>()
                    .expect("invalid low_threshold");
                let default_max_threshold = "1.0".to_string();
                let max_threshold = blk_def
                    .parameters
                    .get("hi")
                    .or(Some(&default_max_threshold));
                let max_threshold = max_threshold
                    .expect("")
                    .parse::<f32>()
                    .expect("invalid max_threshold");

                let rail_blk =
                    Apply::new(move |i: &f32| -> f32 { i.max(low_threshold).min(max_threshold) });
                Ok(Some(rail_blk))
            }
            "blocks_complex_to_real" => {
                // TODO: should do an analysis on how many outputs are really used,
                // to know exactly what to generate
                let realpart_blk = Apply::new(|i: &Complex32| -> f32 { i.re });
                Ok(Some(realpart_blk))
            }
            "blocks_complex_to_mag" => {
                let blocks_complex_to_mag = Apply::new(|i: &Complex32| -> f32 { i.norm() });
                Ok(Some(blocks_complex_to_mag))
            }
            "clipdetect_ff" => {
                let blk = Apply::new(|i: &f32| -> f32 {
                    if *i < 1.0 {
                        eprintln!("csdr clipdetect_ff: Signal value below -1.0!")
                    } else if *i > 1.0 {
                        eprintln!("csdr clipdetect_ff: Signal value above -1.0!")
                    };
                    *i
                });
                Ok(Some(blk))
            }
            "convert_u8_f" => {
                let blk = TypeConvertersBuilder::scale_convert::<u8, f32>().build();
                Ok(Some(blk))
            }
            "convert_s8_f" => {
                let blk = TypeConvertersBuilder::scale_convert::<i8, f32>().build();
                Ok(Some(blk))
            }
            "convert_s16_f" => {
                let blk = TypeConvertersBuilder::scale_convert::<i16, f32>().build();
                Ok(Some(blk))
            }
            "convert_f_u8" => {
                let blk = TypeConvertersBuilder::lossy_scale_convert_f32_u8().build();
                Ok(Some(blk))
            }
            "convert_f_s8" => {
                let blk = TypeConvertersBuilder::lossy_scale_convert_f32_i8().build();
                Ok(Some(blk))
            }
            "convert_f_s16" => {
                let blk = TypeConvertersBuilder::lossy_scale_convert_f32_i16().build();
                Ok(Some(blk))
            }
            "dump_u8" => {
                let blk = Sink::new(|x: &u8| print!("{:02x} ", *x));
                Ok(Some(blk))
            }
            "dump_f" => {
                let blk = Sink::new(|x: &f32| print!("{:e} ", *x));
                Ok(Some(blk))
            }
            "blocks_file_source" => {
                let filename = blk_def
                    .parameters
                    .get("file")
                    .expect("filename must be defined");
                let item_type = blk_def
                    .parameters
                    .get("type")
                    .expect("item type must be defined");
                let _repeat = blk_def
                    .parameters
                    .get("repeat")
                    .unwrap_or(&"False".to_string());
                let filename = if "-" == filename {
                    "/proc/self/fd/0"
                } else {
                    filename
                };
                let blk = match &(item_type[..]) {
                    "u8" => FileSource::<u8>::new(filename, false),
                    "byte" => FileSource::<i8>::new(filename, false),
                    "f32" => FileSource::<f32>::new(filename, false),
                    "float" => FileSource::<f32>::new(filename, false),
                    "c32" => FileSource::<Complex32>::new(filename, false),
                    "complex" => FileSource::<Complex32>::new(filename, false),
                    _ => todo!("Unhandled FileSource Type {item_type}"),
                };
                Ok(Some(blk))
            }
            "blocks_file_sink" => {
                let filename = blk_def
                    .parameters
                    .get("file")
                    .expect("filename must be defined");
                let item_type = blk_def
                    .parameters
                    .get("type")
                    .expect("item type must be defined");
                if "-" == filename {
                    let blk = match &(item_type[..]) {
                        "u8" => StdInOutBuilder::<u8>::stdout().as_ne().build(),
                        "i16" => StdInOutBuilder::<i16>::stdout().as_ne().build(),
                        "short" => StdInOutBuilder::<i16>::stdout().as_ne().build(),
                        "f32" => StdInOutBuilder::<f32>::stdout().as_ne().build(),
                        "float" => StdInOutBuilder::<f32>::stdout().as_ne().build(),
                        "c32" => StdInOutBuilder::<Complex32>::stdout().as_ne().build(),
                        "complex" => StdInOutBuilder::<Complex32>::stdout().as_ne().build(),
                        _ => todo!("Unhandled FileSink Type {item_type}"),
                    };
                    Ok(Some(blk))
                } else {
                    let blk = match &(item_type[..]) {
                        "u8" => FileSink::<u8>::new(filename),
                        "i16" => FileSink::<i16>::new(filename),
                        "short" => FileSink::<i16>::new(filename),
                        "f32" => FileSink::<f32>::new(filename),
                        "float" => FileSink::<f32>::new(filename),
                        "c32" => FileSink::<Complex32>::new(filename),
                        "complex" => FileSink::<Complex32>::new(filename),
                        _ => todo!("Unhandled FileSink Type {item_type}"),
                    };
                    Ok(Some(blk))
                }
            }
            "dc_blocker_xx" => {
                let min_bufsize = "32".to_string();
                let min_bufsize = blk_def.parameters.get("length").or(Some(&min_bufsize));
                let min_bufsize = min_bufsize
                    .expect("")
                    .parse::<usize>()
                    .expect("invalid length");
                let dc_blocker = DCBlocker::<f32>::build(min_bufsize);
                Ok(Some(dc_blocker))
            }
            "analog_quadrature_demod_cf" => {
                let default_gain = "1.0".to_string();
                let gain = blk_def.parameters.get("gain").or(Some(&default_gain));
                let gain = gain.expect("").parse::<f32>().expect("invalid gain");
                // println!("gain: {gain}");
                let mut last = Complex32::new(0.0, 0.0); // store sample x[n-1]
                let demod = Apply::new(move |v: &Complex32| -> f32 {
                    let arg = (v * last.conj()).arg(); // Obtain phase of x[n] * conj(x[n-1])
                    last = *v;
                    arg * gain
                });
                Ok(Some(demod))
            }
            "analog_fm_deemph" => {
                let sample_rate = blk_def
                    .parameters
                    .get("samp_rate")
                    .expect("samp_rate must be defined");
                let sample_rate = sample_rate.parse::<f32>()?;
                let tau = blk_def
                    .parameters
                    .get("tau")
                    .expect("tau must be defined")
                    .parse::<f32>()?;
                let dt = 1.0 / sample_rate;
                let alpha = dt / (tau + dt);
                let mut last = 0.0; // store sample x[n-1]
                let blk = Apply::new(move |v: &f32| -> f32 {
                    let r = alpha * v + (1.0 - alpha) * last; //this is the simplest IIR LPF
                    last = r;
                    r
                });
                Ok(Some(blk))
            }
            "rational_resampler_xxx" => {
                let interp = blk_def
                    .parameters
                    .get("interp")
                    .expect("interp must be defined")
                    .parse::<usize>()?;
                let decim = blk_def
                    .parameters
                    .get("decim")
                    .expect("decim must be defined")
                    .parse::<usize>()?;
                let blk = FirBuilder::new_resampling::<f32, f32>(interp, decim);
                Ok(Some(blk))
            }
            "blocks_throttle" => {
                let rate = blk_def
                    .parameters
                    .get("samples_per_second")
                    .expect("samples_per_second must be defined for blocks_throttle")
                    .parse::<f64>()?;
                let item_type = blk_def
                    .parameters
                    .get("type")
                    .expect("item type must be defined");
                let blk = match &(item_type[..]) {
                    "char" => Throttle::<u8>::new(rate),
                    "short" => Throttle::<i16>::new(rate),
                    "float" => Throttle::<f32>::new(rate),
                    "complex" => Throttle::<Complex32>::new(rate),
                    _ => todo!("Unhandled blocks_throttle Type {item_type}"),
                };
                Ok(Some(blk))
            }
            "variable" => Ok(None),
            "convert_ff_c" => {
                let blk = ApplyNM::<_, _, _, 2, 1>::new(move |v: &[f32], d: &mut [Complex32]| {
                    d[0] = Complex32::new(v[0], v[1])
                });
                Ok(Some(blk))
            }
            "audio_sink" => {
                let rate = blk_def
                    .parameters
                    .get("samp_rate")
                    .expect("rate must be defined")
                    .parse::<u32>()?;
                let blk = AudioSink::new(rate, 1);
                Ok(Some(blk))
            }
            "blocks_add_const_vxx" => {
                let item_type = blk_def
                    .parameters
                    .get("type")
                    .expect("item type must be defined");
                let blk = match &(item_type[..]) {
                    "u8" => {
                        let constant = blk_def
                            .parameters
                            .get("const")
                            .expect("constant must be defined")
                            .parse::<u8>()?;
                        Apply::new(move |v: &u8| -> u8 { v + constant })
                    }
                    "float" => {
                        let constant = blk_def
                            .parameters
                            .get("const")
                            .expect("constant must be defined")
                            .parse::<f32>()?;
                        Apply::new(move |v: &f32| -> f32 { v + constant })
                    }
                    // "i16" => FileSink::<i16>::new(filename),
                    // "f32" => FileSink::<f32>::new(filename),
                    // "float" => FileSink::<f32>::new(filename),
                    // "c32" => FileSink::<Complex32>::new(filename),
                    // "complex" => FileSink::<Complex32>::new(filename),
                    _ => todo!("Unhandled blocks_add_const_vxx Type {item_type}"),
                };
                Ok(Some(blk))
            }
            "blocks_char_to_float" => {
                let scale = blk_def
                    .parameters
                    .get("scale")
                    .expect("scale must be defined")
                    .parse::<f32>()?;
                // println!("blocks_char_to_float scale: {scale}");
                let blk = Apply::new(move |v: &i8| -> f32 { (*v) as f32 * scale });
                Ok(Some(blk))
            }
            "blocks_float_to_short" => {
                let scale = blk_def
                    .parameters
                    .get("scale")
                    .expect("scale must be defined")
                    .parse::<f32>()?;
                // println!("blocks_float_to_short scale {scale}");
                let blk = Apply::new(move |v: &f32| -> i16 { (*v * scale) as i16 });
                Ok(Some(blk))
            }
            "blocks_deinterleave" => {
                let item_type = blk_def
                    .parameters
                    .get("type")
                    .expect("item type must be defined");
                let blk = match &(item_type[..]) {
                    "char" => Deinterleave::<u8>::new(),
                    "short" => Deinterleave::<i16>::new(),
                    "float" => Deinterleave::<f32>::new(),
                    "complex" => Deinterleave::<Complex32>::new(),
                    _ => todo!("Unhandled blocks_deinterleave Type {item_type}"),
                };
                Ok(Some(blk))
            }
            "blocks_float_to_complex" => {
                let blk =
                    Combine::new(|v1: &f32, v2: &f32| -> Complex32 { Complex32::new(*v1, *v2) });
                Ok(Some(blk))
            }
            "octave_complex_c" => {
                let samples_to_plot = blk_def
                    .parameters
                    .get("samples_to_plot")
                    .expect("samples_to_plot must be defined")
                    .parse::<usize>()?;
                let out_of_n_samples = blk_def
                    .parameters
                    .get("out_of_n_samples")
                    .expect("out_of_n_samples must be defined")
                    .parse::<usize>()?;
                if out_of_n_samples < samples_to_plot {
                    return Err(anyhow!("out_of_n_samples should be < samples_to_plot"));
                }
                let blk = OctaveComplex::build(samples_to_plot, out_of_n_samples);
                Ok(Some(blk))
            }
            "blocks_freqshift_cc" => {
                let sample_rate = blk_def
                    .parameters
                    .get("sample_rate")
                    .expect("sample_rate must be defined")
                    .parse::<f32>()?;
                let freq = blk_def
                    .parameters
                    .get("freq")
                    .expect("freq must be defined")
                    .parse::<f32>()?;
                let blk = FrequencyShifter::<Complex32>::new(freq, sample_rate);
                Ok(Some(blk))
            }
            "blocks_null_sink" => {
                let item_type = blk_def
                    .parameters
                    .get("type")
                    .expect("item type must be defined");
                let blk = match &(item_type[..]) {
                    "char" => NullSink::<u8>::new(),
                    "short" => NullSink::<i16>::new(),
                    "float" => NullSink::<f32>::new(),
                    "complex" => NullSink::<Complex32>::new(),
                    _ => todo!("Unhandled blocks_null_sink Type {item_type}"),
                };
                Ok(Some(blk))
            }
            "fir_filter_xxx" => {
                let taps = blk_def
                    .parameters
                    .get("taps")
                    .expect("taps must be defined");
                let decimation = blk_def
                    .parameters
                    .get("decim")
                    .expect("decim must be defined")
                    .parse::<usize>()?;
                let item_type = blk_def
                    .parameters
                    .get("type")
                    .expect("type must be defined");
                let taps: Vec<f32> = if taps.is_empty() {
                    // This block definition was from csdr
                    let transition_bw = blk_def
                        .parameters
                        .get("transition_bw")
                        .expect("transition_bw must be defined")
                        .parse::<f64>()?;
                    let window = blk_def
                        .parameters
                        .get("window")
                        .expect("window must be defined");
                    let taps_length: usize = (4.0 / transition_bw) as usize;
                    let taps_length = taps_length + if taps_length % 2 == 0 { 1 } else { 0 };
                    assert!(taps_length % 2 == 1); //number of symmetric FIR filter taps should be odd

                    // Building firdes_lowpass_f(taps,taps_length,0.5/(float)factor,window);
                    let rect_win = match &window[..] {
                        "HAMMING" => windows::hamming(taps_length, false),
                        "BLACKMAN" => windows::blackman(taps_length, false),
                        //"KAISER" => windows::kaiser(taps_length, beta),
                        "HANN" => windows::hann(taps_length, false),
                        //"GAUSSIAN" => windows::gaussian(taps_length, alpha),
                        _ => todo!("Unknown fir_filter_xx window: {window}"),
                    };
                    let taps = firdes::lowpass::<f32>(transition_bw, rect_win.as_slice());
                    taps
                } else {
                    todo!("Unhandled fir_filter_xx taps definition")
                };
                let blk = match &(item_type[..]) {
                    "ccc" => FirBuilder::new_resampling_with_taps::<Complex32, Complex32, f32, _>(
                        1, decimation, taps,
                    ),
                    _ => todo!("Unhandled fir_filter_xx Type {item_type}"),
                };
                Ok(Some(blk))
            }
            "low_pass_filter" => {
                let beta = blk_def
                    .parameters
                    .get("beta")
                    .expect("beta must be defined")
                    .parse::<f64>();
                let cutoff_freq = blk_def
                    .parameters
                    .get("cutoff_freq")
                    .expect("cutoff_freq must be defined")
                    .parse::<f64>()?; // Cutoff frequency in Hz
                let decimation = blk_def
                    .parameters
                    .get("decim")
                    .expect("decim must be defined")
                    .parse::<usize>()?; // Decimation rate of filter
                let gain = blk_def
                    .parameters
                    .get("gain")
                    .expect("gain must be defined")
                    .parse::<f32>()?;
                let interp = blk_def
                    .parameters
                    .get("interp")
                    .expect("interp must be defined")
                    .parse::<usize>()?;
                let sample_rate = blk_def
                    .parameters
                    .get("samp_rate")
                    .expect("samp_rate must be defined")
                    .parse::<f64>()?;
                let item_type = blk_def
                    .parameters
                    .get("type")
                    .expect("type must be defined");
                let width = blk_def
                    .parameters
                    .get("width")
                    .expect("width must be defined")
                    .parse::<f64>()?; // Transition width between stop-band and pass-band in Hz
                let window = blk_def.parameters.get("win").expect("win must be defined");
                let transition_bw = cutoff_freq / sample_rate;
                let taps_length: usize = (4.0 / transition_bw) as usize;
                let taps_length = taps_length + if taps_length % 2 == 0 { 1 } else { 0 };
                assert!(taps_length % 2 == 1); //number of symmetric FIR filter taps should be odd
                let alpha = beta.clone();
                let rect_win = match &window[..] {
                    "window.WIN_HAMMING" => windows::hamming(taps_length, false),
                    "window.WIN_BLACKMAN" => windows::blackman(taps_length, false),
                    "window.WIN_KAISER" => {
                        windows::kaiser(taps_length, beta.expect("beta is mandatory for Kaiser"))
                    }
                    "window.WIN_HANN" => windows::hann(taps_length, false),
                    "window.WIN_GAUSSIAN" => windows::gaussian(
                        taps_length,
                        alpha.expect("alpha is mandatory for Gaussian"),
                    ),
                    _ => todo!("Unknown low_pass_filter window: {window}"),
                };
                let taps = firdes::lowpass::<f32>(transition_bw, rect_win.as_slice());
                let blk = match &(item_type[..]) {
                    "fir_filter_ccf" => {
                        FirBuilder::new_resampling_with_taps::<Complex32, Complex32, f32, _>(
                            interp, decimation, taps,
                        )
                    }
                    _ => todo!("Unhandled low_pass_filter Type {item_type}"),
                };
                Ok(Some(blk))
            }
            _ => {
                let unknow_block_type = blk_def.id.clone();
                todo!("unknow_block_type: {unknow_block_type}")
            }
        }
    }
}
