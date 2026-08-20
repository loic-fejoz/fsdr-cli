#![feature(core_intrinsics)]

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use fsdr_cli::blocks::{CtcssGenerator, DCBlocker, FrequencyShifter};
use fsdr_blocks::math::FrequencyShifter as FsdrBlocksFreqShift;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;

fn bench_frequency_shifter(c: &mut Criterion) {
    let mut group = c.benchmark_group("frequency_shifter");
    let sample_count = 131072;
    group.throughput(Throughput::Elements(sample_count as u64));

    let input_data: Vec<Complex32> = (0..sample_count)
        .map(|i| Complex32::new((i as f32).sin(), (i as f32).cos()))
        .collect();

    group.bench_function("fsdr_cli_fast_math_freq_shift", |b| {
        b.iter(|| {
            let mut fg = Flowgraph::new();
            let src = fg.add(futuresdr::blocks::VectorSource::<Complex32>::new(input_data.clone())).unwrap();
            let shifter = fg.add(FrequencyShifter::new(1000.0, 48000.0)).unwrap();
            let snk = fg.add(futuresdr::blocks::VectorSink::<Complex32>::new(sample_count)).unwrap();

            fg.stream_dyn(src.id(), "output", shifter.id(), "input").unwrap();
            fg.stream_dyn(shifter.id(), "output", snk.id(), "input").unwrap();

            let term_fg = Runtime::new().run(fg).unwrap();
            let res = term_fg.block(&snk).unwrap();
            black_box(res.items().len());
        });
    });

    group.bench_function("fsdr_blocks_fast_math_freq_shift", |b| {
        b.iter(|| {
            let mut fg = Flowgraph::new();
            let src = fg.add(futuresdr::blocks::VectorSource::<Complex32>::new(input_data.clone())).unwrap();
            let shifter = fg.add(FsdrBlocksFreqShift::<Complex32>::new(1000.0, 48000.0)).unwrap();
            let snk = fg.add(futuresdr::blocks::VectorSink::<Complex32>::new(sample_count)).unwrap();

            fg.stream_dyn(src.id(), "output", shifter.id(), "input").unwrap();
            fg.stream_dyn(shifter.id(), "output", snk.id(), "input").unwrap();

            let term_fg = Runtime::new().run(fg).unwrap();
            let res = term_fg.block(&snk).unwrap();
            black_box(res.items().len());
        });
    });

    group.finish();
}

fn bench_dcblocker(c: &mut Criterion) {
    let mut group = c.benchmark_group("dcblocker");
    let sample_count = 131072;
    group.throughput(Throughput::Elements(sample_count as u64));

    let input_data: Vec<f32> = (0..sample_count)
        .map(|i| (i as f32 * 0.01).sin() + 2.5)
        .collect();

    group.bench_function("fsdr_cli_single_pass_fast_math_dcblocker", |b| {
        b.iter(|| {
            let mut fg = Flowgraph::new();
            let src = fg.add(futuresdr::blocks::VectorSource::<f32>::new(input_data.clone())).unwrap();
            let dcblock = fg.add(DCBlocker::<f32>::new(16)).unwrap();
            let snk = fg.add(futuresdr::blocks::VectorSink::<f32>::new(sample_count)).unwrap();

            fg.stream_dyn(src.id(), "output", dcblock.id(), "input").unwrap();
            fg.stream_dyn(dcblock.id(), "output", snk.id(), "input").unwrap();

            let term_fg = Runtime::new().run(fg).unwrap();
            let res = term_fg.block(&snk).unwrap();
            black_box(res.items().len());
        });
    });

    group.finish();
}

fn bench_ctcss_generator(c: &mut Criterion) {
    let mut group = c.benchmark_group("ctcss_generator");
    let sample_count = 131072;
    group.throughput(Throughput::Elements(sample_count as u64));

    let input_data: Vec<f32> = (0..sample_count)
        .map(|i| (i as f32 * 0.05).sin())
        .collect();

    group.bench_function("fsdr_cli_fast_math_ctcss_gen", |b| {
        b.iter(|| {
            let mut fg = Flowgraph::new();
            let src = fg.add(futuresdr::blocks::VectorSource::<f32>::new(input_data.clone())).unwrap();
            let ctcss = fg.add(CtcssGenerator::new(48000.0)).unwrap();
            let snk = fg.add(futuresdr::blocks::VectorSink::<f32>::new(sample_count)).unwrap();

            fg.stream_dyn(src.id(), "output", ctcss.id(), "input").unwrap();
            fg.stream_dyn(ctcss.id(), "output", snk.id(), "input").unwrap();

            let term_fg = Runtime::new().run(fg).unwrap();
            let res = term_fg.block(&snk).unwrap();
            black_box(res.items().len());
        });
    });

    group.finish();
}

criterion_group!(benches, bench_frequency_shifter, bench_dcblocker, bench_ctcss_generator);
criterion_main!(benches);
