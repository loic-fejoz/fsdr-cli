use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use anyhow::Result;
use futuresdr::blocks::ApplyNM;
use futuresdr::runtime::Flowgraph;

pub struct UnpackBitsConverter {}

impl BlockConverter for UnpackBitsConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let _k = Grc2FutureSdr::parameter_as_f32(blk, "k", "8")? as usize;
        let blk = ApplyNM::<_, u8, u8, 1, 8>::new(move |v: &[u8], d: &mut [u8]| {
            let byte = v[0];
            for (i, item) in d.iter_mut().enumerate().take(8) {
                *item = (byte >> (7 - i)) & 1;
            }
        });
        let blk = fg.add(blk)?.id();
        let blk = DefaultPortAdapter::new(blk);
        Ok(Box::new(blk))
    }
}
