use crate::grc::Grc;
use anyhow::{Context, Result};
use serde_yaml::{self};
use std::path::Path;

#[derive(Default, Clone)]
pub struct GrcParser;

impl GrcParser {
    pub fn load<P>(filename: P) -> Result<Grc>
    where
        P: AsRef<Path>,
    {
        let f = std::fs::File::open(filename).context("Could not open file.")?;
        let grc: Grc = serde_yaml::from_reader(f).context("Could not read values.")?;
        Ok(grc)
    }

    pub fn save<P>(filename: P, grc: &Grc) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(filename)
            .context("Couldn't open file")?;
        serde_yaml::to_writer(f, grc).context("Could not write values.")?;
        Ok(())
    }
}
