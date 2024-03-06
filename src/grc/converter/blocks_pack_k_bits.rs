use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use fsdr_blocks::futuresdr::anyhow::Result;
// use fsdr_blocks::futuresdr::blocks::ApplyNM;
use fsdr_blocks::futuresdr::runtime::Flowgraph;

pub struct PackBitsConverter {}

impl BlockConverter for PackBitsConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let _k = Grc2FutureSdr::parameter_as_f32(blk, "k", "8")? as usize;
        // todo!("handle different value of k than 8")
        // let blk = ApplyNM::<_, u8, u8, 8, 1>::new(move |v: &[u8], d: &mut [u8]| {
        //     d[0] = v
        //         .iter().rev()
        //         .enumerate()
        //         .map(|(i, u)| (*u) << i)
        //         .reduce(|a, b| a | b)
        //         .expect("guaranteee to not be empty due to ApplyNM");
        // });
        let blk = fsdr_blocks::byte::pack_8_in_1();
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
