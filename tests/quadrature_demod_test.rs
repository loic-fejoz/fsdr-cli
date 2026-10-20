use anyhow::Result;
use fsdr_cli::blocks::{QuadratureDemodAlgo, QuadratureDemodCf};
use futuresdr::blocks::{VectorSink, VectorSource};
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::{Flowgraph, Runtime};

#[test]
fn test_quadri_demod_execution_and_precision() -> Result<()> {
    let sample_count = 8192;
    let gain = 1.25;

    // Generate FM test signal: frequency modulated sinusoidal carrier
    let mut input_data = Vec::with_capacity(sample_count);
    let mut phase = 0.0f32;
    for i in 0..sample_count {
        let mod_signal = (i as f32 * 0.005).sin(); // Baseband modulating signal
        let freq_deviation = 0.2 + 0.1 * mod_signal;
        phase += freq_deviation;
        input_data.push(Complex32::new(phase.cos(), phase.sin()));
    }

    // Reference textbook computation
    let mut expected_output = Vec::with_capacity(sample_count);
    let mut last_ref = Complex32::new(0.0, 0.0);
    for &s in &input_data {
        let arg = (s * last_ref.conj()).arg();
        last_ref = s;
        expected_output.push(arg * gain);
    }

    // Run FutureSDR Flowgraph with QuadratureDemodCf (Quadri)
    let mut fg = Flowgraph::new();
    let src = fg.add(VectorSource::<Complex32>::new(input_data))?;
    let demod = fg.add(QuadratureDemodCf::new(gain))?;
    let snk = fg.add(VectorSink::<f32>::new(sample_count))?;

    fg.stream_dyn(src.id(), "output", demod.id(), "input")?;
    fg.stream_dyn(demod.id(), "output", snk.id(), "input")?;

    let term_fg = Runtime::new().run(fg)?;
    let snk_block = term_fg.block(&snk)?;
    let actual_output = snk_block.items();

    assert_eq!(actual_output.len(), sample_count);

    // Verify precision: max absolute error < 1.5e-4
    for (i, (&exp, &act)) in expected_output.iter().zip(actual_output.iter()).enumerate() {
        let diff = (exp - act).abs();
        assert!(
            diff < 1.5e-4,
            "Quadri demod error at sample {i}: expected {exp}, got {act}, diff {diff}"
        );
    }

    Ok(())
}

#[test]
fn test_atan_demod_execution_and_precision() -> Result<()> {
    let sample_count = 8192;
    let gain = 0.8;

    let mut input_data = Vec::with_capacity(sample_count);
    let mut phase = 0.0f32;
    for i in 0..sample_count {
        let mod_signal = (i as f32 * 0.003).sin();
        let freq_deviation = 0.15 + 0.08 * mod_signal;
        phase += freq_deviation;
        input_data.push(Complex32::new(phase.cos(), phase.sin()));
    }

    // Reference textbook computation
    let mut expected_output = Vec::with_capacity(sample_count);
    let mut last_phase = 0.0f32;
    for &s in &input_data {
        let p = s.arg();
        let mut diff = p - last_phase;
        while diff > std::f32::consts::PI {
            diff -= 2.0 * std::f32::consts::PI;
        }
        while diff < -std::f32::consts::PI {
            diff += 2.0 * std::f32::consts::PI;
        }
        last_phase = p;
        expected_output.push(diff * gain);
    }

    // Run FutureSDR Flowgraph with QuadratureDemodCf (Atan)
    let mut fg = Flowgraph::new();
    let src = fg.add(VectorSource::<Complex32>::new(input_data))?;
    let demod = fg.add(QuadratureDemodCf::with_algo(
        gain,
        QuadratureDemodAlgo::Atan,
    ))?;
    let snk = fg.add(VectorSink::<f32>::new(sample_count))?;

    fg.stream_dyn(src.id(), "output", demod.id(), "input")?;
    fg.stream_dyn(demod.id(), "output", snk.id(), "input")?;

    let term_fg = Runtime::new().run(fg)?;
    let snk_block = term_fg.block(&snk)?;
    let actual_output = snk_block.items();

    assert_eq!(actual_output.len(), sample_count);

    for (i, (&exp, &act)) in expected_output.iter().zip(actual_output.iter()).enumerate() {
        let diff = (exp - act).abs();
        assert!(
            diff < 1.5e-4,
            "Atan demod error at sample {i}: expected {exp}, got {act}, diff {diff}"
        );
    }

    Ok(())
}
