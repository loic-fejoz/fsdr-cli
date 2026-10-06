use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use anyhow::{bail, Result};
use pest::iterators::Pair;

pub trait AdpcmCmd<'i> {
    fn fft_size(&self) -> Result<&str>;

    fn build_adpcm_encoder(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::S16)?
            .create_block_instance("encode_ima_adpcm_i16_u8")
            .assert_output(GrcItemType::U8)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_adpcm_decoder(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        grc = grc
            .ensure_source(GrcItemType::U8)?
            .create_block_instance("decode_ima_adpcm_u8_i16")
            .assert_output(GrcItemType::S16)
            .push_and_link()?;
        Ok(grc)
    }

    fn build_compress_fft_adpcm(
        &self,
        grc: GrcBuilder<GraphLevel>,
    ) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let fft_size = self.fft_size()?;
        grc = grc
            .ensure_source(GrcItemType::F32)?
            .create_block_instance("compress_fft_adpcm_f_u8")
            .with_parameter("fft_size", fft_size)
            .assert_output(GrcItemType::U8)
            .push_and_link()?;
        Ok(grc)
    }
}

impl<'i> AdpcmCmd<'i> for Pair<'i, Rule> {
    fn fft_size(&self) -> Result<&'i str> {
        let mut inner = self.clone().into_inner();
        if let Some(val) = inner.next() {
            let mut val_inner = val.clone().into_inner();
            if let Some(sub) = val_inner.next() {
                Ok(sub.as_str())
            } else {
                Ok(val.as_str())
            }
        } else {
            bail!("missing mandatory <fft_size> parameter for compress_fft_adpcm_f_u8 / fftadpcm")
        }
    }
}
