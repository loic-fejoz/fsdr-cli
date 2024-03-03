use futuresdr::anyhow::anyhow;
use futuresdr::log::debug;
use futuresdr::num_complex::Complex32;
use iqengine_plugin::server::{
    error::IQEngineError, CustomParamType, FunctionParameters, FunctionParamsBuilder,
    FunctionPostRequest, FunctionPostResponse, SamplesB64Builder,
};
use serde_derive::{Deserialize, Serialize};

use crate::csdr_cmd::CsdrCmd;
use crate::grc::converter;
use crate::grc::converter_helper::{
    ConnectorAdapter, DefaultPortAdapter, MutBlockConverter, PredefinedBlockConverter,
};
use crate::iqengine_blockconverter::IQEngineOutputBlockConverter;
use crate::{cmd_grammar::Rule, cmd_line::HighLevelCmdLine};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDefinedFunctionParams {
    #[serde(rename = "cli")]
    cli: String,
}

pub struct UserDefinedFunction {}

impl UserDefinedFunction {
    fn create_grc(self, cli: String) -> Result<crate::grc::Grc, IQEngineError> {
        let cli = crate::cmd_grammar::CommandsParser::parse_main(&cli);
        if let Err(err) = cli {
            match err.downcast_ref::<pest::error::Error<Rule>>() {
                Some(err) => match &err.variant {
                    pest::error::ErrorVariant::ParsingError {
                        positives: _,
                        negatives: _,
                    } => {
                        return Err(IQEngineError::MandatoryParameter(
                            "parsing error".to_string(),
                        ));
                    }
                    pest::error::ErrorVariant::CustomError { message: _ } => {
                        return Err(IQEngineError::MandatoryParameter("{err}".to_string()));
                    }
                },
                None => {
                    return Err(IQEngineError::MandatoryParameter("{err}".to_string()));
                }
            }
        }
        let cli = cli?;
        if let Some(cli) = cli.as_csdr_cmd() {
            if let Some(cli) = cli.parse()? {
                return Ok(cli);
            }
        }
        Err(IQEngineError::FutureSDRError(anyhow!("ici")))
    }
}

impl iqengine_plugin::server::IQFunction<UserDefinedFunctionParams> for UserDefinedFunction {
    fn parameters(self) -> FunctionParameters {
        FunctionParamsBuilder::new()
            .max_inputs(1)
            .max_outputs(1)
            .custom_param(
                "cli",
                "The flowgraph to apply",
                CustomParamType::String,
                Some(""),
            )
            .build()
    }

    async fn apply(
        self,
        request: FunctionPostRequest<UserDefinedFunctionParams>,
    ) -> Result<FunctionPostResponse, IQEngineError> {
        if let Some(samples_cloud) = request.samples_cloud {
            if !samples_cloud.is_empty() {
                let metadata = samples_cloud
                    .get(0)
                    .expect("need at least one IQ")
                    .sigmf()
                    .await?;
                //metadata.as_object().expect("msg")[""];
                return Err(IQEngineError::NotYetImplemented(
                    "Cloud samples not yet implemented".to_string(),
                ));
            }
        }
        if request.samples_b64.is_none() {
            return Err(IQEngineError::NotYetImplemented(
                "samples in Base64 are mandatory".to_string(),
            ));
        }
        if let Some(samples_b64) = request.samples_b64 {
            let cli = if let Some(prop) = request.custom_params {
                prop.cli
            } else {
                return Err(IQEngineError::MandatoryParameter("cli".to_string()));
            };
            let grc = self.create_grc(cli)?;
            let tmp = fun_name(samples_b64, grc);
            let (fg, cvter) = tmp?;
            let fg = futuresdr::runtime::Runtime::new().run_async(fg).await?;
            let result: FunctionPostResponse = cvter.as_result(fg);
            return Ok(result);
        }
        return Err(IQEngineError::NotYetImplemented(
            "Something wen't wrong".to_string(),
        ));
    }
}

fn fun_name(
    samples_b64: Vec<iqengine_plugin::server::SamplesB64>,
    grc: crate::grc::Grc,
) -> Result<(futuresdr::runtime::Flowgraph, IQEngineOutputBlockConverter), IQEngineError> {
    let stream1 = samples_b64.get(0).unwrap();
    let sample_rate = stream1.sample_rate.unwrap_or(1_800_000.0);
    debug!("sample_rate is {}", sample_rate);
    let mut converter = crate::grc::converter::Grc2FutureSdr::new();

    // Prepare for commands that use input stream
    match stream1.data_type {
        iqengine_plugin::server::DataType::IqSlashCf32Le => {
            let v = stream1.clone().samples_cf32()?;

            let src = futuresdr::blocks::VectorSource::new(v);
            let blk_cvter = PredefinedBlockConverter::new(src);
            converter
                .with_blocktype_conversion("blocks_file_source".to_string(), Box::new(blk_cvter));
        }
        _ => {
            return Err(IQEngineError::UnsupportedDataType(stream1.data_type));
        }
    }

    // Prepare for commands that output stream
    let snk_builder = IQEngineOutputBlockConverter::new();
    let b: Box<(dyn MutBlockConverter + 'static)> = Box::new(snk_builder);
    converter.with_blocktype_conversion("blocks_file_sink", b);

    // Finalize conversion from flow graph description to FutureSDR
    let fg = converter.convert_grc(grc)?;

    // Retrieve the IQEngineOutputBlockConverter to eventually retrieve actual graph result
    let snk_builder = converter.take("blocks_file_sink").expect("msg");
    let snk_builder = *snk_builder.downcast_iqengine().expect("msg");
    Ok((fg, snk_builder))
}

pub const USER_DEFINED_FUNCTION: UserDefinedFunction = UserDefinedFunction {};
