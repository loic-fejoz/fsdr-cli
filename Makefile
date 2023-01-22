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

define csdr_compare_cmd

	export CSDR_FIXED_BUFSIZE=$(3) && \
	head -c $(2) /dev/random > /tmp/fsdr-test.bin && \
	csdr $(1) > /tmp/csdr-output.bin < /tmp/fsdr-test.bin && \
	$(FSDR_CLI) $(1) > /tmp/fsdr-output.bin < /tmp/fsdr-test.bin && \
	cmp -l /tmp/csdr-output.bin /tmp/fsdr-output.bin

endef

csdr-compare-convert-u8-f: $(FSDR_CLI)
	$(call csdr_compare_cmd,convert_u8_f,32,128)

csdr-compare-realpart-c-f: $(FSDR_CLI)
	$(call csdr_compare_cmd,realpart_cf,128,16)

csdr-compare-clone: $(FSDR_CLI)
	$(call csdr_compare_cmd,clone,32,32)

csdr-compare-dump-u8: $(FSDR_CLI)
	$(call csdr_compare_cmd,dump_u8,32,32)

csdr-compare-dump-f: $(FSDR_CLI)
	$(call csdr_compare_cmd,dump_f,64,16)

.PHONY: csdr-compare cargo-test