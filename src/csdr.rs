use crate::grc::*;
use futuresdr::anyhow::{Context, Result};
use std::collections::BTreeMap;
//use std::default::{default, self};
use std::iter::Peekable;

#[derive(Default)]
pub struct CsdrParser {
    block_count: usize,
    blocks: Vec<BlockInstance>,
    connections: Vec<Vec<String>>,
}

impl CsdrParser {
    pub fn parse_command<A, S>(args: &mut Peekable<A>) -> Result<Grc>
    where
        A: Iterator<Item = S>,
        S: Into<String> + Clone,
    {
        let mut csdr_parser = CsdrParser::default();
        let (block_name, input_type, output_type) = csdr_parser
            .parse_one_command(args)
            .context("invalid csdr command")
            .expect("valid csdr command");
        csdr_parser.add_source_and_maybe_sink(
            block_name.as_str(),
            input_type.as_str(),
            Some(block_name.as_str()),
            output_type,
        );
        csdr_parser.build()
    }

    fn add_source_and_maybe_sink(
        &mut self,
        first_block: &str,
        first_block_type: &str,
        last_block: Option<impl Into<String>>,
        last_block_type: Option<impl Into<String>>,
    ) {
        let src_name = "blocks_file_source_0";
        let stdin_source = BlockInstance::new(src_name, "blocks_file_source")
            .with("file", "-")
            .with("type", first_block_type)
            .with("repeat", "False");
        self.push_block(stdin_source);
        self.connect(src_name, "0", first_block, "0");
        if let Some(last_block_type) = last_block_type {
            let sink_name = "blocks_file_sink_0";
            let stdout_sink = BlockInstance::new(sink_name, "blocks_file_sink")
                .with("file", "-")
                .with("type", last_block_type.into().as_str());
            self.push_block(stdout_sink);
            self.connect(last_block.expect("").into().as_str(), "0", sink_name, "0");
        }
    }

    pub fn parse_one_command<A, S>(
        &mut self,
        args: &mut Peekable<A>,
    ) -> Result<(String, String, Option<String>)>
    where
        A: Iterator<Item = S>,
        S: Into<String> + Clone,
    {
        let cmd_name = args.next().expect("no command").into();
        match &cmd_name[..] {
            "amdemod_cf" => {
                let parameters = BTreeMap::new();
                let block_name = self.push_block_instance("blocks_complex_to_mag".into(), parameters);
                Ok((block_name, "c32".to_string(), Some("f32".to_string())))
            }
            "clipdetect_ff" => {
                let parameters = BTreeMap::new();
                let block_name = self.push_block_instance("clipdetect_ff".into(), parameters);
                Ok((block_name, "f32".to_string(), Some("f32".to_string())))
            }
            "convert_u8_f" => {
                let parameters = BTreeMap::new();
                let block_name = self.push_block_instance("convert_u8_f".into(), parameters);
                Ok((block_name, "u8".to_string(), Some("f32".to_string())))
            }
            "convert_s8_f" => {
                let parameters = BTreeMap::new();
                let block_name = self.push_block_instance("convert_s8_f".into(), parameters);
                Ok((block_name, "i8".to_string(), Some("f32".to_string())))
            }
            "convert_s16_f" => {
                let parameters = BTreeMap::new();
                let block_name = self.push_block_instance("convert_s16_f".into(), parameters);
                Ok((block_name, "i16".to_string(), Some("f32".to_string())))
            }
            "convert_f_u8" => {
                let parameters = BTreeMap::new();
                let block_name = self.push_block_instance("convert_f_u8".into(), parameters);
                Ok((block_name, "f32".to_string(), Some("u8".to_string())))
            }
            "convert_f_s8" => {
                let parameters = BTreeMap::new();
                let block_name = self.push_block_instance("convert_f_s8".into(), parameters);
                Ok((block_name, "f32".to_string(), Some("i8".to_string())))
            }
            "convert_f_s16" => {
                let parameters = BTreeMap::new();
                let block_name = self.push_block_instance("convert_f_s16".into(), parameters);
                Ok((block_name, "f32".to_string(), Some("i16".to_string())))
            }
            "convert_ff_c" => {
                let parameters = BTreeMap::new();
                let block_name = self.push_block_instance("convert_ff_c".into(), parameters);
                Ok((block_name, "f32".to_string(), Some("c32".to_string())))
            }
            "dump_u8" => {
                let parameters = BTreeMap::new();
                let block_name = self.push_block_instance("dump_u8".into(), parameters);
                Ok((block_name, "u8".to_string(), None))
            }
            "dump_f" => {
                let parameters = BTreeMap::new();
                let block_name = self.push_block_instance("dump_f".into(), parameters);
                Ok((block_name, "f32".to_string(), None))
            }
            "fastdcblock_ff" => {
                let mut parameters = BTreeMap::new();
                parameters.insert("length".into(), "32".into());
                parameters.insert("long_form".into(), "False".into());
                parameters.insert("type".into(), "ff".into());
                let block_name = self.push_block_instance("dc_blocker_xx".into(), parameters);
                Ok((block_name, "f32".to_string(), Some("f32".to_string())))
            }
            "limit_ff" => {
                let mut parameters = BTreeMap::new();
                let next_arg = args.peek().cloned();
                let mut default_value = true;
                if let Some(next_arg) = next_arg {
                    let hi: String = next_arg.into();
                    if "|" != hi {
                        let _ = args.next();
                        let lo: String = "-".to_owned() + &hi;
                        parameters.insert("lo".into(), lo);
                        parameters.insert("hi".into(), hi);
                        default_value = false;
                    }
                }
                if default_value {
                    parameters.insert("lo".into(), "-1.0".into());
                    parameters.insert("hi".into(), "1.0".into());
                }
                let block_name = self.push_block_instance("analog_rail_ff".into(), parameters);
                Ok((block_name, "f32".to_string(), Some("f32".to_string())))
            }
            "realpart_cf" => {
                let parameters = BTreeMap::new();
                let block_name = self.push_block_instance("realpart_cf".into(), parameters);
                Ok((block_name, "c32".to_string(), Some("f32".to_string())))
            }
            "shift_addition_cc" => {
                let mut parameters = BTreeMap::<String, String>::new();
                let phase_rate = args
                    .next()
                    .expect("missing mandatory <rate> parameters for shift_addition_cc");
                let phase_rate = phase_rate.into();
                parameters.insert("freq".to_string(), phase_rate);
                parameters.insert("sample_rate".to_string(), "6.283185307179586".to_string());
                let block_name = self.push_block_instance("blocks_freqshift_cc".into(), parameters);
                Ok((block_name, "c32".to_string(), Some("c32".to_string())))
            }
            "fmdemod_quadri_cf" => {
                let mut parameters = BTreeMap::<String, String>::new();
                parameters.insert("gain".to_string(), "1.0".to_string());
                let block_name =
                    self.push_block_instance("analog_quadrature_demod_cf".into(), parameters);
                Ok((block_name, "c32".to_string(), Some("f32".to_string())))
            }
            "fractional_decimator_ff" => {
                let mut parameters = BTreeMap::<String, String>::new();
                let resamp_ratio = args
                    .next()
                    .expect("missing mandatory <decim> parameters for fractional_decimator_ff");
                let resamp_ratio = resamp_ratio.into();
                parameters.insert("decim".to_string(), resamp_ratio);
                parameters.insert("interp".to_string(), "1".to_string());
                let block_name =
                    self.push_block_instance("rational_resampler_xxx".into(), parameters);
                Ok((block_name, "f32".to_string(), Some("f32".to_string())))
            }
            "deemphasis_wfm_ff" => {
                let mut parameters = BTreeMap::<String, String>::new();
                {
                    let sample_rate = args
                        .next()
                        .expect("missing mandatory <sample_rate> parameters for deemphasis_wfm_ff");
                    let sample_rate = sample_rate.into();
                    parameters.insert("samp_rate".to_string(), sample_rate);
                }
                {
                    let tau = args
                        .next()
                        .expect("missing mandatory <tau> parameters for deemphasis_wfm_ff");
                    let tau = tau.into();
                    parameters.insert("tau".to_string(), tau);
                }
                let block_name = self.push_block_instance("analog_fm_deemph".into(), parameters);
                Ok((block_name, "f32".to_string(), Some("f32".to_string())))
            }
            "throttle_ff" => {
                let mut parameters = BTreeMap::<String, String>::new();
                let rate = args
                    .next()
                    .expect("missing mandatory <rate> parameters for throttle_ff");
                let rate = rate.into();
                parameters.insert("samples_per_second".to_string(), rate);
                parameters.insert("type".to_string(), "float".to_string());
                let block_name = self.push_block_instance("blocks_throttle".into(), parameters);
                Ok((block_name, "f32".to_string(), Some("f32".to_string())))
            }
            "throttle_cc" => {
                let mut parameters = BTreeMap::<String, String>::new();
                let rate = args
                    .next()
                    .expect("missing mandatory <rate> parameters for throttle_cc");
                let rate = rate.into();
                parameters.insert("samples_per_second".to_string(), rate);
                parameters.insert("type".to_string(), "complex".to_string());
                let block_name = self.push_block_instance("blocks_throttle".into(), parameters);
                Ok((block_name, "c32".to_string(), Some("c32".to_string())))
            }
            "octave_complex_c" => {
                let samples_to_plot = args
                    .next()
                    .expect("missing mandatory <samples_to_plot> parameters for octave_complex_c");
                let samples_to_plot = samples_to_plot.into();
                let out_of_n_samples = args
                    .next()
                    .expect("missing mandatory <out_of_n_samples> parameters for octave_complex_c");
                let out_of_n_samples = out_of_n_samples.into();
                let mut parameters = BTreeMap::<String, String>::new();
                parameters.insert("samples_to_plot".to_string(), samples_to_plot);
                parameters.insert("out_of_n_samples".to_string(), out_of_n_samples);
                let block_name = self.push_block_instance("octave_complex_c".into(), parameters);
                Ok((block_name, "c32".to_string(), None))
            }
            // "file_source_u8" => {
            //     let mut parameters = BTreeMap::<String, String>::new();
            //     let filename = args.next().expect("missing mandatory <filename> parameters for file_source");
            //     let filename = filename.into();
            //     parameters.insert("file".to_string(), rate);
            //     parameters.insert("type".to_string(), "u8".to_string());
            //     let block_name = self.push_block_instance("blocks_file_source".into(), parameters);
            //     Ok((block_name, "c32".to_string(), Some("u8".to_string())))
            // },
            _ => todo!("parse_command {cmd_name}"),
        }
    }

    fn push_block(&mut self, block: BlockInstance) {
        self.block_count += 1;
        self.blocks.push(block);
    }

    fn push_block_instance(
        &mut self,
        kind: String,
        parameters: BTreeMap<String, String>,
    ) -> String {
        let name = format!("{}_{}", kind, self.block_count);
        self.block_count += 1;
        let block = BlockInstance {
            name: name.clone(),
            id: kind,
            parameters,
            states: States::default(),
        };
        self.blocks.push(block);
        name
    }

    pub fn connect(
        &mut self,
        src_name: &str,
        src_port_name: &str,
        tgt_name: &str,
        tgt_port_name: &str,
    ) {
        let connection = vec![
            src_name.to_string(),
            src_port_name.to_string(),
            tgt_name.to_string(),
            tgt_port_name.to_string(),
        ];
        self.connections.push(connection);
    }

    pub fn parse_multiple_commands<A, S>(args: &mut Peekable<A>) -> Result<Grc>
    where
        A: Iterator<Item = S>,
        S: Into<String> + Clone,
    {
        let mut csdr_parser = CsdrParser::default();
        let mut first_block_name: Option<String> = None;
        let mut first_block_type: Option<String> = None;
        let mut last_block_name: Option<String> = None;
        let mut last_block_type: Option<String> = None;
        loop {
            let next = args.peek().cloned();
            if next.is_none() {
                break;
            }
            let next = next.expect("msg");
            let next: String = next.into();
            if next.eq("csdr") || next.eq("|") {
                args.next();
                continue;
            } else {
                let (block_name, input_type, output_type) = csdr_parser
                    .parse_one_command(args)
                    .context("invalid csdr command")
                    .expect("valid csdr command");
                if first_block_name.is_none() {
                    first_block_name = Some(block_name.clone());
                    first_block_type = Some(input_type);
                } else {
                    csdr_parser.connect(
                        last_block_name.expect("").as_str(),
                        "0",
                        block_name.as_str(),
                        "0",
                    );
                }
                last_block_name = Some(block_name);
                last_block_type = output_type;
            }
        }
        csdr_parser.add_source_and_maybe_sink(
            first_block_name.expect("").as_str(),
            first_block_type.expect("").as_str(),
            last_block_name,
            last_block_type,
        );
        csdr_parser.build()
    }

    pub fn build(self) -> Result<Grc> {
        let grc = Grc {
            options: Options::default(),
            blocks: self.blocks,
            connections: self.connections,
            metadata: Metadata {
                file_format: 1,
                grc_version: "3.10.3.0".to_string(),
            },
        };
        Ok(grc)
    }
}
