FSDR_CLI:=target/release/fsdr-cli

target/debug/fsdr-cli: src/*.rs src/grc/*.rs
	cargo build

target/release/fsdr-cli: src/*.rs src/grc/*.rs
	cargo build --release

test: cargo-test csdr-compare

cargo-test:
	cargo test

usage: $(FSDR_CLI)
	$(FSDR_CLI)

csdr-compare: csdr-compare-realpart-c-f csdr-compare-dump-u8 

define csdr_compare_cmd

	export CSDR_FIXED_BUFSIZE=$(3) && \
	head -c $(2) ./tests/france-culture-extract.c32 > /tmp/fsdr-test.bin && \
	csdr $(1)        < /tmp/fsdr-test.bin | head -c $(3)  > /tmp/csdr-output.bin && \
	$(FSDR_CLI) $(1) < /tmp/fsdr-test.bin | head -c $(3)  > /tmp/fsdr-output.bin   && \
	ls -al /tmp/fsdr-test.bin /tmp/fsdr-output.bin /tmp/csdr-output.bin && \
	cmp -l /tmp/csdr-output.bin /tmp/fsdr-output.bin

endef

define csdr_compare_cmd_c32

	export CSDR_FIXED_BUFSIZE=$(3) && \
	rm -f /tmp/fsdr-test.bin && \
	head -c $(2) ./tests/france-culture-extract.c32 > /tmp/fsdr-test.bin && \
	csdr $(1)        < /tmp/fsdr-test.bin | head -c $(3) > /tmp/csdr-output.bin && \
	$(FSDR_CLI) $(1) < /tmp/fsdr-test.bin | head -c $(3) > /tmp/fsdr-output.bin && \
	ls -al /tmp/fsdr-test.bin /tmp/fsdr-output.bin /tmp/csdr-output.bin && \
	cmp -l /tmp/csdr-output.bin /tmp/fsdr-output.bin

endef

csdr-compare-convert-u8-f: $(FSDR_CLI)
	$(call csdr_compare_cmd,convert_u8_f,128,128)

csdr-compare-realpart-c-f: $(FSDR_CLI)
	$(call csdr_compare_cmd,realpart_cf,1024,512)

csdr-compare-clone: $(FSDR_CLI)
	$(call csdr_compare_cmd,clone,32,32)

csdr-compare-dump-u8: $(FSDR_CLI)
	$(call csdr_compare_cmd,dump_u8,64,32)

csdr-compare-dump-f: $(FSDR_CLI)
	$(call csdr_compare_cmd,dump_f,64,16)

csdr-compare-shift-addition-cc : $(FSDR_CLI)
	$(call csdr_compare_cmd,shift_addition_cc 1256,1024,1024)

test-nfm: $(FSDR_CLI)
	cat tests/test-nfm.c32 | $(FSDR_CLI) csdr fir_decimate_cc 10 0.005 HAMMING ! fmdemod_quadri_cf ! limit_ff ! deemphasis_nfm_ff 48000 ! agc_ff ! convert_f_s16 | mplayer -cache 1024 -quiet -rawaudio samplesize=2:channels=1:rate=48000 -demuxer rawaudio -

test-nfm-load: $(FSDR_CLI)
	$(FSDR_CLI) csdr load_c tests/test-nfm.c32 ! fir_decimate_cc 10 0.005 HAMMING ! fmdemod_quadri_cf ! limit_ff ! deemphasis_nfm_ff 48000 ! agc_ff ! convert_f_s16 | mplayer -cache 1024 -quiet -rawaudio samplesize=2:channels=1:rate=48000 -demuxer rawaudio -

test-nfm-save: $(FSDR_CLI)
	$(FSDR_CLI) csdr --output ./test-nfm.grc fir_decimate_cc 10 0.005 HAMMING ! fmdemod_quadri_cf ! limit_ff ! deemphasis_nfm_ff 48000 ! agc_ff ! convert_f_s16

test-am: $(FSDR_CLI)
	$(FSDR_CLI) csdr load_u8 tests/test-am.u8 ! convert_u8_f ! convert_ff_c ! shift_addition_cc "((145000000-144400000)/2400000)" ! fir_decimate_cc 16 0.005 HAMMING ! amdemod_cf ! fastdcblock_ff ! agc_ff ! limit_ff ! convert_f_s16 | mplayer -cache 1024 -quiet -rawaudio samplesize=2:channels=1:rate=48000 -demuxer rawaudio -

test-am-grc: $(FSDR_CLI)
	$(FSDR_CLI) grc tests/am-demodulation.grc

run: ssb_lsb_256k_complex2.dat
	cargo run --release

tests/ssb_lsb_256k_complex2.dat.zip:
	cd tests && wget https://www.csun.edu/~skatz/katzpage/sdr_project/sdr/ssb_lsb_256k_complex2.dat.zip

tests/ssb_lsb_256k_complex2.dat: tests/ssb_lsb_256k_complex2.dat.zip
	cd tests && unzip -DD ssb_lsb_256k_complex2.dat.zip

test-ssb: $(FSDR_CLI) tests/ssb_lsb_256k_complex2.dat
	$(FSDR_CLI) csdr load_c tests/ssb_lsb_256k_complex2.dat ! shift_addition_cc "(-51500/256000)" ! fir_decimate_cc "(256000/48000)" 0.005 HAMMING ! bandpass_fir_fft_cc -0.1 0.0 0.05 ! realpart_cf ! agc_ff ! limit_ff ! convert_f_s16 | mplayer -cache 1024 -quiet -rawaudio samplesize=2:channels=1:rate=48000 -demuxer rawaudio -

test-ssb-debug: tests/ssb_lsb_256k_complex2.dat
	cargo run -- csdr load_c tests/ssb_lsb_256k_complex2.dat ! shift_addition_cc "(-51500/256000)" ! fir_decimate_cc "(256000/48000)" 0.005 HAMMING ! bandpass_fir_fft_cc 0.0 0.1 0.05 ! realpart_cf ! agc_ff ! limit_ff ! convert_f_s16 | mplayer -cache 1024 -quiet -rawaudio samplesize=2:channels=1:rate=48000 -demuxer rawaudio -

test-ssb-csdr:
	$(FSDR_CLI) csdr load_c tests/ssb_lsb_256k_complex2.dat ! csdr shift_addition_cc -0.201171875 ! csdr fir_decimate_cc 5 0.005 HAMMING ! csdr bandpass_fir_fft_cc 0.0 0.1 0.05 ! csdr realpart_cf ! csdr agc_ff ! csdr limit_ff ! csdr convert_f_s16 | mplayer -cache 1024 -quiet -rawaudio samplesize=2:channels=1:rate=48000 -demuxer rawaudio -

.PHONY: csdr-compare cargo-test csdr-compare-realpart-c-f csdr-compare-dump-u8