use crate::grc::builder::GrcItemType;
use crate::grc::converter_helper::{ConnectorAdapter, DefaultPortAdapter, MutBlockConverter};
use crate::grc::BlockInstance;
use anyhow::{bail, Context, Result};
use futuresdr::{blocks::VectorSink, num_complex::Complex32, runtime::Flowgraph};
use futuresdr::runtime::{BlockId, BlockRef, TerminatedFlowgraph};
use iqengine_plugin::server::{FunctionPostResponse, SamplesB64, SamplesB64Builder};
use std::convert::TryInto;

#[derive(Clone)]
pub enum IQSinkRef {
    C32(BlockRef<VectorSink<Complex32>>),
    U8(BlockRef<VectorSink<u8>>),
    F32(BlockRef<VectorSink<f32>>),
    S16(BlockRef<VectorSink<i16>>),
}

#[derive(Clone)]
pub struct IQEngineOutputBlockConverter {
    blk_idx: Option<BlockId>,
    sink_ref: Option<IQSinkRef>,
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
            sink_ref: None,
            data_type: None,
        }
    }

    pub fn as_result(&self, fg: TerminatedFlowgraph) -> Result<FunctionPostResponse> {
        let mut result = FunctionPostResponse::new();
        let sink_ref = self
            .sink_ref
            .as_ref()
            .context("iqengine_blockconverter: sink_ref not set")?;

        let output: SamplesB64 = match (self.data_type, sink_ref) {
            (
                Some(iqengine_plugin::server::DataType::IqSlashCf32Le),
                IQSinkRef::C32(snk_ref),
            ) => {
                let snk = fg.block(snk_ref)?;
                let snk_0 = snk.items();
                SamplesB64Builder::new()
                    .with_samples_cf32(snk_0.clone())
                    .build()
                    .expect("msg")
            }
            (
                Some(iqengine_plugin::server::DataType::ApplicationSlashOctetStream),
                IQSinkRef::U8(snk_ref),
            ) => {
                let snk = fg.block(snk_ref)?;
                let snk_0 = snk.items();
                SamplesB64Builder::new()
                    .from_wav_data(snk_0.clone())
                    .build()
                    .expect("msg")
            }
            (Some(dt), _) => bail!("iqengine_blockconverter: Unhandled DataType {:?}", dt),
            (None, _) => bail!("iqengine_blockconverter: DataType not set"),
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
                    let b = fg.add(VectorSink::<u8>::new(0))?;
                    let id = b.id();
                    self.sink_ref = Some(IQSinkRef::U8(b));
                    id
                }
                GrcItemType::S16 => {
                    let b = fg.add(VectorSink::<i16>::new(0))?;
                    let id = b.id();
                    self.sink_ref = Some(IQSinkRef::S16(b));
                    id
                }
                GrcItemType::F32 => {
                    self.data_type = Some(iqengine_plugin::server::DataType::AudioSlashWav);
                    let b = fg.add(VectorSink::<f32>::new(0))?;
                    let id = b.id();
                    self.sink_ref = Some(IQSinkRef::F32(b));
                    id
                }
                GrcItemType::C32 => {
                    self.data_type = Some(iqengine_plugin::server::DataType::IqSlashCf32Le);
                    let b = fg.add(VectorSink::<Complex32>::new(0))?;
                    let id = b.id();
                    self.sink_ref = Some(IQSinkRef::C32(b));
                    id
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
