use anyhow::Result;
use futuresdr::runtime::dev::prelude::*;
use futuresdr::runtime::Pmt;
use std::cmp;

pub struct G3ruhScrambler {
    state: u32,
    poly: u32,
}

impl G3ruhScrambler {
    pub const POLY: u32 = (1 << 11) | (1 << 16); // 0x10800
    pub const SEED: u32 = 0x1FFFF;

    pub fn new() -> Self {
        Self {
            state: Self::SEED,
            poly: Self::POLY,
        }
    }

    pub fn scramble_bit(&mut self, bit: u8) -> u8 {
        let xor_sum = (self.state & self.poly).count_ones() % 2;
        let out_bit = (bit as u32 ^ xor_sum) as u8 & 1;
        self.state = (self.state << 1) | (out_bit as u32);
        out_bit
    }

    pub fn scramble_byte(&mut self, byte: u8) -> u8 {
        let mut out = 0u8;
        for i in 0..8 {
            let bit = (byte >> (7 - i)) & 1;
            let scr = self.scramble_bit(bit);
            out |= scr << (7 - i);
        }
        out
    }
}

impl Default for G3ruhScrambler {
    fn default() -> Self {
        Self::new()
    }
}

pub struct G3ruhDescrambler {
    state: u32,
    poly: u32,
}

impl G3ruhDescrambler {
    pub fn new() -> Self {
        Self {
            state: G3ruhScrambler::SEED,
            poly: G3ruhScrambler::POLY,
        }
    }

    pub fn descramble_bit(&mut self, bit: u8) -> u8 {
        let xor_sum = (self.state & self.poly).count_ones() % 2;
        let out_bit = (bit as u32 ^ xor_sum) as u8 & 1;
        self.state = (self.state << 1) | (bit as u32);
        out_bit
    }

    pub fn descramble_byte(&mut self, byte: u8) -> u8 {
        let mut out = 0u8;
        for i in 0..8 {
            let bit = (byte >> (7 - i)) & 1;
            let descr = self.descramble_bit(bit);
            out |= descr << (7 - i);
        }
        out
    }
}

impl Default for G3ruhDescrambler {
    fn default() -> Self {
        Self::new()
    }
}

/// G3RUH Framing block: accepts bytes / PDUs, scrambles and adds framing
#[derive(Block)]
#[message_inputs(pdu_in)]
pub struct PduFramingG3ruh<
    I: CpuBufferReader<Item = u8> = DefaultCpuReader<u8>,
    O: CpuBufferWriter<Item = u8> = DefaultCpuWriter<u8>,
> {
    scrambler: G3ruhScrambler,
    pending_bytes: Vec<u8>,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl PduFramingG3ruh<DefaultCpuReader<u8>, DefaultCpuWriter<u8>> {
    pub fn new() -> Self {
        Self {
            scrambler: G3ruhScrambler::new(),
            pending_bytes: Vec::new(),
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for PduFramingG3ruh<DefaultCpuReader<u8>, DefaultCpuWriter<u8>> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I, O> PduFramingG3ruh<I, O>
where
    I: CpuBufferReader<Item = u8>,
    O: CpuBufferWriter<Item = u8>,
{
    async fn pdu_in(
        &mut self,
        _io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
        p: Pmt,
    ) -> Result<Pmt> {
        if let Pmt::Blob(data) = p {
            // Preamble (0xAA x 4) + Sync word (0x7E) + Scrambled payload + Ending flag (0x7E)
            let mut frame = vec![0xAA, 0xAA, 0xAA, 0xAA, 0x7E];
            for byte in data {
                frame.push(self.scrambler.scramble_byte(byte));
            }
            frame.push(0x7E);
            self.pending_bytes.extend(frame);
        }
        Ok(Pmt::Ok)
    }
}

#[doc(hidden)]
impl<I, O> Kernel for PduFramingG3ruh<I, O>
where
    I: CpuBufferReader<Item = u8>,
    O: CpuBufferWriter<Item = u8>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let out_slice = self.output.slice();

        // Flush pending bytes from PDU message input
        if !self.pending_bytes.is_empty() && !out_slice.is_empty() {
            let n = cmp::min(self.pending_bytes.len(), out_slice.len());
            out_slice[..n].copy_from_slice(&self.pending_bytes[..n]);
            self.pending_bytes.drain(..n);
            self.output.produce(n);
            io.call_again = true;
            return Ok(());
        }

        // Stream mode
        let in_slice = self.input.slice();
        let m = cmp::min(in_slice.len(), out_slice.len());
        if m > 0 {
            for (src, dst) in in_slice[..m].iter().zip(out_slice[..m].iter_mut()) {
                *dst = self.scrambler.scramble_byte(*src);
            }
            self.input.consume(m);
            self.output.produce(m);
        }

        if !self.input.slice().is_empty() && !self.output.slice().is_empty() {
            io.call_again = true;
        }

        if self.input.slice().is_empty() && self.input.finished() && self.pending_bytes.is_empty() {
            io.finished = true;
        }

        Ok(())
    }
}

/// G3RUH Deframing block: descrambles stream / detects frames and posts PDUs
#[derive(Block)]
#[message_outputs(pdu_out)]
pub struct PduDeframingG3ruh<
    I: CpuBufferReader<Item = u8> = DefaultCpuReader<u8>,
    O: CpuBufferWriter<Item = u8> = DefaultCpuWriter<u8>,
> {
    descrambler: G3ruhDescrambler,
    in_frame: bool,
    frame_buf: Vec<u8>,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl PduDeframingG3ruh<DefaultCpuReader<u8>, DefaultCpuWriter<u8>> {
    pub fn new() -> Self {
        Self {
            descrambler: G3ruhDescrambler::new(),
            in_frame: false,
            frame_buf: Vec::new(),
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for PduDeframingG3ruh<DefaultCpuReader<u8>, DefaultCpuWriter<u8>> {
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
impl<I, O> Kernel for PduDeframingG3ruh<I, O>
where
    I: CpuBufferReader<Item = u8>,
    O: CpuBufferWriter<Item = u8>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let in_slice = self.input.slice();
        let out_slice = self.output.slice();

        let m = cmp::min(in_slice.len(), out_slice.len());
        if m > 0 {
            for i in 0..m {
                let raw_byte = in_slice[i];
                let descr = self.descrambler.descramble_byte(raw_byte);
                out_slice[i] = descr;

                if descr == 0x7E {
                    if self.in_frame && !self.frame_buf.is_empty() {
                        let pkt = std::mem::take(&mut self.frame_buf);
                        mio.post("pdu_out", Pmt::Blob(pkt)).await?;
                    }
                    self.in_frame = true;
                } else if self.in_frame {
                    self.frame_buf.push(descr);
                    if self.frame_buf.len() > 2048 {
                        // Frame too long, discard
                        self.frame_buf.clear();
                        self.in_frame = false;
                    }
                }
            }
            self.input.consume(m);
            self.output.produce(m);
        }

        if !self.input.slice().is_empty() && !self.output.slice().is_empty() {
            io.call_again = true;
        }

        if self.input.slice().is_empty() && self.input.finished() {
            io.finished = true;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_g3ruh_roundtrip() {
        let mut scr = G3ruhScrambler::new();
        let mut descr = G3ruhDescrambler::new();

        let data = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let mut scrambled = Vec::new();
        for &b in &data {
            scrambled.push(scr.scramble_byte(b));
        }

        assert_ne!(scrambled, data);

        let mut recovered = Vec::new();
        for &b in &scrambled {
            recovered.push(descr.descramble_byte(b));
        }

        assert_eq!(recovered, data);
    }
}
