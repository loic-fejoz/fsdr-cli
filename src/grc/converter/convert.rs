use super::super::converter_helper::{BlockConverter, ConnectorAdapter, DefaultPortAdapter};
use super::BlockInstance;
use crate::grc::backend::FsdrBackend;
use anyhow::{bail, Result};
use fsdr_blocks::type_converters::TypeConvertersBuilder;

pub struct ConvertBlockConverter {}

impl<B: FsdrBackend> BlockConverter<B> for ConvertBlockConverter {
    fn convert(
        &self,
        blk: &BlockInstance,
        backend: &mut B,
    ) -> Result<Box<dyn ConnectorAdapter<B::BlockRef>>> {
        let blk_id = &blk.id[..];
        let blk_ref = match blk_id {
            "blocks_uchar_to_float" | "convert_u8_f" => backend.add_uchar_to_float()?,
            "blocks_char_to_float" | "convert_s8_f" => {
                let blk = TypeConvertersBuilder::scale_convert::<i8, f32>().build();
                backend.add_block_runtime(blk)?
            }
            "convert_s16_f" => {
                let blk = TypeConvertersBuilder::scale_convert::<i16, f32>().build();
                backend.add_block_runtime(blk)?
            }
            "blocks_float_to_uchar" | "convert_f_u8" => {
                let blk = TypeConvertersBuilder::lossy_scale_convert_f32_u8().build();
                backend.add_block_runtime(blk)?
            }
            "blocks_float_to_char" | "convert_f_s8" => {
                let blk = TypeConvertersBuilder::lossy_scale_convert_f32_i8().build();
                backend.add_block_runtime(blk)?
            }
            "blocks_float_to_short" | "convert_f_s16" => backend.add_f32_to_s16()?,
            "convert_ff_c" | "blocks_float_to_complex" => backend.add_float_to_complex()?,
            "blocks_short_to_float" => {
                let blk = TypeConvertersBuilder::scale_convert::<i16, f32>().build();
                backend.add_block_runtime(blk)?
            }
            _ => bail!("Unknown conversion: {blk_id}"),
        };
        Ok(Box::new(DefaultPortAdapter::new(blk_ref)))
    }
}
