use anyhow::Result;
use futures::channel::mpsc;
use futures::StreamExt;
use futuresdr::runtime::dev::prelude::*;
use futuresdr::runtime::Pmt;
use std::collections::HashMap;

use crate::cmd_grammar::CommandsParser;
use crate::csdr_cmd::eval_cmd::EvalCmd;

#[derive(Block)]
#[message_inputs(update_var)]
#[message_outputs(out)]
pub struct VariableEvaluator {
    rx: mpsc::UnboundedReceiver<Pmt>,
    tx: mpsc::UnboundedSender<Pmt>,
    variables: HashMap<String, f32>,
    expression: String,
}

impl VariableEvaluator {
    pub fn new(expression: String) -> Self {
        let (tx, rx) = mpsc::unbounded();
        Self {
            rx,
            tx,
            variables: HashMap::new(),
            expression,
        }
    }

    async fn update_var(
        &mut self,
        _io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
        p: Pmt,
    ) -> Result<Pmt> {
        let _ = self.tx.unbounded_send(p);
        Ok(Pmt::Ok)
    }
}

#[doc(hidden)]
impl Kernel for VariableEvaluator {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        match self.rx.next().await {
            Some(Pmt::String(msg)) => {
                if let Some((name, val_str)) = msg.split_once('=') {
                    if let Ok(val) = val_str.parse::<f32>() {
                        self.variables.insert(name.to_string(), val);

                        let eval_result =
                            if let Ok(parsed) = CommandsParser::parse_expr(&*self.expression) {
                                parsed.eval_with(&self.variables).ok()
                            } else {
                                None
                            };

                        if let Some(result) = eval_result {
                            mio.post("out", Pmt::F32(result)).await?;
                        }
                    }
                }
                io.call_again = true;
            }
            Some(_) => {
                io.call_again = true;
            }
            None => {
                io.finished = true;
            }
        }
        Ok(())
    }
}
