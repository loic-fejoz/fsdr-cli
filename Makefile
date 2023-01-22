FSDR_CLI:=./target/debug/fsdr-cli

target/debug/fsdr-cli: src/*.rs
	cargo build

test: cargo-test csdr-compare

cargo-test:
	cargo test

usage: $(FSDR_CLI)
	$(FSDR_CLI)

csdr-compare-convert-u8-f: target/debug/fsdr-cli
	head -c 16234 /dev/random > /tmp/fsdr-test.bin && \
	csdr convert_u8_f > /tmp/csdr-output.bin < /tmp/fsdr-test.bin && \
	$(FSDR_CLI) convert_u8_f > /tmp/fsdr-output.bin < /tmp/fsdr-test.bin && \
	cmp -l /tmp/csdr-output.bin /tmp/fsdr-output.bin

csdr-compare-realpart-c-f: target/debug/fsdr-cli
	head -c 16234 /dev/random > /tmp/fsdr-test.bin && \
	csdr realpart_cf > /tmp/csdr-output.bin < /tmp/fsdr-test.bin && \
	$(FSDR_CLI) realpart_cf > /tmp/fsdr-output.bin < /tmp/fsdr-test.bin && \
	cmp -l /tmp/csdr-output.bin /tmp/fsdr-output.bin

.PHONY: csdr-compare cargo-test