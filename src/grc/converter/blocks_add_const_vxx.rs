use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::{BlockInstance, Grc2FutureSdr};
use anyhow::Result;
use futuresdr::blocks::Apply;
use futuresdr::runtime::Flowgraph;

pub struct AddConstVxConverter {}

impl BlockConverter for AddConstVxConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let constant = Grc2FutureSdr::parameter_as_f64(blk, "const", "0.0")? as f32;
        let item_type = blk
            .parameters
            .get("type")
            .expect("item type must be defined");

        let blk: Box<dyn ConnectorAdapter> = match &(item_type[..]) {
            "u8" => {
                let constant = constant as u8;
                let blk: Apply<_, u8, u8> = Apply::new(move |v: &u8| -> u8 { v + constant });
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            "float" => {
                let blk: Apply<_, f32, f32> = Apply::new(move |v: &f32| -> f32 { v + constant });
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            _ => todo!("Unhandled blocks_add_const_vxx Type {item_type}"),
        };
        Ok(blk)
    }
}
