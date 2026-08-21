use crate::blocks::AddDcOffsetCc;
use crate::grc::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use crate::grc::BlockInstance;
use anyhow::{Context, Result};
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;

pub struct AddDcOffsetCcConverter {}

impl BlockConverter for AddDcOffsetCcConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let blk = match (
            blk.parameters.get("offset_re"),
            blk.parameters.get("offset_im"),
        ) {
            (None, None) => AddDcOffsetCc::default(),
            (Some(re), None) => {
                let re: f32 = re.parse().context("failed to parse offset_re")?;
                AddDcOffsetCc::new(Complex32::new(re, 0.0))
            }
            (re_opt, im_opt) => {
                let re: f32 = re_opt.map(|s| s.parse::<f32>()).transpose()?.unwrap_or(1.0);
                let im: f32 = im_opt.map(|s| s.parse::<f32>()).transpose()?.unwrap_or(0.0);
                AddDcOffsetCc::new(Complex32::new(re, im))
            }
        };
        let id = fg.add(blk)?.id();
        Ok(Box::new(DefaultPortAdapter::new(id)))
    }
}
