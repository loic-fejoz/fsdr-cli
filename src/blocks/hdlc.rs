use anyhow::Result;
use futuresdr::runtime::dev::prelude::*;
use futuresdr::runtime::Pmt;
use std::cmp;

/// Standard CRC-16 CCITT (polynomial 0x8408 / reversed 0x1021)
pub fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc = 0xFFFFu16;
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0x8408;
            } else {
                crc >>= 1;
            }
        }
    }
    crc ^ 0xFFFF
}

/// HDLC / AX.25 Framing block: PDUs to byte stream with flags (0x7E), bit stuffing & CRC-16
#[derive(Block)]
#[message_inputs(pdu_in)]
pub struct HdlcFramingPduU8<O: CpuBufferWriter<Item = u8> = DefaultCpuWriter<u8>> {
    pending_bytes: Vec<u8>,
    #[output]
    output: O,
}

impl HdlcFramingPduU8<DefaultCpuWriter<u8>> {
    pub fn new() -> Self {
        Self {
            pending_bytes: Vec::new(),
            output: DefaultCpuWriter::default(),
        }
    }

    pub fn encode_frame(payload: &[u8]) -> Vec<u8> {
        let crc = crc16_ccitt(payload);
        let crc_low = (crc & 0xFF) as u8;
        let crc_high = ((crc >> 8) & 0xFF) as u8;

        let mut data_with_crc = Vec::with_capacity(payload.len() + 2);
        data_with_crc.extend_from_slice(payload);
        data_with_crc.push(crc_low);
        data_with_crc.push(crc_high);

        let mut bits = Vec::new();
        let mut consecutive_ones = 0;

        for &byte in &data_with_crc {
            for i in 0..8 {
                let bit = (byte >> i) & 1; // LSB first in HDLC
                bits.push(bit);
                if bit == 1 {
                    consecutive_ones += 1;
                    if consecutive_ones == 5 {
                        bits.push(0); // Bit stuffing
                        consecutive_ones = 0;
                    }
                } else {
                    consecutive_ones = 0;
                }
            }
        }

        let mut all_bits = Vec::new();
        let flag_bits = [0u8, 1, 1, 1, 1, 1, 1, 0]; // 0x7E in LSB first
        for _ in 0..2 {
            all_bits.extend_from_slice(&flag_bits);
        }
        all_bits.extend(bits);
        for _ in 0..2 {
            all_bits.extend_from_slice(&flag_bits);
        }

        let mut flag_idx = 0;
        while all_bits.len() % 8 != 0 {
            all_bits.push(flag_bits[flag_idx % 8]);
            flag_idx += 1;
        }

        let mut out = Vec::with_capacity(all_bits.len() / 8);
        for chunk in all_bits.chunks(8) {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                byte |= bit << i;
            }
            out.push(byte);
        }
        out
    }
}

impl Default for HdlcFramingPduU8<DefaultCpuWriter<u8>> {
    fn default() -> Self {
        Self::new()
    }
}

impl<O> HdlcFramingPduU8<O>
where
    O: CpuBufferWriter<Item = u8>,
{
    async fn pdu_in(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
        p: Pmt,
    ) -> Result<Pmt> {
        if let Pmt::Blob(data) = p {
            let encoded = HdlcFramingPduU8::encode_frame(&data);
            self.pending_bytes.extend(encoded);
            io.call_again = true;
        }
        Ok(Pmt::Ok)
    }
}

#[doc(hidden)]
impl<O> Kernel for HdlcFramingPduU8<O>
where
    O: CpuBufferWriter<Item = u8>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let out_slice = self.output.slice();

        if !self.pending_bytes.is_empty() && !out_slice.is_empty() {
            let n = cmp::min(self.pending_bytes.len(), out_slice.len());
            out_slice[..n].copy_from_slice(&self.pending_bytes[..n]);
            self.pending_bytes.drain(..n);
            self.output.produce(n);
            if !self.pending_bytes.is_empty() {
                io.call_again = true;
            }
        }

        Ok(())
    }
}

/// HDLC / AX.25 Deframing block: bit destuffing, flag detect, CRC-16 check, emits PDUs
#[derive(Block)]
#[message_outputs(pdu_out)]
pub struct HdlcDeframingU8Pdu<I: CpuBufferReader<Item = u8> = DefaultCpuReader<u8>> {
    in_frame: bool,
    consecutive_ones: usize,
    expect_closing_flag_zero: bool,
    raw_bits: Vec<u8>,
    #[input]
    input: I,
}

impl HdlcDeframingU8Pdu<DefaultCpuReader<u8>> {
    pub fn new() -> Self {
        Self {
            in_frame: false,
            consecutive_ones: 0,
            expect_closing_flag_zero: false,
            raw_bits: Vec::new(),
            input: DefaultCpuReader::default(),
        }
    }
}

impl Default for HdlcDeframingU8Pdu<DefaultCpuReader<u8>> {
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
impl<I> Kernel for HdlcDeframingU8Pdu<I>
where
    I: CpuBufferReader<Item = u8>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let in_slice = self.input.slice();

        if !in_slice.is_empty() {
            for &byte in in_slice {
                for bit_idx in 0..8 {
                    let bit = (byte >> bit_idx) & 1;

                    if self.expect_closing_flag_zero {
                        self.expect_closing_flag_zero = false;
                        if bit == 0 {
                            self.consecutive_ones = 0;
                            continue;
                        }
                    }

                    if bit == 1 {
                        self.consecutive_ones += 1;
                        if self.consecutive_ones == 6 {
                            self.expect_closing_flag_zero = true;
                            if self.in_frame && self.raw_bits.len() >= 6 {
                                let bits_len = self.raw_bits.len().saturating_sub(6);
                                let payload_bits = &self.raw_bits[..bits_len];

                                if payload_bits.len() >= 24 && payload_bits.len().is_multiple_of(8)
                                {
                                    let mut decoded_bytes = Vec::new();
                                    for chunk in payload_bits.chunks(8) {
                                        let mut b = 0u8;
                                        for (k, &cb) in chunk.iter().enumerate() {
                                            b |= cb << k;
                                        }
                                        decoded_bytes.push(b);
                                    }

                                    if decoded_bytes.len() >= 3 {
                                        let data_len = decoded_bytes.len() - 2;
                                        let payload = &decoded_bytes[..data_len];
                                        let rx_crc = (decoded_bytes[data_len] as u16)
                                            | ((decoded_bytes[data_len + 1] as u16) << 8);

                                        if crc16_ccitt(payload) == rx_crc {
                                            mio.post("pdu_out", Pmt::Blob(payload.to_vec()))
                                                .await?;
                                        }
                                    }
                                }
                            }
                            self.in_frame = true;
                            self.raw_bits.clear();
                            self.consecutive_ones = 0;
                            continue;
                        }
                        self.raw_bits.push(bit);
                    } else {
                        // bit == 0
                        if self.consecutive_ones == 5 {
                            // Destuff: drop stuffed 0
                            self.consecutive_ones = 0;
                            continue;
                        }
                        self.consecutive_ones = 0;
                        if self.in_frame {
                            self.raw_bits.push(bit);
                        }
                    }
                }
            }
            let len = in_slice.len();
            self.input.consume(len);
        }

        if !self.input.slice().is_empty() {
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
    fn test_crc16_and_frame_encode() {
        let payload = b"HELLO";
        let crc = crc16_ccitt(payload);
        assert_ne!(crc, 0);

        let frame = HdlcFramingPduU8::encode_frame(payload);
        assert!(frame.len() > payload.len());
        assert_eq!(frame[0], 0x7E);
        assert_eq!(*frame.last().unwrap(), 0x7E);
    }

    #[test]
    fn test_hdlc_encode_decode_roundtrip() {
        let payload = b"HELLO_WORLD_HDLC_12345";
        let encoded = HdlcFramingPduU8::encode_frame(payload);

        let mut consecutive_ones = 0;
        let mut in_frame = false;
        let mut expect_closing_flag_zero = false;
        let mut raw_bits = Vec::new();
        let mut decoded_payloads = Vec::new();

        for &byte in &encoded {
            for bit_idx in 0..8 {
                let bit = (byte >> bit_idx) & 1;

                if expect_closing_flag_zero {
                    expect_closing_flag_zero = false;
                    if bit == 0 {
                        consecutive_ones = 0;
                        continue;
                    }
                }

                if bit == 1 {
                    consecutive_ones += 1;
                    if consecutive_ones == 6 {
                        expect_closing_flag_zero = true;
                        if in_frame && raw_bits.len() >= 6 {
                            let bits_len = raw_bits.len().saturating_sub(6);
                            let payload_bits = &raw_bits[..bits_len];
                            if payload_bits.len() >= 24 && payload_bits.len().is_multiple_of(8) {
                                let mut decoded_bytes = Vec::new();
                                for chunk in payload_bits.chunks(8) {
                                    let mut b = 0u8;
                                    for (k, &cb) in chunk.iter().enumerate() {
                                        b |= cb << k;
                                    }
                                    decoded_bytes.push(b);
                                }
                                if decoded_bytes.len() >= 3 {
                                    let data_len = decoded_bytes.len() - 2;
                                    let p = &decoded_bytes[..data_len];
                                    let rx_crc = (decoded_bytes[data_len] as u16)
                                        | ((decoded_bytes[data_len + 1] as u16) << 8);
                                    if crc16_ccitt(p) == rx_crc {
                                        decoded_payloads.push(p.to_vec());
                                    }
                                }
                            }
                        }
                        in_frame = true;
                        raw_bits.clear();
                        consecutive_ones = 0;
                        continue;
                    }
                    raw_bits.push(bit);
                } else {
                    if consecutive_ones == 5 {
                        consecutive_ones = 0;
                        continue;
                    }
                    consecutive_ones = 0;
                    if in_frame {
                        raw_bits.push(bit);
                    }
                }
            }
        }

        assert_eq!(
            decoded_payloads.len(),
            1,
            "Expected exactly 1 decoded payload"
        );
        assert_eq!(&decoded_payloads[0], payload);
    }

    #[derive(Block)]
    #[message_inputs(in_port)]
    struct TestPduSink {
        messages: std::sync::Arc<std::sync::Mutex<Vec<Vec<u8>>>>,
    }

    impl TestPduSink {
        fn new(messages: std::sync::Arc<std::sync::Mutex<Vec<Vec<u8>>>>) -> Self {
            Self { messages }
        }

        async fn in_port(
            &mut self,
            _io: &mut WorkIo,
            _mio: &mut MessageOutputs,
            _meta: &BlockMeta,
            p: Pmt,
        ) -> Result<Pmt> {
            if let Pmt::Blob(bytes) = p {
                self.messages.lock().unwrap().push(bytes);
            }
            Ok(Pmt::Null)
        }
    }

    #[doc(hidden)]
    impl Kernel for TestPduSink {}

    #[test]
    fn test_hdlc_flowgraph_roundtrip() -> Result<()> {
        use futuresdr::blocks::MessageSource;
        use futuresdr::runtime::Runtime;
        use std::sync::{Arc, Mutex};
        use std::thread;
        use std::time::Duration;

        let mut fg = Flowgraph::new();
        let payload = b"FSD_AX25_TEST".to_vec();

        let src = fg
            .add(MessageSource::new(
                Pmt::Blob(payload.clone()),
                Duration::from_millis(20),
                Some(1),
            ))?
            .id();

        let framer = fg.add(HdlcFramingPduU8::new())?.id();
        let deframer = fg.add(HdlcDeframingU8Pdu::new())?.id();

        let received = Arc::new(Mutex::new(Vec::new()));
        let sink = fg.add(TestPduSink::new(received.clone()))?.id();

        // 1. Message source -> Framer PDU in
        fg.message(src, "out", framer, "pdu_in")?;
        // 2. Framer stream output -> Deframer stream input (verifies no unlinked stream ports on Framer/Deframer!)
        fg.stream_dyn(framer, "output", deframer, "input")?;
        // 3. Deframer PDU out -> Sink
        fg.message(deframer, "pdu_out", sink, "in_port")?;

        let handle = thread::spawn(move || {
            let _ = Runtime::new().run(fg);
        });

        // Verify reception of decoded PDU
        let mut got_payload = false;
        for _ in 0..40 {
            if received.lock().unwrap().contains(&payload) {
                got_payload = true;
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }

        drop(handle);
        assert!(
            got_payload,
            "Expected to receive decoded PDU matching sent payload"
        );
        Ok(())
    }
}
