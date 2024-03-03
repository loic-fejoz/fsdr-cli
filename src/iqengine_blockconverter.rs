use futuresdr::{blocks::VectorSink, num_complex::Complex32, runtime::Flowgraph};
use iqengine_plugin::server::{FunctionPostResponse, SamplesB64, SamplesB64Builder};

use crate::grc::converter_helper::{ConnectorAdapter, DefaultPortAdapter, MutBlockConverter};

#[derive(Clone, Copy)]
pub struct IQEngineOutputBlockConverter {
    blk_idx: Option<usize>,
    data_type: Option<iqengine_plugin::server::DataType>,
}

impl IQEngineOutputBlockConverter {
    pub fn new() -> IQEngineOutputBlockConverter {
        IQEngineOutputBlockConverter{
            blk_idx: None,
            data_type: None,
        }
    }

    pub fn as_result(&self, fg: Flowgraph) -> FunctionPostResponse {
        let mut result = FunctionPostResponse::new();
        let output: SamplesB64 = match self.data_type {
            Some(iqengine_plugin::server::DataType::IqSlashCf32Le) => {
                let snk_0 = fg.kernel::<VectorSink<Complex32>>(self.blk_idx.expect("msg")).unwrap();
                let snk_0 = snk_0.items();
                SamplesB64Builder::new()
                    .with_samples_cf32(snk_0.clone())
                    .build().expect("msg")
            },
            Some(_) => todo!(),
            None => todo!(),
        };
        result.data_output = Some(vec![output]);
        result
    }
}

impl MutBlockConverter for IQEngineOutputBlockConverter {
    fn convert(&mut self, blk: &crate::grc::BlockInstance, fg: &mut futuresdr::runtime::Flowgraph)
        -> futuresdr::anyhow::Result<Box<dyn crate::grc::converter_helper::ConnectorAdapter>> {
        let filename = blk
            .parameters
            .get("file")
            .expect("filename must be defined");
        let item_type = blk
            .parameters
            .get("type")
            .expect("item type must be defined");
        let blk = if "-" == filename {
            match &(item_type[..]) {
                "u8" => VectorSink::<u8>::new(0),
                "i16" | "ishort" | "short" => VectorSink::<i16>::new(0),
                "f32" | "float" => {
                    self.data_type = Some(iqengine_plugin::server::DataType::AudioSlashWav);
                    VectorSink::<f32>::new(0)
                },
                "c32" | "complex" => {
                    self.data_type = Some(iqengine_plugin::server::DataType::IqSlashCf32Le);
                    VectorSink::<Complex32>::new(0)
                },
                _ => todo!("Unhandled FileSink Type {item_type}"),
            }
        } else {
            todo!("bizarre")
        };
        let blk = fg.add_block(blk);
        self.blk_idx = Some(blk);
        let s: Box<dyn ConnectorAdapter> = Box::new(DefaultPortAdapter::new(blk));
        Ok(s)
    }

    fn downcast_iqengine(&self) -> Option<&crate::iqengine_blockconverter::IQEngineOutputBlockConverter> {
        Some(self)
    }
}