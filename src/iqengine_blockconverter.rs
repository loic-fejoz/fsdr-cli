use crate::grc::builder::GrcItemType;
use crate::grc::converter_helper::{ConnectorAdapter, DefaultPortAdapter, MutBlockConverter};
use crate::grc::BlockInstance;
use anyhow::{bail, Context, Result};
use futuresdr::{blocks::VectorSink, num_complex::Complex32, runtime::Flowgraph};
use iqengine_plugin::server::{FunctionPostResponse, SamplesB64, SamplesB64Builder};
use std::convert::TryInto;

use futuresdr::runtime::BlockId;
use futuresdr::runtime::WrappedKernel;

#[derive(Clone, Copy)]
pub struct IQEngineOutputBlockConverter {
    blk_idx: Option<BlockId>,
    data_type: Option<iqengine_plugin::server::DataType>,
}

impl Default for IQEngineOutputBlockConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl IQEngineOutputBlockConverter {
    pub fn new() -> IQEngineOutputBlockConverter {
        IQEngineOutputBlockConverter {
            blk_idx: None,
            data_type: None,
        }
    }

    pub fn as_result(&self, fg: Flowgraph) -> Result<FunctionPostResponse> {
        let mut result = FunctionPostResponse::new();
        let snk_id = self
            .blk_idx
            .context("iqengine_blockconverter: blk_idx not set")?;
        let output: SamplesB64 = match self.data_type {
            Some(iqengine_plugin::server::DataType::IqSlashCf32Le) => {
                let blk = fg.get_block(snk_id)?;
                let mut blk = blk
                    .try_lock()
                    .context("iqengine_blockconverter: failed to lock block")?;
                let snk = blk
                    .as_any_mut()
                    .downcast_mut::<WrappedKernel<VectorSink<Complex32>>>()
                    .context("iqengine_blockconverter: failed to get VectorSink<Complex32>")?;
                let snk_0 = snk.items();
                SamplesB64Builder::new()
                    .with_samples_cf32(snk_0.clone())
                    .build()
                    .expect("msg")
            }
            Some(iqengine_plugin::server::DataType::ApplicationSlashOctetStream) => {
                let blk = fg.get_block(snk_id)?;
                let mut blk = blk
                    .try_lock()
                    .context("iqengine_blockconverter: failed to lock block")?;
                let snk = blk
                    .as_any_mut()
                    .downcast_mut::<WrappedKernel<VectorSink<u8>>>()
                    .context("iqengine_blockconverter: failed to get VectorSink<u8>")?;
                let snk_0 = snk.items();
                SamplesB64Builder::new()
                    .from_wav_data(snk_0.clone())
                    .build()
                    .expect("msg")
            }
            Some(dt) => bail!("iqengine_blockconverter: Unhandled DataType {:?}", dt),
            None => bail!("iqengine_blockconverter: DataType not set"),
        };
        result.data_output = Some(vec![output]);
        Ok(result)
    }
}

impl MutBlockConverter for IQEngineOutputBlockConverter {
    fn convert(
        &mut self,
        blk: &BlockInstance,
        fg: &mut Flowgraph,
    ) -> Result<Box<dyn ConnectorAdapter>> {
        let filename = blk
            .parameters
            .get("file")
            .context("iqengine_blockconverter: filename must be defined")?;
        let item_type: GrcItemType = blk
            .parameters
            .get("type")
            .context("iqengine_blockconverter: item type must be defined")?
            .try_into()?;
        let blk_id = if "-" == filename {
            match item_type {
                GrcItemType::U8 => {
                    self.data_type =
                        Some(iqengine_plugin::server::DataType::ApplicationSlashOctetStream);
                    fg.add_block(VectorSink::<u8>::new(0)).into()
                }
                GrcItemType::S16 => fg.add_block(VectorSink::<i16>::new(0)).into(),
                GrcItemType::F32 => {
                    self.data_type = Some(iqengine_plugin::server::DataType::AudioSlashWav);
                    fg.add_block(VectorSink::<f32>::new(0)).into()
                }
                GrcItemType::C32 => {
                    self.data_type = Some(iqengine_plugin::server::DataType::IqSlashCf32Le);
                    fg.add_block(VectorSink::<Complex32>::new(0)).into()
                }
                _ => {
                    let item_type_str = item_type.as_csdr();
                    bail!("iqengine_blockconverter: Unhandled FileSink Type {item_type_str}")
                }
            }
        } else {
            bail!("iqengine_blockconverter: Unsupported filename {filename}")
        };
        self.blk_idx = Some(blk_id);
        let s: Box<dyn ConnectorAdapter> = Box::new(DefaultPortAdapter::new(blk_id));
        Ok(s)
    }

    fn downcast_iqengine(&self) -> Option<&IQEngineOutputBlockConverter> {
        Some(self)
    }
}
