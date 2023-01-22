use std::path::Path;
use futuresdr::anyhow::Result;
use serde_yaml::{self};
use crate::grc::Grc;


#[derive(Default,Clone)]
pub struct GrcParser;

impl GrcParser {
    pub fn load<P>(filename: P) -> Result<Grc>
    where
        P: AsRef<Path>
    {
        let f = std::fs::File::open(filename).expect("Could not open file.");
        let grc: Grc = serde_yaml::from_reader(f).expect("Could not read values.");
        Ok(grc)
    }
}