use anyhow::Result;
use fsdr_cli::csdr_cmd::CsdrParser;
use fsdr_cli::grc::converter::Grc2FutureSdr;
use futuresdr::runtime::Runtime;
use std::f32::consts::PI;
use std::fs::File;
use std::io::Write;

#[test]
fn test_end_to_end_nfm_ctcss_pipeline() -> Result<()> {
    let output_flag_file = tempfile::NamedTempFile::new()?.into_temp_path();
    let flag_file_str = output_flag_file.to_str().unwrap().to_string();

    let samp_rate = 2_400_000.0f32;
    let tone_ctcss = 88.5f32;
    let audio_tone = 1000.0f32;

    // Synthesize IQ samples:
    // 0.2s no carrier, then 0.4s FM carrier with 88.5Hz CTCSS + 1kHz tone, then 0.2s no carrier
    let total_seconds = 0.8f32;
    let total_samples = (total_seconds * samp_rate) as usize;

    let mut u8_bytes = Vec::with_capacity(total_samples * 2);
    let mut phase = 0.0f32;
    let f_deviation = 5000.0f32;

    for i in 0..total_samples {
        let t = i as f32 / samp_rate;
        let is_carrier_on = t >= 0.2 && t <= 0.6;

        if is_carrier_on {
            // Audio signal: CTCSS (0.25 amp) + 1kHz tone (0.75 amp)
            let audio =
                0.25 * (2.0 * PI * tone_ctcss * t).sin() + 0.75 * (2.0 * PI * audio_tone * t).sin();
            let freq_inst = f_deviation * audio;
            phase += 2.0 * PI * freq_inst / samp_rate;
            if phase > 2.0 * PI {
                phase -= 2.0 * PI;
            }

            let i_val = phase.cos();
            let q_val = phase.sin();

            // Convert to u8 (127.5 offset)
            let i_u8 = ((i_val * 120.0) + 127.5).clamp(0.0, 255.0) as u8;
            let q_u8 = ((q_val * 120.0) + 127.5).clamp(0.0, 255.0) as u8;
            u8_bytes.push(i_u8);
            u8_bytes.push(q_u8);
        } else {
            // No carrier: low noise around center
            u8_bytes.push(128);
            u8_bytes.push(128);
        }
    }

    let input_iq_file = tempfile::NamedTempFile::new()?.into_temp_path();
    {
        let mut f = File::create(&input_iq_file)?;
        f.write_all(&u8_bytes)?;
        f.flush()?;
    }

    // Build the GRC flowgraph using fsdr-cli pipeline parser
    let cmd = format!(
        "csdr load_u8 {} ! convert_u8_f ! convert_ff_c \
        ! power_tagger_cc --samp-rate 2400000 --off-threshold -20dB --off-delay 20ms --off-tag msgend \
        ! rational_resampler_cc 1 50 \
        ! fmdemod_quadri_cf \
        ! deemphasis_nfm_ff 48000 \
        ! ctcss_detect_ff --samp-rate 48000 --tone 88.5 --threshold 0.0001 --duration 20ms --tag msgstart \
        ! timer_tagger_ff --samp-rate 48000 --duration 10s --start msgstart --end msgend \
        ! cmd_trigger_f --start-tag msgstart --end-tag msgend --cmd 'echo $input_file >> {}'",
        input_iq_file.to_str().unwrap(),
        flag_file_str
    );

    let grc = CsdrParser::parse_multiple_commands(&cmd)?.unwrap();
    let mut cvt = Grc2FutureSdr::new();
    let fg = cvt.convert_grc(grc)?;

    let rt = Runtime::new();
    let _ = rt.run(fg)?;

    // Allow background thread execution to complete
    std::thread::sleep(std::time::Duration::from_millis(800));

    let content = std::fs::read_to_string(&output_flag_file)?;
    let lines: Vec<&str> = content.lines().collect();

    assert!(
        !lines.is_empty(),
        "Expected at least 1 message triggered by the pipeline, found: {:?}",
        lines
    );
    assert!(lines[0].ends_with("message.sigmf-meta"));

    Ok(())
}
