use anyhow::Result;
use futuresdr::num_complex::Complex32;
use futuresdr::prelude::*;
use std::io::Write;

#[derive(Block)]
pub struct OctaveComplex<I: CpuBufferReader<Item = Complex32> = DefaultCpuReader<Complex32>> {
    samples_to_plot: usize,
    out_of_n_samples: usize,
    #[input]
    input: I,
}

impl<I: CpuBufferReader<Item = Complex32>> OctaveComplex<I> {
    pub fn new(samples_to_plot: usize, out_of_n_samples: usize) -> Self {
        assert!(samples_to_plot <= out_of_n_samples);
        Self {
            samples_to_plot,
            out_of_n_samples,
            input: I::default(),
        }
    }
}

#[doc(hidden)]
impl<I: CpuBufferReader<Item = Complex32>> Kernel for OctaveComplex<I> {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i = self.input.slice();

        let m = i.len();
        if m >= self.out_of_n_samples {
            let mut stdout = std::io::stdout();
            let samples_to_plot = self.samples_to_plot;
            print!("N = {samples_to_plot};\nisig = [");
            for c in i.iter().take(samples_to_plot) {
                print!("{} ", c.re);
            }
            print!("];\nqsig = [");
            for c in i.iter().take(samples_to_plot) {
                print!("{} ", c.im);
            }
            println!("];\nzsig = [0:N-1];");
            println!("plot3(isig,zsig,qsig);");
            stdout.flush().expect("flush error on stdout");
            self.input.consume(self.out_of_n_samples);
        }

        if self.input.finished() && m - self.out_of_n_samples < self.out_of_n_samples {
            io.finished = true;
        }

        Ok(())
    }
}
