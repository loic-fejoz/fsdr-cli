use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub mod builder;
pub mod converter_helper;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Grc {
    pub options: Options,
    pub blocks: Vec<BlockInstance>,
    pub connections: Vec<[String; 4]>,
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct Options {
    parameters: Parameters,
    states: States,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Parameters {
    author: String,
    // catch_exceptions: String,
    // category: String,
    // comment: String,
    // copyright: String,
    // description: String,
    id: String,
    title: String,
}

impl Default for Parameters {
    fn default() -> Self {
        Parameters {
            author: "fsdr-cli".to_string(),
            id: "fsdrcli".to_string(),
            title: "Created by fsdr-cli".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct States {
    bus_sink: bool,
    bus_source: bool,
    pub state: String,
    coordinate: [usize; 2],
}

impl Default for States {
    fn default() -> Self {
        States {
            bus_sink: false,
            bus_source: false,
            state: "enabled".to_string(),
            coordinate: [8, 8],
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Metadata {
    pub file_format: i32,
    pub grc_version: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct BlockInstance {
    pub name: String,
    pub id: String,
    pub parameters: BTreeMap<String, String>,
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
