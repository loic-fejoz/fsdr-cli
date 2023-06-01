use crate::grc::{BlockInstance, Grc, Metadata, Options, States};
use futuresdr::anyhow::Result;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct GraphLevel {}

#[derive(Clone)]
pub struct BlockLevel {
    block_builder: GrcBlockInstanceBuilder,
}

pub trait GrcBuilderState {}
impl GrcBuilderState for GraphLevel {}
impl GrcBuilderState for BlockLevel {}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GrcItemType {
    U8,
    S8,
    U16,
    S16,
    // S32
    F32,
    F64,
    C32,
    // Complex Integer 64
    // Complex Integer 32
    // Complex Integer 16
    // Complex Integer 8
    InterleavedF32,
}

impl GrcItemType {
    pub fn as_csdr(self) -> &'static str {
        match self {
            Self::U8 => "u8",
            Self::S8 => "s8",
            Self::U16 => "u16",
            Self::S16 => "s16",
            Self::F32 => "f",
            Self::F64 => "f64",
            Self::C32 => "c",
            Self::InterleavedF32 => "ff",
        }
    }

    pub fn as_grc(self) -> &'static str {
        match self {
            Self::U8 => "uchar",
            Self::S8 => "char",
            Self::U16 => "short",
            Self::S16 => "short",
            Self::F32 => "float",
            Self::F64 => "float64",
            Self::C32 => "complex",
            Self::InterleavedF32 => "float",
        }
    }
}

impl From<&str> for GrcItemType {
    fn from(value: &str) -> Self {
        match value {
            "u8" => Self::U8,
            "s8" => Self::S8,
            "u16" => Self::U16,
            "s16" => Self::S16,
            "f" => Self::F32,
            "ff" => Self::InterleavedF32,
            "f32" => Self::F32,
            "c" => Self::C32,
            "c32" => Self::C32,
            "uchar" => Self::U8,
            "byte" => Self::U8,
            "char" => Self::S8,
            "short" => Self::U16,
            "ishort" => Self::S16,
            "float" => Self::F32,
            "float64" => Self::F64,
            "f64" => Self::F64,
            _ => todo!("Unknown GNU Radio type: {value}"),
        }
    }
}

#[derive(Clone)]
pub struct GrcBuilderActualState {
    block_count: usize,
    blocks: Vec<BlockInstance>,
    connections: Vec<[String; 4]>,
    last_output_type: Option<GrcItemType>,
    last_block_name: Option<String>,
}

#[derive(Clone)]
pub struct GrcBuilder<S: GrcBuilderState> {
    state: Box<GrcBuilderActualState>,
    extra: S,
}

impl Default for GrcBuilder<GraphLevel> {
    fn default() -> Self {
        Self::new()
    }
}

impl GrcBuilder<GraphLevel> {
    pub fn new() -> GrcBuilder<GraphLevel> {
        let gl = GraphLevel {};
        let actual_state = GrcBuilderActualState {
            block_count: 0,
            blocks: Vec::<BlockInstance>::new(),
            connections: Vec::<[String; 4]>::new(),
            last_output_type: None,
            last_block_name: None,
        };
        let actual_state = Box::new(actual_state);
        GrcBuilder {
            state: actual_state,
            extra: gl,
        }
    }

    fn push_block(&mut self, block: &mut GrcBlockInstanceBuilder) {
        let block_type = block.kind.clone().expect("block kind");
        let block_name = format!("{}_{}", block_type, self.state.block_count);
        block.with_name(block_name.clone());
        self.state.last_output_type = block.output_type;
        self.state.last_block_name = Some(block_name);
        self.state.block_count += 1;
        self.state.blocks.push(block.build());
        assert_eq!(self.state.block_count, self.state.blocks.len());
    }

    fn push_and_link_block(&mut self, block: &mut GrcBlockInstanceBuilder) {
        let previous_block_name = self.state.last_block_name.clone().expect("");
        self.push_block(block);
        let this_block_name = self.state.last_block_name.clone().expect("");
        self.connect(previous_block_name, "0", this_block_name, "0");
    }

    pub fn ensure_source(&mut self, expected_last_output_type: GrcItemType) -> Self {
        if let Some(last_output_type) = self.state.last_output_type {
            // Ensure proper connectivity
            match (last_output_type, expected_last_output_type) {
                (GrcItemType::F32, GrcItemType::C32) => {
                    let mut convert_ff_c_block = GrcBlockInstanceBuilder::new();
                    convert_ff_c_block
                        .with_block_type("convert_ff_c")
                        .assert_output(GrcItemType::C32);
                    self.push_and_link_block(&mut convert_ff_c_block);
                },
                (GrcItemType::F32, GrcItemType::InterleavedF32) => {}
                _ => {
                    assert_eq!(last_output_type, expected_last_output_type);
                }
            }
        } else {
            let mut src_block = GrcBlockInstanceBuilder::new();
            src_block
                .with_block_type("blocks_file_source")
                .with_parameter("file", "-")
                .with_parameter("type", expected_last_output_type.as_grc())
                .with_parameter("repeat", "false")
                .assert_output(expected_last_output_type);
            self.push_block(&mut src_block);
        }
        (*self).clone()
    }

    pub fn ensure_sink(&mut self) -> &mut Self {
        if let Some(last_output_type) = self.state.last_output_type {
            let mut snk_block = GrcBlockInstanceBuilder::new();
            snk_block
                .with_block_type("blocks_file_sink")
                .with_parameter("file", "-")
                .with_parameter("type", last_output_type.as_grc());
            self.push_and_link_block(&mut snk_block);
            self.state.last_output_type = None;
        }
        self
    }

    pub fn create_block_instance(&self, block_type: impl Into<String>) -> GrcBuilder<BlockLevel> {
        let mut block_builder = GrcBlockInstanceBuilder::new();
        block_builder.with_block_type(block_type);
        let bl = BlockLevel { block_builder };
        GrcBuilder {
            state: self.state.clone(),
            extra: bl,
        }
    }

    pub fn build(&self) -> Result<Grc> {
        let grc = Grc {
            options: Options::default(),
            blocks: self.state.blocks.clone(),
            connections: self.state.connections.clone(),
            metadata: Metadata {
                file_format: 1,
                grc_version: "3.10.3.0".to_string(),
            },
        };
        Ok(grc)
    }

    fn connect(
        &mut self,
        src_name: impl Into<String>,
        src_port_name: impl Into<String>,
        tgt_name: impl Into<String>,
        tgt_port_name: impl Into<String>,
    ) {
        let connection = [
            src_name.into(),
            src_port_name.into(),
            tgt_name.into(),
            tgt_port_name.into(),
        ];
        self.state.connections.push(connection);
    }
}

impl GrcBuilder<BlockLevel> {
    pub fn with_parameter(
        &mut self,
        param_name: impl Into<String>,
        param_value: impl Into<String>,
    ) -> &mut Self {
        self.extra
            .block_builder
            .with_parameter(param_name, param_value);
        self
    }

    pub fn assert_output(&mut self, output_type: GrcItemType) -> &mut Self {
        self.extra.block_builder.assert_output(output_type);
        self
    }

    pub fn push(&self) -> GrcBuilder<GraphLevel> {
        let mut blk_builder = self.extra.block_builder.clone();
        let gl = GraphLevel {};
        let mut grc_builder = GrcBuilder {
            state: self.state.clone(),
            extra: gl,
        };
        grc_builder.push_block(&mut blk_builder);
        grc_builder
    }

    pub fn push_and_link(&self) -> GrcBuilder<GraphLevel> {
        let mut blk_builder = self.extra.block_builder.clone();
        let gl = GraphLevel {};
        let mut grc_builder = GrcBuilder {
            state: self.state.clone(),
            extra: gl,
        };
        grc_builder.push_and_link_block(&mut blk_builder);
        grc_builder
    }
}

#[derive(Clone)]
pub struct GrcBlockInstanceBuilder {
    name: Option<String>,
    kind: Option<String>,
    parameters: BTreeMap<String, String>,
    output_type: Option<GrcItemType>,
}

impl GrcBlockInstanceBuilder {
    pub fn new() -> GrcBlockInstanceBuilder {
        let parameters = BTreeMap::<String, String>::new();
        GrcBlockInstanceBuilder {
            name: None,
            kind: None,
            parameters,
            output_type: None,
        }
    }

    pub fn with_name(&mut self, block_name: impl Into<String>) -> &mut Self {
        self.name = Some(block_name.into());
        self
    }

    pub fn with_block_type(&mut self, block_type: impl Into<String>) -> &mut Self {
        self.kind = Some(block_type.into());
        self
    }

    pub fn with_parameter(
        &mut self,
        param_name: impl Into<String>,
        param_value: impl Into<String>,
    ) -> &mut Self {
        self.parameters
            .insert(param_name.into(), param_value.into());
        self
    }

    pub fn assert_output(&mut self, output_type: GrcItemType) -> &mut Self {
        self.output_type = Some(output_type);
        self
    }

    pub fn build(&self) -> BlockInstance {
        BlockInstance {
            name: self.name.clone().expect("block name"),
            id: self.kind.clone().expect("block type"),
            parameters: self.parameters.clone(),
            states: States::default(),
        }
    }
}

impl Default for GrcBlockInstanceBuilder {
    fn default() -> Self {
        GrcBlockInstanceBuilder::new()
    }
}
