use std::collections::VecDeque;
use anyhow::{Result, Context};
use futuresdr::prelude::*;

enum State {
    DUMPING(usize),            // Reminder
    SEARCHING(VecDeque<bool>), // Current active states of the underlying search non-deterministic automata
}

#[derive(Block)]
pub struct PatternSearch<A>
where
    A: Copy + Send + Sync + 'static + Default + std::fmt::Debug,
{
    _item_type: std::marker::PhantomData<A>,

    values_after: usize,
    pattern_values: Vec<A>,

    current_state: State,

    #[input]
    input: DefaultCpuReader<A>,
    #[output]
    output: DefaultCpuWriter<A>,
}

impl<A> PatternSearch<A>
where
    A: Copy + Send + Sync + 'static + Default + std::fmt::Debug,
{
    fn empty_active_states(capacity: usize) -> VecDeque<bool> {
        let mut active_states = VecDeque::with_capacity(capacity);
        for _ in 0..capacity {
            active_states.push_back(false);
        }
        active_states.make_contiguous();
        active_states
    }

    pub fn new(values_after: usize, pattern_values: Vec<A>) -> Self {
        let active_states = PatternSearch::<A>::empty_active_states(pattern_values.len());
        Self {
            _item_type: std::marker::PhantomData,
            values_after,
            pattern_values,
            current_state: State::SEARCHING(active_states),
            input: Default::default(),
            output: Default::default(),
        }
    }
}

#[doc(hidden)]
impl Kernel for PatternSearch<u8> {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i_len: usize;
        let mut m = 0;
        let mut out_len = 0;

        {
            let i = self.input.slice();
            let o = self.output.slice();

            i_len = i.len();
            self.current_state = match &mut self.current_state {
                State::DUMPING(nb) => {
                    let nb = *nb;
                    let mut counter = 0usize;
                    for (x, y) in i.iter().zip(o).take(nb) {
                        *y = *x;
                        counter += 1;
                    }
                    m = counter;
                    out_len = counter;
                    if counter == nb {
                        State::SEARCHING(PatternSearch::<u8>::empty_active_states(
                            self.pattern_values.len(),
                        ))
                    } else {
                        State::DUMPING(nb - counter)
                    }
                }
                State::SEARCHING(potential_idx) => {
                    let mut potential_idx: VecDeque<bool> = potential_idx.clone();
                    let mut next_state = State::SEARCHING(potential_idx.clone());
                    for input in i.iter() {
                        m += 1;
                        potential_idx.push_front(true);
                        
                        potential_idx = potential_idx.make_contiguous()
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
                            next_state = State::SEARCHING(potential_idx.clone())
                        }
                    }
                    next_state
                }
            };
        }

        if m > 0 {
            self.input.consume(m);
        }
        if out_len > 0 {
            self.output.produce(out_len);
        }

        if self.input.finished() && m == i_len {
            io.finished = true;
        }

        Ok(())
    }
}
