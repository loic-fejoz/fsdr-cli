use anyhow::Result;
use futuresdr::num_complex::Complex32;
use futuresdr::prelude::*;

#[derive(Clone, Debug)]
pub enum TimingAlgorithm {
    GARDNER,
    EARLYLATE,
}

#[derive(Block)]
pub struct TimingRecovery<A>
where
    A: Copy + Send + Sync + 'static + Default + std::fmt::Debug,
{
    _item_type: std::marker::PhantomData<A>,

    algo: TimingAlgorithm,

    decimation_rate: usize,
    mu: f32, // loop_gain
    max_error: f32,

    // use_q: bool,
    // debug_phase: usize,
    // debug_every_nth: usize,
    // char* debug_writefiles_path,
    last_correction_offset: isize,
    earlylate_ratio: f32,

    #[input]
    input: DefaultCpuReader<A>,
    #[output]
    output: DefaultCpuWriter<A>,
}

impl<A> TimingRecovery<A>
where
    A: Copy + Send + Sync + 'static + Default + std::fmt::Debug,
{
    pub fn new(algo: TimingAlgorithm, decimation_rate: usize, mu: f32, max_error: f32) -> Self {
        TimingRecovery::<A> {
            algo,
            decimation_rate,
            mu,
            max_error,
            _item_type: std::marker::PhantomData,
            // use_q: todo!(),
            // debug_phase: todo!(),
            // debug_every_nth: todo!(),
            last_correction_offset: 0,
            earlylate_ratio: 0.25f32,
            input: Default::default(),
            output: Default::default(),
        }
    }
}

#[doc(hidden)]
impl Kernel for TimingRecovery<Complex32> {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i_len: usize;
        let m: usize;
        let mut current_bitstart_index = 0usize;
        let mut correction_offset = self.last_correction_offset;
        let mut oindex = 0usize;

        {
            let i = self.input.slice();
            let o = self.output.slice();

            i_len = i.len();
            m = std::cmp::min(i.len(), self.decimation_rate * o.len());

            let num_samples_bit = self.decimation_rate;
            let num_samples_halfbit = self.decimation_rate / 2;
            let num_samples_quarterbit = self.decimation_rate / 4;
            let num_samples_earlylate_wing = ((num_samples_bit as f32) * self.earlylate_ratio) as usize;

            loop {
                if current_bitstart_index + (num_samples_halfbit as usize) * 3 >= m {
                    break;
                }

                correction_offset =
                    if correction_offset.abs() >= (0.9 * (num_samples_quarterbit as f32)) as isize {
                        0isize
                    } else {
                        correction_offset
                    };

                let error = match self.algo {
                    TimingAlgorithm::GARDNER => {
                        let el_point_right_index = current_bitstart_index + num_samples_halfbit * 3;
                        let el_point_left_index = current_bitstart_index + num_samples_halfbit * 1;
                        let el_point_mid_index = current_bitstart_index + num_samples_halfbit * 2;
                        o[oindex] = i[el_point_mid_index];
                        oindex += 1;

                        i[el_point_right_index].re
                            - i[el_point_left_index].re * i[el_point_mid_index].re
                    }
                    TimingAlgorithm::EARLYLATE => {
                        let el_point_right_index =
                            current_bitstart_index + num_samples_earlylate_wing * 3;
                        let el_point_left_index =
                            ((current_bitstart_index + num_samples_earlylate_wing * 1) as isize
                                - correction_offset) as usize;
                        let el_point_mid_index = current_bitstart_index + num_samples_halfbit;
                        o[oindex] = i[el_point_mid_index];
                        oindex += 1;
                        i[el_point_right_index].re
                            - i[el_point_left_index].re * i[el_point_mid_index].re
                    }
                };
                let error = error.min(self.max_error).max(-self.max_error);
                let error_sign = match self.algo {
                    TimingAlgorithm::EARLYLATE => 1isize,
                    TimingAlgorithm::GARDNER => -1,
                };
                correction_offset =
                    (((num_samples_halfbit as isize) * error_sign) as f32 * self.mu * error) as isize;
                current_bitstart_index =
                    ((current_bitstart_index + num_samples_bit) as isize + correction_offset) as usize;
            }
        }
        self.last_correction_offset = correction_offset;
        self.input.consume(current_bitstart_index);
        self.output.produce(oindex);

        if self.input.finished() && m == i_len {
            io.finished = true;
        }

        Ok(())
    }
}
