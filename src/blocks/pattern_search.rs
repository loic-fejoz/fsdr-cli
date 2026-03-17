use anyhow::Result;
use futuresdr::prelude::*;
use std::collections::VecDeque;

#[derive(Debug)]
enum State {
    Dumping(usize),            // Number of items to dump after a match
    Searching(VecDeque<bool>), // Current active states of the underlying search NFA
}

#[derive(Block)]
pub struct PatternSearch<A>
where
    A: Copy + Send + Sync + 'static + Default + std::fmt::Debug + PartialEq,
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
    A: Copy + Send + Sync + 'static + Default + std::fmt::Debug + PartialEq,
{
    fn empty_active_states(capacity: usize) -> VecDeque<bool> {
        let mut active_states = VecDeque::with_capacity(capacity);
        for _ in 0..capacity {
            active_states.push_back(false);
        }
        active_states
    }

    pub fn new(values_after: usize, pattern_values: Vec<A>) -> Self {
        let active_states = Self::empty_active_states(pattern_values.len());
        Self {
            _item_type: std::marker::PhantomData,
            values_after,
            pattern_values,
            current_state: State::Searching(active_states),
            input: Default::default(),
            output: Default::default(),
        }
    }
}

#[doc(hidden)]
impl<A> Kernel for PatternSearch<A>
where
    A: Copy + Send + Sync + 'static + Default + std::fmt::Debug + PartialEq,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let mut consumed = 0;
        let mut produced = 0;
        let i_len;
        let mut next_state = None;

        let values_after = self.values_after;
        let pattern_len = self.pattern_values.len();

        {
            let i = self.input.slice();
            let o = self.output.slice();
            i_len = i.len();

            match &mut self.current_state {
                State::Dumping(nb) => {
                    let n = (*nb).min(i.len()).min(o.len());
                    if n > 0 {
                        o[..n].copy_from_slice(&i[..n]);
                        consumed = n;
                        produced = n;
                        *nb -= n;
                    }
                    if *nb == 0 {
                        next_state = Some(State::Searching(Self::empty_active_states(pattern_len)));
                    }
                }
                State::Searching(active_states) => {
                    let mut found_at = None;
                    for (idx, &input) in i.iter().enumerate() {
                        let mut prev = true;
                        for (state, &expected) in
                            active_states.iter_mut().zip(self.pattern_values.iter())
                        {
                            let current = *state;
                            *state = prev && (input == expected);
                            prev = current;
                        }

                        if *active_states.back().unwrap_or(&false) {
                            found_at = Some(idx);
                            break;
                        }
                    }

                    if let Some(idx) = found_at {
                        consumed = idx + 1;
                        next_state = Some(State::Dumping(values_after));
                    } else {
                        consumed = i.len();
                    }
                }
            }
        }

        if let Some(s) = next_state {
            self.current_state = s;
            io.call_again = true;
        }

        if consumed > 0 {
            self.input.consume(consumed);
        }
        if produced > 0 {
            self.output.produce(produced);
        }

        if self.input.finished() && consumed == i_len {
            io.finished = true;
        }

        Ok(())
    }
}
