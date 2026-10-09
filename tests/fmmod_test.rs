use anyhow::Result;
use fsdr_cli::blocks::{FmModFc, QuadratureDemodCf};
use futuresdr::blocks::{VectorSink, VectorSource};
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::{Flowgraph, Runtime};

#[test]
fn test_fmmod_pure_tone_and_unit_norm() -> Result<()> {
    let sample_count = 4096;
    let freq_dev = 0.05f32; // constant frequency deviation

    let input_data = vec![freq_dev; sample_count];

    let mut fg = Flowgraph::new();
    let src = fg.add(VectorSource::<f32>::new(input_data))?;
    let fmmod = fg.add(FmModFc::new())?;
    let snk = fg.add(VectorSink::<Complex32>::new(sample_count))?;

    fg.stream_dyn(src.id(), "output", fmmod.id(), "input")?;
    fg.stream_dyn(fmmod.id(), "output", snk.id(), "input")?;

    let term_fg = Runtime::new().run(fg)?;
    let snk_block = term_fg.block(&snk)?;
    let output = snk_block.items();

    assert_eq!(output.len(), sample_count);

    // Verify unit amplitude and exact phase step
    let mut expected_phase = 0.0f32;
    for (i, &sample) in output.iter().enumerate() {
        expected_phase += freq_dev;
        if expected_phase > std::f32::consts::PI {
            expected_phase -= 2.0 * std::f32::consts::PI;
        } else if expected_phase < -std::f32::consts::PI {
            expected_phase += 2.0 * std::f32::consts::PI;
        }
        let expected_cos = expected_phase.cos();
        let expected_sin = expected_phase.sin();

        let norm = (sample.re * sample.re + sample.im * sample.im).sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-4,
            "FmModFc produced non-unitary amplitude at sample {i}: norm = {norm}"
        );

        assert!(
            (sample.re - expected_cos).abs() < 1e-4,
            "FmModFc Re mismatch at sample {i}: expected {expected_cos}, got {}",
            sample.re
        );
        assert!(
            (sample.im - expected_sin).abs() < 1e-4,
            "FmModFc Im mismatch at sample {i}: expected {expected_sin}, got {}",
            sample.im
        );
    }

    Ok(())
}

#[test]
fn test_fmmod_quadrature_demod_roundtrip() -> Result<()> {
    let sample_count = 8192;

    // Modulating audio signal (superposition of two tones)
    let mut modulating_signal = Vec::with_capacity(sample_count);
    for i in 0..sample_count {
        let t = i as f32;
        let tone1 = 0.05 * (t * 0.02).sin();
        let tone2 = 0.02 * (t * 0.07).cos();
        modulating_signal.push(tone1 + tone2);
    }

    // Pipeline: Modulating Audio -> FmModFc -> QuadratureDemodCf -> Demodulated Audio
    let mut fg = Flowgraph::new();
    let src = fg.add(VectorSource::<f32>::new(modulating_signal.clone()))?;
    let fmmod = fg.add(FmModFc::new())?;
    let demod = fg.add(QuadratureDemodCf::new(1.0))?;
    let snk = fg.add(VectorSink::<f32>::new(sample_count))?;

    fg.stream_dyn(src.id(), "output", fmmod.id(), "input")?;
    fg.stream_dyn(fmmod.id(), "output", demod.id(), "input")?;
    fg.stream_dyn(demod.id(), "output", snk.id(), "input")?;

    let term_fg = Runtime::new().run(fg)?;
    let snk_block = term_fg.block(&snk)?;
    let recovered_signal = snk_block.items();

    assert_eq!(recovered_signal.len(), sample_count);

    // Verify perfect roundtrip recovery (ignoring initial startup sample)
    for i in 1..sample_count {
        let original = modulating_signal[i];
        let recovered = recovered_signal[i];
        let diff = (original - recovered).abs();
        assert!(
            diff < 2e-4,
            "FM Mod/Demod roundtrip error at sample {i}: original={original}, recovered={recovered}, diff={diff}"
        );
    }

    Ok(())
}
