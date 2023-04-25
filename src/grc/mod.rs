use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub mod builder;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Grc {
    pub options: Options,
    pub blocks: Vec<BlockInstance>,
    pub connections: Vec<Vec<String>>,
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct Options {
    parameters: Parameters,
    // states: States,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct Parameters {
    // author: String,
    // catch_exceptions: String,
    // category: String,
    // comment: String,
    // copyright: String,
    // description: String,
    // title: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
pub struct States {
    bus_sink: bool,
    bus_source: bool,
    pub state: String,
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
            states: States {
                bus_sink: false,
                bus_source: false,
                state: "enabled".to_string(),
            },
        }
    }

    pub fn with(mut self, key: &str, value: &str) -> BlockInstance {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }
}
