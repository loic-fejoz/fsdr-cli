use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use futuresdr::anyhow::Result;
use futuresdr::blocks::Apply;
use futuresdr::runtime::Flowgraph;

pub struct MulConstVxConverter {}

impl BlockConverter for MulConstVxConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let constant = Grc2FutureSdr::parameter_as_f32(blk, "const", "0.0")?;
        let item_type = blk
            .parameters
            .get("type")
            .expect("item type must be defined");

        let blk = match &(item_type[..]) {
            "u8" => {
                let constant = constant as u8;
                Apply::new(move |v: &u8| -> u8 { v * constant })
            }
            "float" => Apply::new(move |v: &f32| -> f32 { v * constant }),
            _ => todo!("Unhandled blocks_multiply_const_vxx Type {item_type}"),
        };
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
