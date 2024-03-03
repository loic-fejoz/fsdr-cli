use std::vec;

use futuresdr::anyhow::Result;
use futuresdr::log::debug;
use futuresdr::macros::async_trait;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Block;
use futuresdr::runtime::BlockMeta;
use futuresdr::runtime::BlockMetaBuilder;
use futuresdr::runtime::Kernel;
use futuresdr::runtime::MessageIo;
use futuresdr::runtime::MessageIoBuilder;
use futuresdr::runtime::StreamIo;
use futuresdr::runtime::StreamIoBuilder;
use futuresdr::runtime::WorkIo;

enum State {
    DUMPING(usize), // Reminder
    SEARCHING(Vec<usize>),
}

pub struct PatternSearch<A>
where
    A: Send + 'static,
{
    _item_type: std::marker::PhantomData<A>,

    values_after: usize,
    pattern_values: Vec<A>,

    current_state: State,
}

impl<A> PatternSearch<A>
where
    A: Send + 'static + Default,
    PatternSearch<A>: Kernel,
{
    pub fn build(values_after: usize, pattern_values: Vec<A>) -> Block {
        Block::new(
            BlockMetaBuilder::new("PatternSearch".to_string()).build(),
            StreamIoBuilder::new()
                .add_input::<A>("in")
                .add_output::<A>("out")
                .build(),
            MessageIoBuilder::<Self>::new().build(),
            PatternSearch::<A> {
                _item_type: std::marker::PhantomData,
                values_after,
                pattern_values,
                current_state: State::SEARCHING(vec![]),
            },
        )
    }
}

#[doc(hidden)]
#[async_trait]
impl Kernel for PatternSearch<u8> {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i = sio.input(0).slice::<u8>();
        let o = sio.output(0).slice::<u8>();

        let mut m = 0;
        self.current_state = match self.current_state {
            State::DUMPING(nb) => {
                let mut counter = 0usize;
                for (x, y) in i.iter().zip(o).take(nb) {
                    *y = *x;
                    counter += 1;
                }
                sio.input(0).consume(counter);
                sio.output(0).produce(counter);
                m = counter;
                if counter == nb {
                    State::SEARCHING(vec![])
                } else {
                    State::DUMPING(nb - counter)
                }
            }
            State::SEARCHING(potential_idx) => State::SEARCHING(potential_idx),
        };

        if sio.input(0).finished() && m == i.len() {
            io.finished = true;
        }

        Ok(())
    }
}
