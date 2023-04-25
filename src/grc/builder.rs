use crate::grc::{BlockInstance, Grc, States, Options, Metadata};
use futuresdr::anyhow::{bail, Result};
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
    F32,
    C32,
}

impl GrcItemType {
    pub fn as_grc(self) -> &'static str {
        match self {
            Self::F32 => "float",
            Self::C32 => "complex",
        }
    }
}

#[derive(Clone)]
pub struct GrcBuilderActualState {
    block_count: usize,
    blocks: Vec<BlockInstance>,
    connections: Vec<Vec<String>>,
    last_output_type: Option<GrcItemType>,
    last_block_name: Option<String>,
}

#[derive(Clone)]
pub struct GrcBuilder<S: GrcBuilderState> {
    state: Box<GrcBuilderActualState>,
    extra: S,
}

impl GrcBuilder<GraphLevel> {
    pub fn new() -> GrcBuilder<GraphLevel> {
        let gl = GraphLevel {};
        let actual_state = GrcBuilderActualState {
            block_count: 0,
            blocks: Vec::<BlockInstance>::new(),
            connections: Vec::<Vec<String>>::new(),
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
        let block_type =  block.kind.clone().expect("block kind");
        let block_name = format!("{}_{}", block_type, self.state.block_count);
        block.with_name(block_name.clone());
        self.state.last_output_type = block.output_type;
        self.state.last_block_name = Some(block_name);
        self.state.block_count += 1;
        self.state.blocks.push(block.build());
        assert_eq!(self.state.block_count, self.state.blocks.len());
    }

    pub fn ensure_source(&mut self, expected_last_output_type: GrcItemType) -> Self {
        if let Some(last_output_type) = self.state.last_output_type {
            assert_eq!(last_output_type, expected_last_output_type);
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
            self.push_block(&mut snk_block);
            self.state.last_output_type = None;
        }
        self
    }

    pub fn create_block_instance(
        &self,
        block_type: impl Into<String>,
    ) -> GrcBuilder<BlockLevel> {
        let mut block_builder = GrcBlockInstanceBuilder::new();
        block_builder.with_block_type(block_type);
        let bl = BlockLevel {
            block_builder
        };
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
        src_port_name:  impl Into<String>,
        tgt_name:  impl Into<String>,
        tgt_port_name:  impl Into<String>,
    ) {
        let connection = vec![
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
        let previous_block_name = self.state.last_block_name.clone().expect("");
        let mut grc = self.push();
        let this_block_name = grc.state.last_block_name.clone().expect("");
        grc.connect(previous_block_name, "0", this_block_name, "0");
        grc
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
            name: self.name.clone().expect("block name").clone(),
            id: self.kind.clone().expect("block type").clone(),
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
