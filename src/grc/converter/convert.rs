use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use fsdr_blocks::type_converters::TypeConvertersBuilder;
use futuresdr::anyhow::{bail, Result};
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
        let blk = match blk_id {
            "blocks_uchar_to_float" | "convert_u8_f" => {
                TypeConvertersBuilder::scale_convert::<u8, f32>().build()
            }
            "blocks_char_to_float" | "convert_s8_f" => {
                TypeConvertersBuilder::scale_convert::<i8, f32>().build()
            }
            "convert_s16_f" => TypeConvertersBuilder::scale_convert::<i16, f32>().build(),
            "blocks_float_to_uchar" | "convert_f_u8" => {
                TypeConvertersBuilder::lossy_scale_convert_f32_u8().build()
            }
            "blocks_float_to_char" | "convert_f_s8" => {
                TypeConvertersBuilder::lossy_scale_convert_f32_i8().build()
            }
            "blocks_float_to_short" | "convert_f_s16" => {
                TypeConvertersBuilder::lossy_scale_convert_f32_i16().build()
            }
            "convert_ff_c" => {
                ApplyNM::<_, _, _, 2, 1>::new(move |v: &[f32], d: &mut [Complex32]| {
                    d[0] = Complex32::new(v[0], v[1])
                })
            }
            "blocks_short_to_float" => TypeConvertersBuilder::scale_convert::<i16, f32>().build(),
            // "blocks_short_to_char" => {
            //     TypeConvertersBuilder::scale_convert::<i16, i8>().build()
            // },
            _ => bail!("Unknown conversion: {blk_id}"),
        };
        let blk = fg.add_block(blk);
        let blk = DefaultPortAdapter::new(blk);
        let blk = Box::new(blk);
        Ok(blk)
    }
}
