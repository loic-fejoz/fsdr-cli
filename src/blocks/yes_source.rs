use anyhow::Result;
use futuresdr::runtime::dev::prelude::*;

#[derive(Block)]
pub struct YesF<O: CpuBufferWriter<Item = f32> = DefaultCpuWriter<f32>> {
    value: f32,
    #[output]
    output: O,
}

impl YesF<DefaultCpuWriter<f32>> {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for YesF<DefaultCpuWriter<f32>> {
    fn default() -> Self {
        Self::new(1.0)
    }
}

#[doc(hidden)]
impl<O> Kernel for YesF<O>
where
    O: CpuBufferWriter<Item = f32>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let o = self.output.slice();
        let n = o.len();
        if n > 0 {
            o.fill(self.value);
            self.output.produce(n);
        }

        if !self.output.slice().is_empty() {
            io.call_again = true;
        }

        Ok(())
    }
}
