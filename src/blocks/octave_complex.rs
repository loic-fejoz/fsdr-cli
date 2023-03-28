use futuresdr::anyhow::Result;
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
use std::io::Write;

pub struct OctaveComplex {
    samples_to_plot: usize,
    out_of_n_samples: usize,
}

impl OctaveComplex {
    pub fn build(samples_to_plot: usize, out_of_n_samples: usize) -> Block {
        assert!(samples_to_plot <= out_of_n_samples);
        Block::new(
            BlockMetaBuilder::new("OctaveComplex".to_string()).build(),
            StreamIoBuilder::new().add_input::<Complex32>("in").build(),
            MessageIoBuilder::<Self>::new().build(),
            OctaveComplex {
                samples_to_plot,
                out_of_n_samples,
            },
        )
    }
}

#[doc(hidden)]
#[async_trait]
impl Kernel for OctaveComplex {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let i = sio.input(0).slice::<Complex32>();

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
            sio.input(0).consume(self.out_of_n_samples);
        }

        if sio.input(0).finished() && m - self.out_of_n_samples < self.out_of_n_samples {
            io.finished = true;
        }

        Ok(())
    }
}
