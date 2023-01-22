use crate::grc::*;
use futuresdr::anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::iter::Peekable;

#[derive(Default)]
pub struct CsdrParser {
    block_count: usize,
    blocks: Vec<BlockInstance>,
    connections: Vec<Vec<String>>,
}

impl CsdrParser {
    pub fn parse_command<A>(args: Peekable<A>) -> Result<Grc>
    where
        A: Iterator<Item = String>,
    {
        let mut csdr_parser = CsdrParser::default();
        let (block_name, input_type, output_type) = csdr_parser
            .parse_one_command(args)
            .context("invalid csdr command")
            .expect("valid csdr command");
        let src_name = "blocks_file_source_0";
        let stdin_source = BlockInstance::new(src_name, "blocks_file_source")
            .with("file", "-")
            .with("type", &input_type)
            .with("repeat", "False");
        csdr_parser.push_block(stdin_source);
        csdr_parser.connect(src_name, "0", block_name.as_str(), "0");
        let sink_name = "blocks_file_sink_0";
        let stdout_sink = BlockInstance::new(sink_name, "blocks_file_sink")
            .with("file", "-")
            .with("type", &output_type);
        csdr_parser.push_block(stdout_sink);
        csdr_parser.connect(&block_name, "0", sink_name, "0");
        csdr_parser.build()
    }

    pub fn parse_one_command<A>(
        &mut self,
        mut args: Peekable<A>,
    ) -> Result<(String, String, String)>
    where
        A: Iterator<Item = String>,
    {
        let cmd_name = args.next().expect("no command");
        match &cmd_name[..] {
            "realpart_cf" => {
                let parameters = BTreeMap::new();
                let block_name = self.push_block_instance("realpart_cf".into(), parameters);
                Ok((block_name, "c32".to_string(), "f32".to_string()))
            },
            "convert_u8_f" => {
                let parameters = BTreeMap::new();
                let block_name = self.push_block_instance("convert_u8_f".into(), parameters);
                Ok((block_name, "u8".to_string(), "f32".to_string()))
            },
            _ => todo!("parse_command"),
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

    pub fn parse_multiple_commands() -> Result<Grc> {
        todo!("parse_multiple_commands");
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
