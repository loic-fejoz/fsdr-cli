use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait LogAveragePowerCmd<'i> {
    fn fft_size(&self) -> Result<&str>;
    fn avg_number(&self) -> Result<&str>;
    fn add_db(&self) -> Result<Option<&str>>;

    fn build_logaveragepower_cf(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let fft_size = self.fft_size()?;
        let avg_number = self.avg_number()?;
        let add_db = self.add_db()?.unwrap_or("0.0");
        grc = grc
            .ensure_source(GrcItemType::C32)?
            .create_block_instance("logaveragepower_cf")
            .with_parameter("fft_size", fft_size)
            .with_parameter("avg_number", avg_number)
            .with_parameter("add_db", add_db)
            .assert_output(GrcItemType::F32)
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> LogAveragePowerCmd<'i> for Pair<'i, Rule> {
    fn fft_size(&self) -> Result<&'i str> {
        let mut inner = self.clone().into_inner();
        if let Some(value) = inner.next() {
            Ok(value.as_str())
        } else {
            bail!("missing mandatory <fft_size> parameter for logaveragepower")
        }
    }

    fn avg_number(&self) -> Result<&'i str> {
        let mut inner = self.clone().into_inner();
        inner.next();
        if let Some(value) = inner.next() {
            Ok(value.as_str())
        } else {
            bail!("missing mandatory <avg_number> parameter for logaveragepower")
        }
    }

    fn add_db(&self) -> Result<Option<&'i str>> {
        let mut inner = self.clone().into_inner();
        inner.next();
        inner.next();
        if let Some(value) = inner.next() {
            let mut val_inner = value.clone().into_inner();
            if let Some(sub) = val_inner.next() {
                Ok(Some(sub.as_str()))
            } else {
                Ok(Some(value.as_str()))
            }
        } else {
            Ok(None)
        }
    }
}
