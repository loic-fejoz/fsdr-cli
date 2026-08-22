#!/bin/bash
export SAMPLING=1800000
export FREQ=433500000       # 433.500 MHz (Fréquence d'appel simplex FM 70cm)
export CTCSS_TONE=88.5      # Ton CTCSS configuré sur votre émetteur

rtl_sdr -f ${FREQ} -s ${SAMPLING} -g 35 - | \
./target/release/fsdr-cli csdr convert_u8_f ! convert_ff_c \
    ! rational_resampler_cc 2 75 \
    ! power_tagger_cc --samp-rate 48000 --on-threshold -15dB --off-threshold -22dB --off-delay 300ms --off-tag msgend \
    ! fmdemod_quadri_cf \
    ! deemphasis_nfm_ff 48000 \
    ! ctcss_detect_ff --samp-rate 48000 --tone "${CTCSS_TONE}" --threshold 0.000003 --duration 200ms --tag msgstart \
    ! timer_tagger_ff --samp-rate 48000 --duration 30s --start msgstart --end msgend \
    ! cmd_trigger_f --start-tag msgstart --end-tag msgend --cmd './nfm2dtn-script.sh $input_file'
