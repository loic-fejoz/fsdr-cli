# CSDR Command Implementation Roadmap

This document outlines the implementation priority for the remaining `csdr` retrocompatibility commands in `fsdr-cli`. The prioritization is based on their frequency of use in SDR receiver pipelines (specifically **OpenWebRx**), core DSP utility, and flowgraph execution efficiency.

---

## 📊 Summary of Priority Tiers

| Priority Tier | Commands | Primary Use-Case | Rationale |
| :--- | :--- | :--- | :--- |
| **Tier 1: High** | `fft_cc`, `fft_fc`, `logpower_cf`, `logaveragepower_cf`, `fft_exchange_sides_ff` | Waterfall & Spectrum display | Required by OpenWebRx for rendering the real-time waterfall display. Without these, `fsdr-cli` cannot fully replace `csdr` in a server backend. |
| **Tier 2: Medium-High** | `dcblock_ff`, `decimating_shift_addition_cc`, `firdes_lowpass_f`, `firdes_bandpass_c` | Demodulation & Filtering | Standard DSP operations and optimizations (like combined frequency shifting and decimation) that improve CPU efficiency. |
| **Tier 3: Medium** | `clone`, `tee`, `flowcontrol`, `pack_bits_1to8_u8_u8` | Stream Routing & Flow Control | Crucial for multi-stream pipelines (branching flowgraphs) and offline data processing/throttling. |
| **Tier 4: Low** | `pll_cc`, `bpsk_costas_loop_cc`, `dbpsk_decoder_c_u8`, `bfsk_demod_cf`, digital decoders | Digital Demodulation & Coding | Specialized digital mode blocks. Often handled by downstream software (e.g., `multimon-ng` or `direwolf`) rather than within the core `csdr` pipeline. |

---

## 🔍 Detailed Analysis & Rationale

### Tier 1: Spectrum & Waterfall Support (High Priority)
OpenWebRx uses `csdr` pipelines to feed the web interface waterfall. The standard pipeline is:
`... | csdr fft_cc <fft_size> | csdr logpower_cf | csdr fft_exchange_sides_ff`

*   **`fft_cc` / `fft_fc`**: Essential for frequency-domain representation.
*   **`logpower_cf` / `logaveragepower_cf`**: Converts complex FFT output into logarithmic Power Spectral Density (PSD) values (dB scale).
*   **`fft_exchange_sides_ff`**: Shifts the DC component to the center of the FFT vector, allowing direct rendering of the RF spectrum.

*Implementing these allows `fsdr-cli` to immediately serve as a backend replacement for OpenWebRx spectrum generation.*

---

### Tier 2: Demodulation & DSP Optimizations (Medium-High Priority)
These commands optimize core DSP performance or are standard signal-conditioning steps.

*   **`decimating_shift_addition_cc`**: Combines shifting and decimation. Shifting and decimating at high sample rates is extremely CPU intensive. Performing them in a single combined kernel avoids processing samples that will eventually be discarded.
*   **`dcblock_ff`**: Standard DC blocker. Highly used in audio chains to remove residual DC offsets from FM/AM demodulators that cause clicking or biasing.
*   **`firdes_lowpass_f` / `firdes_bandpass_c`**: Helper commands to generate filter taps. In a unified flowgraph tool, this can be resolved either as CLI tools or evaluated dynamically inside the block parameters.

---

### Tier 3: Routing & Flow Control (Medium Priority)
These blocks help match complex pipeline typologies.

*   **`clone` / `tee`**: Allows branching a pipeline so one input feeds multiple demodulators or outputs (e.g., audio output + file recording).
*   **`flowcontrol`**: Limits sample rates for offline processing/simulations so the pipeline doesn't consume 100% CPU.
*   **`pack_bits_1to8_u8_u8`**: Inverse of `pack_bits_8to1_u8_u8` (already implemented). Crucial for bit-sliced stream serialization.

---

### Tier 4: Digital Demodulation & Specific Codecs (Low Priority)
These blocks implement specific digital communications physical/link layer logic.

*   **`pll_cc` / `bpsk_costas_loop_cc`**: Phase lock loops for carrier recovery.
*   **`dbpsk_decoder_c_u8` / `bfsk_demod_cf` / `generic_slicer_f_u8`**: Specialized demodulators.
*   **`psk31_*` / `rtty_*`**: Mode-specific ASCII text decoders.

*Since modern users typically feed IQ or demodulated FM/AM audio into highly specialized decoders (like `direwolf` for APRS, or WSJT-X for FT8), implementing these in the core `fsdr-cli` is less urgent.*
