use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use fsdr_blocks::math::FrequencyShifter as FsdrBlocksFreqShift;
use fsdr_cli::blocks::synchronizers::AfcFf;
use fsdr_cli::blocks::{
    AddConstCc, AddDcOffsetCc, CostasLoopCc, CtcssGenerator, DCBlocker, DcBlockFf,
    FixedAmplitudeCc, FmModFc, FrequencyShifter, LogPowerCf,
};
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
            let src = fg
                .add(futuresdr::blocks::VectorSource::<Complex32>::new(
                    input_data.clone(),
                ))
                .unwrap();
            let shifter = fg
                .add(FrequencyShifter::<Complex32>::new(1000.0, 48000.0))
                .unwrap();
            let snk = fg
                .add(futuresdr::blocks::VectorSink::<Complex32>::new(
                    sample_count,
                ))
                .unwrap();

            fg.stream_dyn(src.id(), "output", shifter.id(), "input")
                .unwrap();
            fg.stream_dyn(shifter.id(), "output", snk.id(), "input")
                .unwrap();

            let term_fg = Runtime::new().run(fg).unwrap();
            let res = term_fg.block(&snk).unwrap();
            black_box(res.items().len());
        });
    });

    group.bench_function("fsdr_blocks_fast_math_freq_shift", |b| {
        b.iter(|| {
            let mut fg = Flowgraph::new();
            let src = fg
                .add(futuresdr::blocks::VectorSource::<Complex32>::new(
                    input_data.clone(),
                ))
                .unwrap();
            let shifter = fg
                .add(FsdrBlocksFreqShift::<Complex32>::new(1000.0, 48000.0))
                .unwrap();
            let snk = fg
                .add(futuresdr::blocks::VectorSink::<Complex32>::new(
                    sample_count,
                ))
                .unwrap();

            fg.stream_dyn(src.id(), "output", shifter.id(), "input")
                .unwrap();
            fg.stream_dyn(shifter.id(), "output", snk.id(), "input")
                .unwrap();

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
            let src = fg
                .add(futuresdr::blocks::VectorSource::<f32>::new(
                    input_data.clone(),
                ))
                .unwrap();
            let dcblock = fg.add(DCBlocker::<f32>::new(16)).unwrap();
            let snk = fg
                .add(futuresdr::blocks::VectorSink::<f32>::new(sample_count))
                .unwrap();

            fg.stream_dyn(src.id(), "output", dcblock.id(), "input")
                .unwrap();
            fg.stream_dyn(dcblock.id(), "output", snk.id(), "input")
                .unwrap();

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

    let input_data: Vec<f32> = (0..sample_count).map(|i| (i as f32 * 0.05).sin()).collect();

    group.bench_function("fsdr_cli_fast_math_ctcss_gen", |b| {
        b.iter(|| {
            let mut fg = Flowgraph::new();
            let src = fg
                .add(futuresdr::blocks::VectorSource::<f32>::new(
                    input_data.clone(),
                ))
                .unwrap();
            let ctcss = fg.add(CtcssGenerator::new(48000.0)).unwrap();
            let snk = fg
                .add(futuresdr::blocks::VectorSink::<f32>::new(sample_count))
                .unwrap();

            fg.stream_dyn(src.id(), "output", ctcss.id(), "input")
                .unwrap();
            fg.stream_dyn(ctcss.id(), "output", snk.id(), "input")
                .unwrap();

            let term_fg = Runtime::new().run(fg).unwrap();
            let res = term_fg.block(&snk).unwrap();
            black_box(res.items().len());
        });
    });

    group.finish();
}

fn bench_add_dcoffset(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_dcoffset");
    let sample_count = 131072;
    group.throughput(Throughput::Elements(sample_count as u64));

    let input_data: Vec<Complex32> = (0..sample_count)
        .map(|i| Complex32::new((i as f32 * 0.01).sin(), (i as f32 * 0.01).cos()))
        .collect();

    group.bench_function("fsdr_cli_add_dcoffset", |b| {
        b.iter(|| {
            let mut fg = Flowgraph::new();
            let src = fg
                .add(futuresdr::blocks::VectorSource::<Complex32>::new(
                    input_data.clone(),
                ))
                .unwrap();
            let block = fg
                .add(AddDcOffsetCc::new(Complex32::new(0.5, 0.5)))
                .unwrap();
            let snk = fg
                .add(futuresdr::blocks::VectorSink::<Complex32>::new(
                    sample_count,
                ))
                .unwrap();

            fg.stream_dyn(src.id(), "output", block.id(), "input")
                .unwrap();
            fg.stream_dyn(block.id(), "output", snk.id(), "input")
                .unwrap();

            let term_fg = Runtime::new().run(fg).unwrap();
            let res = term_fg.block(&snk).unwrap();
            black_box(res.items().len());
        });
    });

    group.finish();
}

fn bench_dcblock_iir(c: &mut Criterion) {
    let mut group = c.benchmark_group("dcblock_iir");
    let sample_count = 131072;
    group.throughput(Throughput::Elements(sample_count as u64));

    let input_data: Vec<f32> = (0..sample_count)
        .map(|i| (i as f32 * 0.01).sin() + 1.5)
        .collect();

    group.bench_function("fsdr_cli_dcblock_iir", |b| {
        b.iter(|| {
            let mut fg = Flowgraph::new();
            let src = fg
                .add(futuresdr::blocks::VectorSource::<f32>::new(
                    input_data.clone(),
                ))
                .unwrap();
            let block = fg.add(DcBlockFf::new(0.999)).unwrap();
            let snk = fg
                .add(futuresdr::blocks::VectorSink::<f32>::new(sample_count))
                .unwrap();

            fg.stream_dyn(src.id(), "output", block.id(), "input")
                .unwrap();
            fg.stream_dyn(block.id(), "output", snk.id(), "input")
                .unwrap();

            let term_fg = Runtime::new().run(fg).unwrap();
            let res = term_fg.block(&snk).unwrap();
            black_box(res.items().len());
        });
    });

    group.finish();
}

fn bench_logpower(c: &mut Criterion) {
    let mut group = c.benchmark_group("logpower");
    let sample_count = 131072;
    group.throughput(Throughput::Elements(sample_count as u64));

    let input_data: Vec<Complex32> = (0..sample_count)
        .map(|i| Complex32::new((i as f32 * 0.01).sin(), (i as f32 * 0.01).cos()))
        .collect();

    group.bench_function("fsdr_cli_logpower", |b| {
        b.iter(|| {
            let mut fg = Flowgraph::new();
            let src = fg
                .add(futuresdr::blocks::VectorSource::<Complex32>::new(
                    input_data.clone(),
                ))
                .unwrap();
            let block = fg.add(LogPowerCf::new(0.0)).unwrap();
            let snk = fg
                .add(futuresdr::blocks::VectorSink::<f32>::new(sample_count))
                .unwrap();

            fg.stream_dyn(src.id(), "output", block.id(), "input")
                .unwrap();
            fg.stream_dyn(block.id(), "output", snk.id(), "input")
                .unwrap();

            let term_fg = Runtime::new().run(fg).unwrap();
            let res = term_fg.block(&snk).unwrap();
            black_box(res.items().len());
        });
    });

    group.finish();
}

fn bench_costas_loop(c: &mut Criterion) {
    let mut group = c.benchmark_group("costas_loop");
    let sample_count = 131072;
    group.throughput(Throughput::Elements(sample_count as u64));

    let input_data: Vec<Complex32> = (0..sample_count)
        .map(|i| Complex32::new((i as f32 * 0.01).sin(), (i as f32 * 0.01).cos()))
        .collect();

    group.bench_function("fsdr_cli_costas_loop", |b| {
        b.iter(|| {
            let mut fg = Flowgraph::new();
            let src = fg
                .add(futuresdr::blocks::VectorSource::<Complex32>::new(
                    input_data.clone(),
                ))
                .unwrap();
            let block = fg.add(CostasLoopCc::new(0.05, 0.707)).unwrap();
            let snk = fg
                .add(futuresdr::blocks::VectorSink::<Complex32>::new(
                    sample_count,
                ))
                .unwrap();

            fg.stream_dyn(src.id(), "output", block.id(), "input")
                .unwrap();
            fg.stream_dyn(block.id(), "output", snk.id(), "input")
                .unwrap();

            let term_fg = Runtime::new().run(fg).unwrap();
            let res = term_fg.block(&snk).unwrap();
            black_box(res.items().len());
        });
    });

    group.finish();
}

fn bench_fixed_amplitude(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixed_amplitude");
    let sample_count = 131072;
    group.throughput(Throughput::Elements(sample_count as u64));

    let input_data: Vec<Complex32> = (0..sample_count)
        .map(|i| Complex32::new((i as f32 * 0.01).sin() + 0.1, (i as f32 * 0.01).cos() + 0.1))
        .collect();

    group.bench_function("fsdr_cli_fixed_amplitude", |b| {
        b.iter(|| {
            let mut fg = Flowgraph::new();
            let src = fg
                .add(futuresdr::blocks::VectorSource::<Complex32>::new(
                    input_data.clone(),
                ))
                .unwrap();
            let block = fg.add(FixedAmplitudeCc::new(1.0)).unwrap();
            let snk = fg
                .add(futuresdr::blocks::VectorSink::<Complex32>::new(
                    sample_count,
                ))
                .unwrap();

            fg.stream_dyn(src.id(), "output", block.id(), "input")
                .unwrap();
            fg.stream_dyn(block.id(), "output", snk.id(), "input")
                .unwrap();

            let term_fg = Runtime::new().run(fg).unwrap();
            let res = term_fg.block(&snk).unwrap();
            black_box(res.items().len());
        });
    });

    group.finish();
}

fn bench_afc_ff(c: &mut Criterion) {
    let mut group = c.benchmark_group("afc_ff");
    let sample_count = 131072;
    group.throughput(Throughput::Elements(sample_count as u64));

    let input_data: Vec<f32> = (0..sample_count)
        .map(|i| (i as f32 * 0.01).sin() + 0.5)
        .collect();

    group.bench_function("fsdr_cli_afc_ff", |b| {
        b.iter(|| {
            let mut fg = Flowgraph::new();
            let src = fg
                .add(futuresdr::blocks::VectorSource::<f32>::new(
                    input_data.clone(),
                ))
                .unwrap();
            let block = fg.add(AfcFf::new(0.01, 1.0)).unwrap();
            let snk = fg
                .add(futuresdr::blocks::VectorSink::<f32>::new(sample_count))
                .unwrap();

            fg.stream_dyn(src.id(), "output", block.id(), "input")
                .unwrap();
            fg.stream_dyn(block.id(), "output", snk.id(), "input")
                .unwrap();

            let term_fg = Runtime::new().run(fg).unwrap();
            let res = term_fg.block(&snk).unwrap();
            black_box(res.items().len());
        });
    });

    group.finish();
}

fn bench_add_const(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_const");
    let sample_count = 131072;
    group.throughput(Throughput::Elements(sample_count as u64));

    let input_data: Vec<Complex32> = (0..sample_count)
        .map(|i| Complex32::new((i as f32 * 0.01).sin(), (i as f32 * 0.01).cos()))
        .collect();

    group.bench_function("fsdr_cli_add_const", |b| {
        b.iter(|| {
            let mut fg = Flowgraph::new();
            let src = fg
                .add(futuresdr::blocks::VectorSource::<Complex32>::new(
                    input_data.clone(),
                ))
                .unwrap();
            let block = fg.add(AddConstCc::new(Complex32::new(1.0, -1.0))).unwrap();
            let snk = fg
                .add(futuresdr::blocks::VectorSink::<Complex32>::new(
                    sample_count,
                ))
                .unwrap();

            fg.stream_dyn(src.id(), "output", block.id(), "input")
                .unwrap();
            fg.stream_dyn(block.id(), "output", snk.id(), "input")
                .unwrap();

            let term_fg = Runtime::new().run(fg).unwrap();
            let res = term_fg.block(&snk).unwrap();
            black_box(res.items().len());
        });
    });

    group.finish();
}

fn bench_fmmod(c: &mut Criterion) {
    let mut group = c.benchmark_group("fmmod");
    let sample_count = 131072;
    group.throughput(Throughput::Elements(sample_count as u64));

    let input_data: Vec<f32> = (0..sample_count)
        .map(|i| (i as f32 * 0.001).sin() * 0.1)
        .collect();

    group.bench_function("fsdr_cli_fmmod", |b| {
        b.iter(|| {
            let mut fg = Flowgraph::new();
            let src = fg
                .add(futuresdr::blocks::VectorSource::<f32>::new(
                    input_data.clone(),
                ))
                .unwrap();
            let block = fg.add(FmModFc::new()).unwrap();
            let snk = fg
                .add(futuresdr::blocks::VectorSink::<Complex32>::new(
                    sample_count,
                ))
                .unwrap();

            fg.stream_dyn(src.id(), "output", block.id(), "input")
                .unwrap();
            fg.stream_dyn(block.id(), "output", snk.id(), "input")
                .unwrap();

            let term_fg = Runtime::new().run(fg).unwrap();
            let res = term_fg.block(&snk).unwrap();
            black_box(res.items().len());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_frequency_shifter,
    bench_dcblocker,
    bench_ctcss_generator,
    bench_add_dcoffset,
    bench_dcblock_iir,
    bench_logpower,
    bench_costas_loop,
    bench_fixed_amplitude,
    bench_afc_ff,
    bench_add_const,
    bench_fmmod
);
criterion_main!(benches);
