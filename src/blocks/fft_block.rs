use anyhow::Result;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;
use rustfft::{Fft, FftPlanner};
use std::cmp;
use std::sync::Arc;

pub fn create_window(window_type: &str, size: usize) -> Vec<f32> {
    if size <= 1 {
        return vec![1.0; size];
    }
    let win_str = window_type.to_uppercase();
    match win_str.as_str() {
        "HAMMING" | "WIN_HAMMING" | "WINDOW.WIN_HAMMING" => (0..size)
            .map(|i| {
                let rate = i as f32 / (size - 1) as f32;
                0.54 - 0.46 * (2.0 * std::f32::consts::PI * rate).cos()
            })
            .collect(),
        "BLACKMAN" | "WIN_BLACKMAN" | "WINDOW.WIN_BLACKMAN" => (0..size)
            .map(|i| {
                let rate = i as f32 / (size - 1) as f32;
                0.42 - 0.5 * (2.0 * std::f32::consts::PI * rate).cos()
                    + 0.08 * (4.0 * std::f32::consts::PI * rate).cos()
            })
            .collect(),
        "HANN" | "HANNING" | "WIN_HANN" | "WINDOW.WIN_HANN" => (0..size)
            .map(|i| {
                let rate = i as f32 / (size - 1) as f32;
                0.5 - 0.5 * (2.0 * std::f32::consts::PI * rate).cos()
            })
            .collect(),
        _ => vec![1.0; size],
    }
}

#[derive(Block)]
pub struct FftCc<
    I: CpuBufferReader<Item = Complex32> = DefaultCpuReader<Complex32>,
    O: CpuBufferWriter<Item = Complex32> = DefaultCpuWriter<Complex32>,
> {
    fft_size: usize,
    every_n_samples: usize,
    window: Vec<f32>,
    plan: Arc<dyn Fft<f32>>,
    scratch: Vec<Complex32>,
    buff: Vec<Complex32>,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl FftCc<DefaultCpuReader<Complex32>, DefaultCpuWriter<Complex32>> {
    pub fn new(fft_size: usize, every_n_samples: usize, window_type: &str) -> Self {
        assert!(
            fft_size > 0 && every_n_samples > 0,
            "fft_size and every_n_samples must be greater than 0"
        );
        let mut planner = FftPlanner::<f32>::new();
        let plan = planner.plan_fft_forward(fft_size);
        let scratch_len = plan.get_outofplace_scratch_len();
        let window = create_window(window_type, fft_size);
        Self {
            fft_size,
            every_n_samples,
            window,
            plan,
            scratch: vec![Complex32::new(0.0, 0.0); scratch_len],
            buff: vec![Complex32::new(0.0, 0.0); fft_size],
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for FftCc<I, O>
where
    I: CpuBufferReader<Item = Complex32>,
    O: CpuBufferWriter<Item = Complex32>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let in_slice = self.input.slice();
        let out_slice = self.output.slice();

        let mut in_offset = 0;
        let mut out_offset = 0;

        let req_in = cmp::max(self.fft_size, self.every_n_samples);

        while in_slice.len() - in_offset >= req_in && out_slice.len() - out_offset >= self.fft_size
        {
            for k in 0..self.fft_size {
                let s = in_slice[in_offset + k];
                let w = self.window[k];
                self.buff[k] = Complex32::new(s.re * w, s.im * w);
            }
            self.plan.process_outofplace_with_scratch(
                &mut self.buff,
                &mut out_slice[out_offset..out_offset + self.fft_size],
                &mut self.scratch,
            );
            in_offset += self.every_n_samples;
            out_offset += self.fft_size;
        }

        if in_offset > 0 {
            self.input.consume(in_offset);
        }
        if out_offset > 0 {
            self.output.produce(out_offset);
        }

        if self.input.slice().len() >= req_in && self.output.slice().len() >= self.fft_size {
            io.call_again = true;
        }

        if self.input.finished() && self.input.slice().len() < req_in {
            io.finished = true;
        }

        Ok(())
    }
}

#[derive(Block)]
pub struct FftFc<
    I: CpuBufferReader<Item = f32> = DefaultCpuReader<f32>,
    O: CpuBufferWriter<Item = Complex32> = DefaultCpuWriter<Complex32>,
> {
    fft_size: usize,
    every_n_samples: usize,
    window: Vec<f32>,
    plan: Arc<dyn Fft<f32>>,
    scratch: Vec<Complex32>,
    buff: Vec<Complex32>,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl FftFc<DefaultCpuReader<f32>, DefaultCpuWriter<Complex32>> {
    pub fn new(fft_size: usize, every_n_samples: usize, window_type: &str) -> Self {
        assert!(
            fft_size > 0 && every_n_samples > 0,
            "fft_size and every_n_samples must be greater than 0"
        );
        let mut planner = FftPlanner::<f32>::new();
        let plan = planner.plan_fft_forward(fft_size);
        let scratch_len = plan.get_outofplace_scratch_len();
        let window = create_window(window_type, fft_size);
        Self {
            fft_size,
            every_n_samples,
            window,
            plan,
            scratch: vec![Complex32::new(0.0, 0.0); scratch_len],
            buff: vec![Complex32::new(0.0, 0.0); fft_size],
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for FftFc<I, O>
where
    I: CpuBufferReader<Item = f32>,
    O: CpuBufferWriter<Item = Complex32>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let in_slice = self.input.slice();
        let out_slice = self.output.slice();

        let mut in_offset = 0;
        let mut out_offset = 0;

        let req_in = cmp::max(self.fft_size, self.every_n_samples);

        while in_slice.len() - in_offset >= req_in && out_slice.len() - out_offset >= self.fft_size
        {
            for k in 0..self.fft_size {
                let s = in_slice[in_offset + k];
                let w = self.window[k];
                self.buff[k] = Complex32::new(s * w, 0.0);
            }
            self.plan.process_outofplace_with_scratch(
                &mut self.buff,
                &mut out_slice[out_offset..out_offset + self.fft_size],
                &mut self.scratch,
            );
            in_offset += self.every_n_samples;
            out_offset += self.fft_size;
        }

        if in_offset > 0 {
            self.input.consume(in_offset);
        }
        if out_offset > 0 {
            self.output.produce(out_offset);
        }

        if self.input.slice().len() >= req_in && self.output.slice().len() >= self.fft_size {
            io.call_again = true;
        }

        if self.input.finished() && self.input.slice().len() < req_in {
            io.finished = true;
        }

        Ok(())
    }
}
