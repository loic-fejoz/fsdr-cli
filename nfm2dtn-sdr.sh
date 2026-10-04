#!/bin/bash
export SAMPLING=1800000
export FREQ=433500000       # 433.500 MHz (Fréquence d'appel simplex FM 70cm)
export TUNE_FREQ=433750000  # Offset +250 kHz pour éviter le pic DC matériel
export CTCSS_TONE=88.5      # Ton CTCSS configuré sur votre émetteur

rtl_sdr -f ${TUNE_FREQ} -s ${SAMPLING} -g 35 - | \
./target/release/fsdr-cli csdr convert_u8_f ! convert_ff_c \
    ! shift_addition_cc "((433500000-433750000)/1800000)" \
    ! rational_resampler_cc 2 75 \
    ! agc_ff --reference 0.8 --rate 0.0001 \
    ! power_tagger_cc --samp-rate 48000 --snr-on 10dB --snr-off 4dB --off-delay 300ms --off-tag msgend \
    ! fmdemod_quadri_cf \
    ! afc_ff --alpha 0.01 \
    ! dcblock_ff \
    ! deemphasis_nfm_ff 48000 \
    ! ctcss_detect_ff --samp-rate 48000 --tone "${CTCSS_TONE}" --threshold 0.000006 --duration 220ms --tag msgstart \
    ! timer_tagger_ff --samp-rate 48000 --duration 30s --start msgstart --end msgend \
    ! cmd_trigger_f --start-tag msgstart --end-tag msgend --cmd './nfm2dtn-script.sh $input_file'
