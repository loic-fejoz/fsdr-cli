use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use fsdr_blocks::math::FrequencyShifter as FsdrBlocksFreqShift;
use fsdr_cli::blocks::synchronizers::AfcFf;
use fsdr_cli::blocks::{
    AddConstCc, AddDcOffsetCc, CostasLoopCc, CtcssGenerator, DCBlocker, DcBlockFf,
    FixedAmplitudeCc, FmModFc, FrequencyShifter, LogPowerCf, QuadratureDemodAlgo,
    QuadratureDemodCf,
};
use fsdr_cli::math::{fast_atan2, fast_sincos};
use futuresdr::blocks::Apply;
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

fn bench_quadrature_demod(c: &mut Criterion) {
    let mut group = c.benchmark_group("quadrature_demod");
    let sample_count = 131072;
    group.throughput(Throughput::Elements(sample_count as u64));

    let mut input_data = Vec::with_capacity(sample_count);
    let mut phase = 0.0f32;
    for i in 0..sample_count {
        phase += 0.2 + 0.1 * (i as f32 * 0.005).sin();
        input_data.push(Complex32::new(phase.cos(), phase.sin()));
    }

    // Benchmark 1: Legacy scalar Apply with atan2f@GLIBC
    group.bench_function("legacy_scalar_apply_quadri_demod", |b| {
        b.iter(|| {
            let mut fg = Flowgraph::new();
            let src = fg
                .add(futuresdr::blocks::VectorSource::<Complex32>::new(
                    input_data.clone(),
                ))
                .unwrap();
            let mut last = Complex32::new(0.0, 0.0);
            let demod: Apply<_, Complex32, f32> = Apply::new(move |v: &Complex32| -> f32 {
                let arg = (v * last.conj()).arg();
                last = *v;
                arg * 1.0
            });
            let demod_id = fg.add(demod).unwrap();
            let snk = fg
                .add(futuresdr::blocks::VectorSink::<f32>::new(sample_count))
                .unwrap();

            fg.stream_dyn(src.id(), "output", demod_id.id(), "input")
                .unwrap();
            fg.stream_dyn(demod_id.id(), "output", snk.id(), "input")
                .unwrap();

            let term_fg = Runtime::new().run(fg).unwrap();
            let res = term_fg.block(&snk).unwrap();
            black_box(res.items().len());
        });
    });

    // Benchmark 2: Optimized Dedicated QuadratureDemodCf (Quadri) with chunked FMA fast_atan2
    group.bench_function("optimized_quadri_demod_cf", |b| {
        b.iter(|| {
            let mut fg = Flowgraph::new();
            let src = fg
                .add(futuresdr::blocks::VectorSource::<Complex32>::new(
                    input_data.clone(),
                ))
                .unwrap();
            let demod = fg.add(QuadratureDemodCf::new(1.0)).unwrap();
            let snk = fg
                .add(futuresdr::blocks::VectorSink::<f32>::new(sample_count))
                .unwrap();

            fg.stream_dyn(src.id(), "output", demod.id(), "input")
                .unwrap();
            fg.stream_dyn(demod.id(), "output", snk.id(), "input")
                .unwrap();

            let term_fg = Runtime::new().run(fg).unwrap();
            let res = term_fg.block(&snk).unwrap();
            black_box(res.items().len());
        });
    });

    // Benchmark 3: Optimized Dedicated QuadratureDemodCf (Atan) with chunked FMA fast_atan2
    group.bench_function("optimized_atan_demod_cf", |b| {
        b.iter(|| {
            let mut fg = Flowgraph::new();
            let src = fg
                .add(futuresdr::blocks::VectorSource::<Complex32>::new(
                    input_data.clone(),
                ))
                .unwrap();
            let demod = fg
                .add(QuadratureDemodCf::with_algo(1.0, QuadratureDemodAlgo::Atan))
                .unwrap();
            let snk = fg
                .add(futuresdr::blocks::VectorSink::<f32>::new(sample_count))
                .unwrap();

            fg.stream_dyn(src.id(), "output", demod.id(), "input")
                .unwrap();
            fg.stream_dyn(demod.id(), "output", snk.id(), "input")
                .unwrap();

            let term_fg = Runtime::new().run(fg).unwrap();
            let res = term_fg.block(&snk).unwrap();
            black_box(res.items().len());
        });
    });

    group.finish();
}

fn bench_transcendental_micro(c: &mut Criterion) {
    let mut group = c.benchmark_group("transcendental_math_micro");
    let sample_count = 131072;
    group.throughput(Throughput::Elements(sample_count as u64));

    let y_vals: Vec<f32> = (0..sample_count).map(|i| (i as f32 * 0.01).sin()).collect();
    let x_vals: Vec<f32> = (0..sample_count).map(|i| (i as f32 * 0.01).cos()).collect();
    let mut out = vec![0.0f32; sample_count];

    group.bench_function("scalar_libm_atan2", |b| {
        b.iter(|| {
            for i in 0..sample_count {
                out[i] = y_vals[i].atan2(x_vals[i]);
            }
            black_box(&out[..]);
        });
    });

    group.bench_function("fast_fma_atan2", |b| {
        b.iter(|| {
            for i in 0..sample_count {
                out[i] = fast_atan2(y_vals[i], x_vals[i]);
            }
            black_box(&out[..]);
        });
    });

    group.bench_function("scalar_libm_sincos", |b| {
        b.iter(|| {
            for i in 0..sample_count {
                let (s, c) = y_vals[i].sin_cos();
                out[i] = s + c;
            }
            black_box(&out[..]);
        });
    });

    group.bench_function("fast_fma_sincos", |b| {
        b.iter(|| {
            for i in 0..sample_count {
                let (s, c) = fast_sincos(y_vals[i]);
                out[i] = s + c;
            }
            black_box(&out[..]);
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
    bench_fmmod,
    bench_quadrature_demod,
    bench_transcendental_micro
);
criterion_main!(benches);
