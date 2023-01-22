use crate::grc::Grc;
use fsdr_blocks::type_converters::*;
use futuresdr::anyhow::Result;
use futuresdr::blocks::{Apply, FileSink, FileSource, Sink};
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
        let fsdr_blocks: Vec<Option<usize>> = grc
            .blocks
            .iter()
            .map(|blk_def| Self::convert_add_block(&mut fg, blk_def, &grc).expect(""))
            .collect();
        for (name, idx) in names.iter().zip(fsdr_blocks.iter()) {
            if let Some(idx) = *idx {
                names_to_id.insert(name.clone(), idx);
            }
        }
        for connection in grc.connections {
            let src_name = connection[0].clone();
            let src_block = *names_to_id
                .get(&src_name)
                .expect("no source block id found for");

            let tgt_name = connection[2].clone();
            let dst_block = *names_to_id
                .get(&tgt_name)
                .expect("no target block id found for");

            let src_port = Self::adapt_src_port(&connection[1]);
            let dst_port = Self::adapt_dst_port(&connection[3]);

            fg.connect_stream(src_block, src_port, dst_block, dst_port)?;
        }
        Ok(fg)
    }

    fn adapt_src_port(port_out: &str) -> &str {
        if "0" == port_out {
            "out"
        } else {
            port_out
        }
    }

    fn adapt_dst_port(port_in: &str) -> &str {
        if "0" == port_in {
            "in"
        } else {
            port_in
        }
    }

    fn convert_add_block(
        fg: &mut Flowgraph,
        blk_def: &BlockInstance,
        grc: &Grc,
    ) -> Result<Option<usize>> {
        let block = Self::convert_block(blk_def, grc)?;
        if let Some(block) = block {
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
            "blocks_complex_to_real" => {
                // TODO: should do an analysis on how many outputs are really used,
                // to know exactly what to generate
                let realpart_blk = Apply::new(|i: &Complex32| -> f32 { i.re });
                Ok(Some(realpart_blk))
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
            // "convert_f_u8" => {
            //     let blk = TypeConvertersBuilder::scale_convert::<f32, u8>().build();
            //     Ok(Some(blk))
            // },
            // "convert_f_s8" => {
            //     let blk = TypeConvertersBuilder::scale_convert::<f32, i8>().build();
            //     Ok(Some(blk))
            // },
            // "convert_f_s16" => {
            //     let blk = TypeConvertersBuilder::scale_convert::<f32, i16>().build();
            //     Ok(Some(blk))
            // },
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
                let filename = if "-" == filename {
                    "/proc/self/fd/1"
                } else {
                    filename
                };
                let blk = match &(item_type[..]) {
                    "u8" => FileSink::<u8>::new(filename),
                    "f32" => FileSink::<f32>::new(filename),
                    "float" => FileSink::<f32>::new(filename),
                    "c32" => FileSink::<Complex32>::new(filename),
                    "complex" => FileSink::<Complex32>::new(filename),
                    _ => todo!("Unhandled FileSink Type {item_type}"),
                };
                Ok(Some(blk))
            }
            "variable" => Ok(None),
            _ => {
                let unknow_block_type = blk_def.id.clone();
                todo!("{unknow_block_type}")
            }
        }
    }
}
