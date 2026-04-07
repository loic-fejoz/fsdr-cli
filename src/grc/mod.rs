use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub mod builder;
pub mod converter_helper;

/// Representation of a GNU Radio Companion (GRC) flowgraph.
/// This structure is designed to be 100% compatible with the .grc file format (YAML/JSON representation).
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Grc {
    pub options: Options,
    pub blocks: Vec<BlockInstance>,
    pub connections: Vec<[String; 4]>,
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Options {
    pub parameters: BTreeMap<String, String>,
    pub states: States,
}

impl Default for Options {
    fn default() -> Self {
        let mut parameters = BTreeMap::new();
        parameters.insert("author".to_string(), "fsdr-cli".to_string());
        parameters.insert("id".to_string(), "fsdrcli".to_string());
        parameters.insert("title".to_string(), "Created by fsdr-cli".to_string());
        parameters.insert("generate_options".to_string(), "qt_gui".to_string());
        parameters.insert("output_language".to_string(), "python".to_string());
        parameters.insert("window_size".to_string(), "(1000,1000)".to_string());
        parameters.insert("run".to_string(), "True".to_string());
        parameters.insert("catch_exceptions".to_string(), "True".to_string());

        Options {
            parameters,
            states: States::default(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct States {
    bus_sink: bool,
    bus_source: bool,
    pub state: String,
    coordinate: [f32; 2],
}

impl Default for States {
    fn default() -> Self {
        States {
            bus_sink: false,
            bus_source: false,
            state: "enabled".to_string(),
            coordinate: [8.0, 8.0],
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Metadata {
    pub file_format: i32,
    pub grc_version: String,
}

/// An instance of a GNU Radio block within a flowgraph.
/// `id` and `parameters` keys must match those defined in GRC block YAML files.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct BlockInstance {
    pub name: String,
    pub id: String, // The GRC block ID (e.g., "blocks_file_sink")
    pub parameters: BTreeMap<String, String>, // Parameter names and values from GRC
    pub states: States,
}

mod parser;
pub use parser::GrcParser;

pub mod converter;

impl BlockInstance {
    pub fn new(name: &str, id: &str) -> BlockInstance {
        BlockInstance {
            name: name.to_string(),
            id: id.to_string(),
            parameters: BTreeMap::new(),
            states: States::default(),
        }
    }

    pub fn with(mut self, key: &str, value: &str) -> BlockInstance {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }

    pub fn parameter(&self, key: &str) -> Option<&String> {
        self.parameters.get(key)
    }

    pub fn parameter_or<'i>(&'i self, key: &'i str, default_value: impl Into<&'i str>) -> &'i str {
        if let Some(r) = self.parameters.get(key) {
            r.as_ref()
        } else {
            default_value.into()
        }
    }
}
