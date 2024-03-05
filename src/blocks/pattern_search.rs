use std::collections::VecDeque;

use futuresdr::anyhow::Result;
use futuresdr::macros::async_trait;
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
    DUMPING(usize),            // Reminder
    SEARCHING(VecDeque<bool>), // Current active state of the underlying search automata
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
    fn empty_active_states(capacity: usize) -> VecDeque<bool> {
        let mut active_states = VecDeque::with_capacity(capacity);
        for _ in 0..capacity {
            active_states.push_back(false);
        }
        active_states.make_contiguous();
        active_states
    }

    pub fn build(values_after: usize, pattern_values: Vec<A>) -> Block {
        let active_states = PatternSearch::empty_active_states(pattern_values.len());
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
                current_state: State::SEARCHING(active_states),
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
        self.current_state = match &mut self.current_state {
            State::DUMPING(nb) => {
                let nb = *nb;
                let mut counter = 0usize;
                for (x, y) in i.iter().zip(o).take(nb) {
                    *y = *x;
                    counter += 1;
                }
                sio.input(0).consume(counter);
                sio.output(0).produce(counter);
                m = counter;
                if counter == nb {
                    State::SEARCHING(PatternSearch::empty_active_states(
                        self.pattern_values.len(),
                    ))
                } else {
                    State::DUMPING(nb - counter)
                }
            }
            State::SEARCHING(potential_idx) => {
                let mut next_state = State::SEARCHING(potential_idx.clone());
                for input in i.iter() {
                    m += 1;
                    potential_idx.push_front(*input == self.pattern_values[0]);
                    let potential_idx: VecDeque<bool> = potential_idx
                        .iter()
                        .zip(self.pattern_values.iter())
                        .map(|(previous, expected_value)|
                            (*expected_value == *input) & previous
                        )
                        .collect();
                    assert!(!potential_idx.is_empty());
                    let is_last_state_active = *potential_idx.iter().last().expect("");
                    if is_last_state_active {
                        next_state = State::DUMPING(self.values_after);
                        break;
                    } else {
                        next_state = State::SEARCHING(potential_idx)
                    }
                }
                sio.input(0).consume(m);
                sio.output(0).produce(0);
                next_state
            }
        };
        // todo!("Make it more efficient by looping immediately if still have input")

        if sio.input(0).finished() && m == i.len() {
            io.finished = true;
        }

        Ok(())
    }
}
