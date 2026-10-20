use anyhow::Result;
use fsdr_cli::blocks::CostasLoopCc;
use futuresdr::blocks::{VectorSink, VectorSource};
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::{Flowgraph, Runtime};
use std::f32::consts::PI;

#[test]
fn test_costas_loop_lock_and_convergence() -> Result<()> {
    let sample_count = 4096;
    let freq_offset = 0.02f32; // rad/sample carrier frequency offset
    let initial_phase = 0.45f32; // rad initial phase offset

    // Generate BPSK sequence with carrier offset and phase offset:
    // s[n] = b[n] * exp(j * (freq_offset * n + initial_phase))
    let mut input_data = Vec::with_capacity(sample_count);
    let mut bits = Vec::with_capacity(sample_count);
    let mut phase = initial_phase;

    for i in 0..sample_count {
        // Pseudo-random BPSK symbols (+1.0 or -1.0)
        let bit = if ((i * 1103515245 + 12345) / 65536) % 2 == 0 {
            1.0f32
        } else {
            -1.0f32
        };
        bits.push(bit);

        phase += freq_offset;
        let carrier = Complex32::new(phase.cos(), phase.sin());
        input_data.push(Complex32::new(bit, 0.0) * carrier);
    }

    // Run Costas Loop with loop_bw = 0.04, damping = 0.707
    let mut fg = Flowgraph::new();
    let src = fg.add(VectorSource::<Complex32>::new(input_data))?;
    let costas = fg.add(CostasLoopCc::new(0.04, 0.707))?;
    let snk = fg.add(VectorSink::<Complex32>::new(sample_count))?;

    fg.stream_dyn(src.id(), "output", costas.id(), "input")?;
    fg.stream_dyn(costas.id(), "output", snk.id(), "input")?;

    let term_fg = Runtime::new().run(fg)?;
    let snk_block = term_fg.block(&snk)?;
    let output = snk_block.items();

    assert_eq!(output.len(), sample_count);

    // Verify convergence on second half of samples (after loop locks)
    let check_start = sample_count / 2;
    for (idx, &sample) in output[check_start..].iter().enumerate() {
        let i = check_start + idx;
        // In BPSK after lock: Im should be close to 0, |Re| should be close to 1.0
        assert!(
            sample.im.abs() < 0.15,
            "Costas loop failed to lock carrier phase: Im(out[{i}]) = {}",
            sample.im
        );
        assert!(
            (sample.re.abs() - 1.0).abs() < 0.15,
            "Costas loop amplitude distortion: |Re(out[{i}])| = {}",
            sample.re.abs()
        );
    }

    Ok(())
}

#[test]
fn test_costas_loop_against_textbook_simulation() -> Result<()> {
    let sample_count = 1024;
    let loop_bw = 0.05f32;
    let damping = 0.707f32;

    let denom = 1.0 + 2.0 * damping * loop_bw + loop_bw * loop_bw;
    let alpha = (4.0 * damping * loop_bw) / denom;
    let beta = (4.0 * loop_bw * loop_bw) / denom;

    let input_data: Vec<Complex32> = (0..sample_count)
        .map(|i| {
            let p = 0.03 * (i as f32);
            Complex32::new(p.cos(), p.sin())
        })
        .collect();

    // Textbook step-by-step simulation
    let mut expected_output = Vec::with_capacity(sample_count);
    let mut sim_phase = 0.0f32;
    let mut sim_freq = 0.0f32;

    for &s in &input_data {
        let (sin_val, cos_val) = sim_phase.sin_cos();
        let rot_re = s.re * cos_val + s.im * sin_val;
        let rot_im = -s.re * sin_val + s.im * cos_val;
        expected_output.push(Complex32::new(rot_re, rot_im));

        let sgn = if rot_re >= 0.0 { 1.0 } else { -1.0 };
        let err = sgn * rot_im;
        sim_freq = (sim_freq + beta * err).clamp(-1.0, 1.0);
        sim_phase += sim_freq + alpha * err;
        while sim_phase > PI {
            sim_phase -= 2.0 * PI;
        }
        while sim_phase < -PI {
            sim_phase += 2.0 * PI;
        }
    }

    // Run CostasLoopCc block
    let mut fg = Flowgraph::new();
    let src = fg.add(VectorSource::<Complex32>::new(input_data))?;
    let costas = fg.add(CostasLoopCc::new(loop_bw, damping))?;
    let snk = fg.add(VectorSink::<Complex32>::new(sample_count))?;

    fg.stream_dyn(src.id(), "output", costas.id(), "input")?;
    fg.stream_dyn(costas.id(), "output", snk.id(), "input")?;

    let term_fg = Runtime::new().run(fg)?;
    let snk_block = term_fg.block(&snk)?;
    let actual_output = snk_block.items();

    assert_eq!(actual_output.len(), sample_count);

    // Verify precision agreement with textbook simulation
    for (i, (&exp, &act)) in expected_output.iter().zip(actual_output.iter()).enumerate() {
        let diff_re = (exp.re - act.re).abs();
        let diff_im = (exp.im - act.im).abs();
        assert!(
            diff_re < 5e-4 && diff_im < 5e-4,
            "Costas simulation mismatch at {i}: expected {exp}, got {act}"
        );
    }

    Ok(())
}
