use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use anyhow::{bail, Result};
use fsdr_blocks::type_converters::TypeConvertersBuilder;
use futuresdr::blocks::ApplyNM;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;

pub struct ConvertBlockConverter {}

impl BlockConverter for ConvertBlockConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let blk_id = &blk.id[..];
        let blk: Box<dyn ConnectorAdapter> = match blk_id {
            "blocks_uchar_to_float" | "convert_u8_f" => {
                let blk = TypeConvertersBuilder::scale_convert::<u8, f32>().build();
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            "blocks_char_to_float" | "convert_s8_f" => {
                let blk = TypeConvertersBuilder::scale_convert::<i8, f32>().build();
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            "convert_s16_f" => {
                let blk = TypeConvertersBuilder::scale_convert::<i16, f32>().build();
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            "blocks_float_to_uchar" | "convert_f_u8" => {
                let blk = TypeConvertersBuilder::lossy_scale_convert_f32_u8().build();
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            "blocks_float_to_char" | "convert_f_s8" => {
                let blk = TypeConvertersBuilder::lossy_scale_convert_f32_i8().build();
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            "blocks_float_to_short" | "convert_f_s16" => {
                let blk = TypeConvertersBuilder::lossy_scale_convert_f32_i16().build();
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            "convert_ff_c" => {
                let blk = ApplyNM::<_, _, _, 2, 1>::new(move |v: &[f32], d: &mut [Complex32]| {
                    d[0] = Complex32::new(v[0], v[1])
                });
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            "blocks_short_to_float" => {
                let blk = TypeConvertersBuilder::scale_convert::<i16, f32>().build();
                Box::new(DefaultPortAdapter::new(fg.add_block(blk).into()))
            }
            _ => bail!("Unknown conversion: {blk_id}"),
        };
        Ok(blk)
    }
}
