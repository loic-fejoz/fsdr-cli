use futuresdr::anyhow::Result;
use futuresdr::runtime::Block;
use futuresdr::runtime::BlockMeta;
use futuresdr::runtime::BlockMetaBuilder;
use futuresdr::runtime::Kernel;
use futuresdr::runtime::MessageIo;
use futuresdr::runtime::MessageIoBuilder;
use futuresdr::runtime::StreamIo;
use futuresdr::runtime::StreamIoBuilder;
use futuresdr::runtime::WorkIo;

pub struct DCBlocker<A>
where
    A: Send + 'static,
{
    last_dc_level: A,
    min_bufsize: usize,
}

impl<A> DCBlocker<A>
where
    A: Send + 'static + Default,
    DCBlocker<A>: Kernel
{
    pub fn new(min_bufsize: usize) -> Block {
        Block::new(
            BlockMetaBuilder::new(format!("DCBlocker")).build(),
            StreamIoBuilder::new()
                .add_input::<A>("in")
                .add_output::<A>("out")
                .build(),
            MessageIoBuilder::<Self>::new().build(),
            DCBlocker {
                last_dc_level: A::default(),
                min_bufsize,
            },
        )
    }
}

#[doc(hidden)]
#[async_trait]
impl Kernel for DCBlocker<f32>
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i = sio.input(0).slice::<f32>();
        let o = sio.output(0).slice::<f32>();

        let m = std::cmp::min(i.len(), o.len());
        if m > self.min_bufsize {
            let sum: f32 = i.iter().sum();
            let avg = sum / (i.len() as f32);
            println!("average is {avg:?}");
            let avgdiff = avg - self.last_dc_level;

            let input_size = m as f32;
            for (i, (v, r)) in i.iter().zip(o.iter_mut()).enumerate() {
                let linear_dc_level_change = avgdiff * ((i as f32) / input_size);
                let dc_removal_level = self.last_dc_level + linear_dc_level_change;
                *r = *v - dc_removal_level;
            }

            self.last_dc_level = avg;

            sio.input(0).consume(m);
            sio.output(0).produce(m);
        }

        if sio.input(0).finished() && m == i.len() {
            io.finished = true;
        }

        Ok(())
    }
}
