use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use fsdr_blocks::futuresdr::anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait OctaveComplexCmd<'i> {
    fn samples_to_plot(&self) -> Result<&'i str>;
    fn out_of_n_samples(&self) -> Result<&'i str>;
    fn build_octave_complex(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let samples_to_plot = self.samples_to_plot()?;
        let out_of_n_samples = self.out_of_n_samples()?;
        grc = grc
            .ensure_source(GrcItemType::C32)
            .create_block_instance("octave_complex_c")
            .with_parameter("samples_to_plot", samples_to_plot)
            .with_parameter("out_of_n_samples", out_of_n_samples)
            .push_and_link();
        Ok(grc)
    }
}

impl<'i> OctaveComplexCmd<'i> for Pair<'i, Rule> {
    fn samples_to_plot(&self) -> Result<&'i str> {
        println!("{self}");
        if let Some(value) = self.clone().into_inner().next() {
            Ok(value.as_str())
        } else {
            bail!("missing mandatory <samples_to_plot> parameters for octave_complex_c")
        }
    }

    fn out_of_n_samples(&self) -> Result<&'i str> {
        let mut inner = self.clone().into_inner();
        inner.next();
        if let Some(value) = inner.next() {
            Ok(value.as_str())
        } else {
            bail!("missing mandatory <out_of_n_samples> parameters for octave_complex_c")
        }
    }
}
