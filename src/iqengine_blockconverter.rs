use crate::grc::builder::GrcItemType;
use crate::grc::converter_helper::{ConnectorAdapter, DefaultPortAdapter, MutBlockConverter};
use crate::grc::BlockInstance;
use fsdr_blocks::futuresdr::anyhow::{bail, Context, Result};
use fsdr_blocks::futuresdr::{blocks::VectorSink, num_complex::Complex32, runtime::Flowgraph};
use iqengine_plugin::server::{FunctionPostResponse, SamplesB64, SamplesB64Builder};
use std::convert::TryInto;

#[derive(Clone, Copy)]
pub struct IQEngineOutputBlockConverter {
    blk_idx: Option<usize>,
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
        let output: SamplesB64 = match self.data_type {
            Some(iqengine_plugin::server::DataType::IqSlashCf32Le) => {
                let snk_idx = self.blk_idx.context("iqengine_blockconverter: blk_idx not set")?;
                let snk_0 = fg
                    .kernel::<VectorSink<Complex32>>(snk_idx)
                    .context("iqengine_blockconverter: failed to get VectorSink<Complex32>")?;
                let snk_0 = snk_0.items();
                SamplesB64Builder::new()
                    .with_samples_cf32(snk_0.clone())
                    .build()
                    .expect("msg")
            }
            Some(iqengine_plugin::server::DataType::ApplicationSlashOctetStream) => {
                let snk_idx = self.blk_idx.context("iqengine_blockconverter: blk_idx not set")?;
                let snk_0 = fg
                    .kernel::<VectorSink<u8>>(snk_idx)
                    .context("iqengine_blockconverter: failed to get VectorSink<u8>")?;
                let snk_0 = snk_0.items();
                SamplesB64Builder::new()
                    .from_u8_data(snk_0.clone())
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
        let blk = if "-" == filename {
            match item_type {
                GrcItemType::U8 => {
                    self.data_type =
                        Some(iqengine_plugin::server::DataType::ApplicationSlashOctetStream);
                    VectorSink::<u8>::new(0)
                }
                GrcItemType::S16 => VectorSink::<i16>::new(0),
                GrcItemType::F32 => {
                    self.data_type = Some(iqengine_plugin::server::DataType::AudioSlashWav);
                    VectorSink::<f32>::new(0)
                }
                GrcItemType::C32 => {
                    self.data_type = Some(iqengine_plugin::server::DataType::IqSlashCf32Le);
                    VectorSink::<Complex32>::new(0)
                }
                _ => {
                    let item_type_str = item_type.as_csdr();
                    bail!("iqengine_blockconverter: Unhandled FileSink Type {item_type_str}")
                }
            }
        } else {
            bail!("iqengine_blockconverter: Unsupported filename {filename}")
        };
        let blk = fg.add_block(blk);
        self.blk_idx = Some(blk);
        let s: Box<dyn ConnectorAdapter> = Box::new(DefaultPortAdapter::new(blk));
        Ok(s)
    }

    fn downcast_iqengine(
        &self,
    ) -> Option<&crate::iqengine_blockconverter::IQEngineOutputBlockConverter> {
        Some(self)
    }
}
