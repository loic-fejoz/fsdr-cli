use crate::grc::Grc;
use futuresdr::anyhow::Result;
// use serde_yaml;
use std::path::Path;

#[derive(Default, Clone)]
pub struct GrcParser;

impl GrcParser {
    pub fn load<P>(filename: P) -> Result<Grc>
    where
        P: AsRef<Path>,
    {
        let f = std::fs::File::open(filename).expect("Could not open file.");
        let grc: Grc = serde_yaml::from_reader(f).expect("Could not read values.");
        Ok(grc)
    }

    pub fn save<P>(filename: P, grc: &Grc)
    where
        P: AsRef<Path>,
    {
        let f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(filename)
            .expect("Couldn't open file");
        serde_yaml::to_writer(f, grc).unwrap();
    }
}
