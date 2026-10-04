use anyhow::Result;
use fsdr_cli::blocks::{AfcCc, AfcFf};
use futuresdr::blocks::VectorSource;
use futuresdr::num_complex::Complex32;
use futuresdr::prelude::connect;
use futuresdr::runtime::dev::prelude::*;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;
use std::f32::consts::PI;

#[derive(Block)]
struct FloatCollectorSink {
    #[input]
    input: DefaultCpuReader<f32>,
    items: Vec<f32>,
}

impl FloatCollectorSink {
    fn new() -> Self {
        Self {
            input: DefaultCpuReader::default(),
            items: Vec::new(),
        }
    }
}

#[doc(hidden)]
impl Kernel for FloatCollectorSink {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mo: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let (i, _tags) = self.input.slice_with_tags();
        let m = i.len();
        if m > 0 {
            self.items.extend_from_slice(i);
            self.input.consume(m);
        }
        if self.input.finished() {
            io.finished = true;
        }
        Ok(())
    }
}

#[derive(Block)]
struct ComplexCollectorSink {
    #[input]
    input: DefaultCpuReader<Complex32>,
    items: Vec<Complex32>,
}

impl ComplexCollectorSink {
    fn new() -> Self {
        Self {
            input: DefaultCpuReader::default(),
            items: Vec::new(),
        }
    }
}

#[doc(hidden)]
impl Kernel for ComplexCollectorSink {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mo: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let (i, _tags) = self.input.slice_with_tags();
        let m = i.len();
        if m > 0 {
            self.items.extend_from_slice(i);
            self.input.consume(m);
        }
        if self.input.finished() {
            io.finished = true;
        }
        Ok(())
    }
}

#[test]
fn test_afc_ff_dc_offset_tracking() -> Result<()> {
    let mut fg = Flowgraph::new();

    // Signal: DC offset of 0.75 + AC sine wave
    let mut samples = Vec::new();
    for n in 0..1000 {
        let ac = (2.0 * PI * 100.0 * (n as f32) / 48000.0).sin() * 0.1;
        samples.push(0.75 + ac);
    }

    let src = VectorSource::<f32>::new(samples);
    let afc = AfcFf::new(0.01, 2.0);
    let snk = FloatCollectorSink::new();

    connect!(fg, src > afc > snk;);

    let rt = Runtime::new();
    let fg = rt.run(fg)?;

    let snk_blk = fg.block(&snk)?;
    let out_items = &snk_blk.items;

    // Output should converge to mean 0.0 in the second half of the stream
    let second_half = &out_items[500..];
    let avg: f32 = second_half.iter().sum::<f32>() / (second_half.len() as f32);

    assert!(
        avg.abs() < 0.05,
        "afc_ff failed to track and remove DC offset, avg = {}",
        avg
    );

    Ok(())
}

#[test]
fn test_afc_cc_carrier_tracking() -> Result<()> {
    let mut fg = Flowgraph::new();

    // Signal: Frequency offset of +1000 Hz at 48000 Hz sample rate (freq_rad = 2*pi*1000/48000 = 0.1309 rad/sample)
    let freq_offset_hz = 1000.0f32;
    let samp_rate = 48000.0f32;
    let freq_rad = 2.0 * PI * freq_offset_hz / samp_rate;

    let mut samples = Vec::new();
    for n in 0..2000 {
        let phase = freq_rad * (n as f32);
        samples.push(Complex32::new(phase.cos(), phase.sin()));
    }

    let src = VectorSource::<Complex32>::new(samples);
    let afc = AfcCc::new(0.005, PI / 2.0);
    let snk = ComplexCollectorSink::new();

    connect!(fg, src > afc > snk;);

    let rt = Runtime::new();
    let fg = rt.run(fg)?;

    let snk_blk = fg.block(&snk)?;
    let out_items = &snk_blk.items;

    // Compute residual phase change per sample in the second half
    let second_half = &out_items[1000..];
    let mut max_phase_step = 0.0f32;

    for i in 1..second_half.len() {
        let diff = second_half[i] * Complex32::new(second_half[i - 1].re, -second_half[i - 1].im);
        let phase_step = diff.arg().abs();
        if phase_step > max_phase_step {
            max_phase_step = phase_step;
        }
    }

    assert!(
        max_phase_step < 0.05,
        "afc_cc failed to lock on carrier, max residual phase step = {} rad/sample",
        max_phase_step
    );

    Ok(())
}
