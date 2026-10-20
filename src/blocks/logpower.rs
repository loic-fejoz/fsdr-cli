use anyhow::Result;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;

#[derive(Block)]
pub struct LogPowerCf<
    I: CpuBufferReader<Item = Complex32> = DefaultCpuReader<Complex32>,
    O: CpuBufferWriter<Item = f32> = DefaultCpuWriter<f32>,
> {
    add_db: f32,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl LogPowerCf<DefaultCpuReader<Complex32>, DefaultCpuWriter<f32>> {
    pub fn new(add_db: f32) -> Self {
        Self {
            add_db,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for LogPowerCf<I, O>
where
    I: CpuBufferReader<Item = Complex32>,
    O: CpuBufferWriter<Item = f32>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let i = self.input.slice();
        let o = self.output.slice();

        let m = cmp::min(i.len(), o.len());
        if m > 0 {
            let add_db = self.add_db;
            for (s, dst) in i[..m].iter().zip(o[..m].iter_mut()) {
                let p = s.re.mul_add(s.re, s.im.algebraic_mul(s.im));
                *dst = 10.0f32.mul_add(p.log10(), add_db);
            }
            self.input.consume(m);
            self.output.produce(m);
        }

        if !self.input.slice().is_empty() && !self.output.slice().is_empty() {
            io.call_again = true;
        }

        if self.input.slice().is_empty() && self.input.finished() {
            io.finished = true;
        }

        Ok(())
    }
}

#[derive(Block)]
pub struct LogAveragePowerCf<
    I: CpuBufferReader<Item = Complex32> = DefaultCpuReader<Complex32>,
    O: CpuBufferWriter<Item = f32> = DefaultCpuWriter<f32>,
> {
    fft_size: usize,
    avg_number: usize,
    add_db: f32,
    collector: Vec<f32>,
    collected: usize,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl LogAveragePowerCf<DefaultCpuReader<Complex32>, DefaultCpuWriter<f32>> {
    pub fn new(fft_size: usize, avg_number: usize, add_db: f32) -> Self {
        assert!(fft_size > 0, "fft_size must be greater than 0");
        assert!(avg_number > 0, "avg_number must be greater than 0");
        Self {
            fft_size,
            avg_number,
            add_db,
            collector: vec![0.0; fft_size],
            collected: 0,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for LogAveragePowerCf<I, O>
where
    I: CpuBufferReader<Item = Complex32>,
    O: CpuBufferWriter<Item = f32>,
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

        while in_slice.len() - in_offset >= self.fft_size {
            if self.collected == self.avg_number - 1 && out_slice.len() - out_offset < self.fft_size
            {
                break;
            }

            for k in 0..self.fft_size {
                self.collector[k] += in_slice[in_offset + k].norm_sqr();
            }
            in_offset += self.fft_size;
            self.collected += 1;

            if self.collected == self.avg_number {
                let correction = self.add_db - 10.0 * (self.avg_number as f32).log10();
                for k in 0..self.fft_size {
                    out_slice[out_offset + k] = 10.0 * self.collector[k].log10() + correction;
                }
                out_offset += self.fft_size;
                self.collector.fill(0.0);
                self.collected = 0;
            }
        }

        if in_offset > 0 {
            self.input.consume(in_offset);
        }
        if out_offset > 0 {
            self.output.produce(out_offset);
        }

        if self.input.slice().len() >= self.fft_size
            && (self.collected < self.avg_number - 1 || self.output.slice().len() >= self.fft_size)
        {
            io.call_again = true;
        }

        if self.input.finished() && self.input.slice().len() < self.fft_size {
            io.finished = true;
        }

        Ok(())
    }
}
