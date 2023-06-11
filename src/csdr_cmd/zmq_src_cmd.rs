use super::cmd::CsdrCmd;
use crate::cmd_grammar::Rule;
use crate::grc::builder::{GraphLevel, GrcBuilder, GrcItemType};
use futuresdr::anyhow::Result;
use pest::iterators::Pair;

pub trait ZmqSrcCmd<'i> {
    fn address(&self) -> Result<&str>;
    fn input_type(&self) -> Result<GrcItemType>;

    fn build_zmq_src(&self, grc: GrcBuilder<GraphLevel>) -> Result<GrcBuilder<GraphLevel>> {
        let mut grc = grc;
        let address = self.address()?;
        let item_type = self.input_type()?;
        grc = grc
            .create_block_instance("zeromq_sub_source")
            .with_parameter("address", address)
            .with_parameter("hwm", "-1")
            .with_parameter("pass_tags", "False")
            .with_parameter("timeout", "100")
            .with_parameter("vlen", "1")
            .with_parameter("type", item_type.as_grc())
            .assert_output(item_type)
            .push();
        Ok(grc)
    }
}

impl<'i> ZmqSrcCmd<'i> for Pair<'i, Rule> {
    fn address(&self) -> Result<&'i str> {
        CsdrCmd::arg::<2>(self, "missing mandatory <address> parameter for zmq_src_XX")
    }

    fn input_type(&self) -> Result<GrcItemType> {
        CsdrCmd::type_arg::<1>(
            self,
            "missing mandatory <item type> parameter for zmq_src_XX",
        )
    }
}
