use anyhow::Result;
use futuresdr::runtime::dev::prelude::*;

pub struct VaricodeItem {
    pub code: u64,
    pub bitcount: usize,
    pub ascii: u8,
}

// Canonical Varicode Table (ASCII 0..127)
pub const VARICODE_TABLE: &[VaricodeItem] = &[
    VaricodeItem {
        code: 0b1010101011,
        bitcount: 10,
        ascii: 0x00,
    },
    VaricodeItem {
        code: 0b1011011011,
        bitcount: 10,
        ascii: 0x01,
    },
    VaricodeItem {
        code: 0b1011101101,
        bitcount: 10,
        ascii: 0x02,
    },
    VaricodeItem {
        code: 0b1101110111,
        bitcount: 10,
        ascii: 0x03,
    },
    VaricodeItem {
        code: 0b1011101011,
        bitcount: 10,
        ascii: 0x04,
    },
    VaricodeItem {
        code: 0b1101011111,
        bitcount: 10,
        ascii: 0x05,
    },
    VaricodeItem {
        code: 0b1011101111,
        bitcount: 10,
        ascii: 0x06,
    },
    VaricodeItem {
        code: 0b1011111101,
        bitcount: 10,
        ascii: 0x07,
    },
    VaricodeItem {
        code: 0b1011111111,
        bitcount: 10,
        ascii: 0x08,
    },
    VaricodeItem {
        code: 0b11101111,
        bitcount: 8,
        ascii: 0x09,
    },
    VaricodeItem {
        code: 0b11101,
        bitcount: 5,
        ascii: 0x0a,
    },
    VaricodeItem {
        code: 0b1101101111,
        bitcount: 10,
        ascii: 0x0b,
    },
    VaricodeItem {
        code: 0b1011011101,
        bitcount: 10,
        ascii: 0x0c,
    },
    VaricodeItem {
        code: 0b11111,
        bitcount: 5,
        ascii: 0x0d,
    },
    VaricodeItem {
        code: 0b1101110101,
        bitcount: 10,
        ascii: 0x0e,
    },
    VaricodeItem {
        code: 0b1110101011,
        bitcount: 10,
        ascii: 0x0f,
    },
    VaricodeItem {
        code: 0b1011110111,
        bitcount: 10,
        ascii: 0x10,
    },
    VaricodeItem {
        code: 0b1011110101,
        bitcount: 10,
        ascii: 0x11,
    },
    VaricodeItem {
        code: 0b1110101101,
        bitcount: 10,
        ascii: 0x12,
    },
    VaricodeItem {
        code: 0b1110101111,
        bitcount: 10,
        ascii: 0x13,
    },
    VaricodeItem {
        code: 0b1101011011,
        bitcount: 10,
        ascii: 0x14,
    },
    VaricodeItem {
        code: 0b1101101011,
        bitcount: 10,
        ascii: 0x15,
    },
    VaricodeItem {
        code: 0b1101101101,
        bitcount: 10,
        ascii: 0x16,
    },
    VaricodeItem {
        code: 0b1101010111,
        bitcount: 10,
        ascii: 0x17,
    },
    VaricodeItem {
        code: 0b1101111011,
        bitcount: 10,
        ascii: 0x18,
    },
    VaricodeItem {
        code: 0b1101111101,
        bitcount: 10,
        ascii: 0x19,
    },
    VaricodeItem {
        code: 0b1110110111,
        bitcount: 10,
        ascii: 0x1a,
    },
    VaricodeItem {
        code: 0b1101010101,
        bitcount: 10,
        ascii: 0x1b,
    },
    VaricodeItem {
        code: 0b1101011101,
        bitcount: 10,
        ascii: 0x1c,
    },
    VaricodeItem {
        code: 0b1110111011,
        bitcount: 10,
        ascii: 0x1d,
    },
    VaricodeItem {
        code: 0b1011111011,
        bitcount: 10,
        ascii: 0x1e,
    },
    VaricodeItem {
        code: 0b1101111111,
        bitcount: 10,
        ascii: 0x1f,
    },
    VaricodeItem {
        code: 0b1,
        bitcount: 1,
        ascii: 0x20,
    },
    VaricodeItem {
        code: 0b111111111,
        bitcount: 9,
        ascii: 0x21,
    },
    VaricodeItem {
        code: 0b101011111,
        bitcount: 9,
        ascii: 0x22,
    },
    VaricodeItem {
        code: 0b111110101,
        bitcount: 9,
        ascii: 0x23,
    },
    VaricodeItem {
        code: 0b111011011,
        bitcount: 9,
        ascii: 0x24,
    },
    VaricodeItem {
        code: 0b1011010101,
        bitcount: 10,
        ascii: 0x25,
    },
    VaricodeItem {
        code: 0b1010111011,
        bitcount: 10,
        ascii: 0x26,
    },
    VaricodeItem {
        code: 0b101111111,
        bitcount: 9,
        ascii: 0x27,
    },
    VaricodeItem {
        code: 0b11111011,
        bitcount: 8,
        ascii: 0x28,
    },
    VaricodeItem {
        code: 0b11110111,
        bitcount: 8,
        ascii: 0x29,
    },
    VaricodeItem {
        code: 0b101101111,
        bitcount: 9,
        ascii: 0x2a,
    },
    VaricodeItem {
        code: 0b111011111,
        bitcount: 9,
        ascii: 0x2b,
    },
    VaricodeItem {
        code: 0b1110101,
        bitcount: 7,
        ascii: 0x2c,
    },
    VaricodeItem {
        code: 0b110101,
        bitcount: 6,
        ascii: 0x2d,
    },
    VaricodeItem {
        code: 0b1010111,
        bitcount: 7,
        ascii: 0x2e,
    },
    VaricodeItem {
        code: 0b110101111,
        bitcount: 9,
        ascii: 0x2f,
    },
    VaricodeItem {
        code: 0b10110111,
        bitcount: 8,
        ascii: 0x30,
    },
    VaricodeItem {
        code: 0b10111101,
        bitcount: 8,
        ascii: 0x31,
    },
    VaricodeItem {
        code: 0b11101101,
        bitcount: 8,
        ascii: 0x32,
    },
    VaricodeItem {
        code: 0b11111111,
        bitcount: 8,
        ascii: 0x33,
    },
    VaricodeItem {
        code: 0b101110111,
        bitcount: 9,
        ascii: 0x34,
    },
    VaricodeItem {
        code: 0b101011011,
        bitcount: 9,
        ascii: 0x35,
    },
    VaricodeItem {
        code: 0b101101011,
        bitcount: 9,
        ascii: 0x36,
    },
    VaricodeItem {
        code: 0b110101101,
        bitcount: 9,
        ascii: 0x37,
    },
    VaricodeItem {
        code: 0b110101011,
        bitcount: 9,
        ascii: 0x38,
    },
    VaricodeItem {
        code: 0b110110111,
        bitcount: 9,
        ascii: 0x39,
    },
    VaricodeItem {
        code: 0b11110101,
        bitcount: 8,
        ascii: 0x3a,
    },
    VaricodeItem {
        code: 0b110111101,
        bitcount: 9,
        ascii: 0x3b,
    },
    VaricodeItem {
        code: 0b1110110101,
        bitcount: 10,
        ascii: 0x3c,
    },
    VaricodeItem {
        code: 0b1010101,
        bitcount: 7,
        ascii: 0x3d,
    },
    VaricodeItem {
        code: 0b1110101101,
        bitcount: 10,
        ascii: 0x3e,
    },
    VaricodeItem {
        code: 0b1010101101,
        bitcount: 10,
        ascii: 0x3f,
    },
    VaricodeItem {
        code: 0b1010111101,
        bitcount: 10,
        ascii: 0x40,
    },
    VaricodeItem {
        code: 0b1111101,
        bitcount: 7,
        ascii: 0x41,
    },
    VaricodeItem {
        code: 0b11101011,
        bitcount: 8,
        ascii: 0x42,
    },
    VaricodeItem {
        code: 0b10101101,
        bitcount: 8,
        ascii: 0x43,
    },
    VaricodeItem {
        code: 0b10110101,
        bitcount: 8,
        ascii: 0x44,
    },
    VaricodeItem {
        code: 0b1110111,
        bitcount: 7,
        ascii: 0x45,
    },
    VaricodeItem {
        code: 0b11011011,
        bitcount: 8,
        ascii: 0x46,
    },
    VaricodeItem {
        code: 0b11111101,
        bitcount: 8,
        ascii: 0x47,
    },
    VaricodeItem {
        code: 0b101010101,
        bitcount: 9,
        ascii: 0x48,
    },
    VaricodeItem {
        code: 0b1111111,
        bitcount: 7,
        ascii: 0x49,
    },
    VaricodeItem {
        code: 0b111111101,
        bitcount: 9,
        ascii: 0x4a,
    },
    VaricodeItem {
        code: 0b101111101,
        bitcount: 9,
        ascii: 0x4b,
    },
    VaricodeItem {
        code: 0b11010111,
        bitcount: 8,
        ascii: 0x4c,
    },
    VaricodeItem {
        code: 0b10111011,
        bitcount: 8,
        ascii: 0x4d,
    },
    VaricodeItem {
        code: 0b11011101,
        bitcount: 8,
        ascii: 0x4e,
    },
    VaricodeItem {
        code: 0b10101011,
        bitcount: 8,
        ascii: 0x4f,
    },
    VaricodeItem {
        code: 0b11010101,
        bitcount: 8,
        ascii: 0x50,
    },
    VaricodeItem {
        code: 0b111011101,
        bitcount: 9,
        ascii: 0x51,
    },
    VaricodeItem {
        code: 0b10101111,
        bitcount: 8,
        ascii: 0x52,
    },
    VaricodeItem {
        code: 0b1101111,
        bitcount: 7,
        ascii: 0x53,
    },
    VaricodeItem {
        code: 0b1101101,
        bitcount: 7,
        ascii: 0x54,
    },
    VaricodeItem {
        code: 0b101010111,
        bitcount: 9,
        ascii: 0x55,
    },
    VaricodeItem {
        code: 0b110110101,
        bitcount: 9,
        ascii: 0x56,
    },
    VaricodeItem {
        code: 0b101011101,
        bitcount: 9,
        ascii: 0x57,
    },
    VaricodeItem {
        code: 0b101110101,
        bitcount: 9,
        ascii: 0x58,
    },
    VaricodeItem {
        code: 0b101111011,
        bitcount: 9,
        ascii: 0x59,
    },
    VaricodeItem {
        code: 0b1010101111,
        bitcount: 10,
        ascii: 0x5a,
    },
    VaricodeItem {
        code: 0b111110111,
        bitcount: 9,
        ascii: 0x5b,
    },
    VaricodeItem {
        code: 0b111101111,
        bitcount: 9,
        ascii: 0x5c,
    },
    VaricodeItem {
        code: 0b111111011,
        bitcount: 9,
        ascii: 0x5d,
    },
    VaricodeItem {
        code: 0b1010111101,
        bitcount: 10,
        ascii: 0x5e,
    },
    VaricodeItem {
        code: 0b101101101,
        bitcount: 9,
        ascii: 0x5f,
    },
    VaricodeItem {
        code: 0b1011011111,
        bitcount: 10,
        ascii: 0x60,
    },
    VaricodeItem {
        code: 0b1011,
        bitcount: 4,
        ascii: 0x61,
    },
    VaricodeItem {
        code: 0b1011111,
        bitcount: 7,
        ascii: 0x62,
    },
    VaricodeItem {
        code: 0b101111,
        bitcount: 6,
        ascii: 0x63,
    },
    VaricodeItem {
        code: 0b101101,
        bitcount: 6,
        ascii: 0x64,
    },
    VaricodeItem {
        code: 0b11,
        bitcount: 2,
        ascii: 0x65,
    },
    VaricodeItem {
        code: 0b111101,
        bitcount: 6,
        ascii: 0x66,
    },
    VaricodeItem {
        code: 0b111111,
        bitcount: 6,
        ascii: 0x67,
    },
    VaricodeItem {
        code: 0b111011,
        bitcount: 6,
        ascii: 0x68,
    },
    VaricodeItem {
        code: 0b1101,
        bitcount: 4,
        ascii: 0x69,
    },
    VaricodeItem {
        code: 0b111010111,
        bitcount: 9,
        ascii: 0x6a,
    },
    VaricodeItem {
        code: 0b10111011,
        bitcount: 8,
        ascii: 0x6b,
    },
    VaricodeItem {
        code: 0b11011,
        bitcount: 5,
        ascii: 0x6c,
    },
    VaricodeItem {
        code: 0b1110101,
        bitcount: 7,
        ascii: 0x6d,
    },
    VaricodeItem {
        code: 0b1111,
        bitcount: 4,
        ascii: 0x6e,
    },
    VaricodeItem {
        code: 0b111,
        bitcount: 3,
        ascii: 0x6f,
    },
    VaricodeItem {
        code: 0b1111011,
        bitcount: 7,
        ascii: 0x70,
    },
    VaricodeItem {
        code: 0b110111011,
        bitcount: 9,
        ascii: 0x71,
    },
    VaricodeItem {
        code: 0b10101,
        bitcount: 5,
        ascii: 0x72,
    },
    VaricodeItem {
        code: 0b10111,
        bitcount: 5,
        ascii: 0x73,
    },
    VaricodeItem {
        code: 0b101,
        bitcount: 3,
        ascii: 0x74,
    },
    VaricodeItem {
        code: 0b110111,
        bitcount: 6,
        ascii: 0x75,
    },
    VaricodeItem {
        code: 0b11110101,
        bitcount: 8,
        ascii: 0x76,
    },
    VaricodeItem {
        code: 0b1101011,
        bitcount: 7,
        ascii: 0x77,
    },
    VaricodeItem {
        code: 0b11011111,
        bitcount: 8,
        ascii: 0x78,
    },
    VaricodeItem {
        code: 0b1011101,
        bitcount: 7,
        ascii: 0x79,
    },
    VaricodeItem {
        code: 0b1110111011,
        bitcount: 10,
        ascii: 0x7a,
    },
    VaricodeItem {
        code: 0b1010110111,
        bitcount: 10,
        ascii: 0x7b,
    },
    VaricodeItem {
        code: 0b110111011,
        bitcount: 9,
        ascii: 0x7c,
    },
    VaricodeItem {
        code: 0b1010110101,
        bitcount: 10,
        ascii: 0x7d,
    },
    VaricodeItem {
        code: 0b1011010111,
        bitcount: 10,
        ascii: 0x7e,
    },
    VaricodeItem {
        code: 0b1110110101,
        bitcount: 10,
        ascii: 0x7f,
    },
];

#[derive(Block)]
pub struct VaricodeDecoder<
    I: CpuBufferReader<Item = u8> = DefaultCpuReader<u8>,
    O: CpuBufferWriter<Item = u8> = DefaultCpuWriter<u8>,
> {
    status: u64,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl VaricodeDecoder<DefaultCpuReader<u8>, DefaultCpuWriter<u8>> {
    pub fn new() -> Self {
        Self {
            status: 0,
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for VaricodeDecoder<DefaultCpuReader<u8>, DefaultCpuWriter<u8>> {
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
impl<I, O> Kernel for VaricodeDecoder<I, O>
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
        let i = self.input.slice();
        let o = self.output.slice();

        let mut consumed = 0;
        let mut produced = 0;

        while consumed < i.len() && produced < o.len() {
            let symbol = i[consumed];
            consumed += 1;

            self.status = (self.status << 1) | (symbol as u64 & 0b1);

            if (self.status & 0xFFF) == 0 {
                continue;
            }

            for item in VARICODE_TABLE {
                let mask = (1u64 << (item.bitcount + 4)) - 1;
                if (item.code << 2) == (self.status & mask) {
                    o[produced] = item.ascii;
                    produced += 1;
                    break;
                }
            }
        }

        self.input.consume(consumed);
        self.output.produce(produced);

        if !self.input.slice().is_empty() && !self.output.slice().is_empty() {
            io.call_again = true;
        }

        if self.input.slice().is_empty() && self.input.finished() {
            io.finished = true;
        }

        Ok(())
    }
}

#[derive(Block)]
pub struct VaricodeEncoder<
    I: CpuBufferReader<Item = u8> = DefaultCpuReader<u8>,
    O: CpuBufferWriter<Item = u8> = DefaultCpuWriter<u8>,
> {
    #[input]
    input: I,
    #[output]
    output: O,
}

impl VaricodeEncoder<DefaultCpuReader<u8>, DefaultCpuWriter<u8>> {
    pub fn new() -> Self {
        Self {
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for VaricodeEncoder<DefaultCpuReader<u8>, DefaultCpuWriter<u8>> {
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
impl<I, O> Kernel for VaricodeEncoder<I, O>
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
        let i = self.input.slice();
        let o = self.output.slice();

        let mut consumed = 0;
        let mut produced = 0;

        while consumed < i.len() {
            let ascii = i[consumed];
            if let Some(item) = VARICODE_TABLE.iter().find(|it| it.ascii == ascii) {
                let total_bits = item.bitcount + 2; // codeword + "00" delimiter
                if produced + total_bits > o.len() {
                    break; // output buffer full
                }
                consumed += 1;
                // Emit codeword bits from MSB to LSB
                for bit_idx in (0..item.bitcount).rev() {
                    let bit = ((item.code >> bit_idx) & 1) as u8;
                    o[produced] = bit;
                    produced += 1;
                }
                // Emit delimiter "00"
                o[produced] = 0;
                o[produced + 1] = 0;
                produced += 2;
            } else {
                consumed += 1; // skip unknown ascii
            }
        }

        self.input.consume(consumed);
        self.output.produce(produced);

        if !self.input.slice().is_empty() && self.output.slice().len() >= 16 {
            io.call_again = true;
        }

        if self.input.slice().is_empty() && self.input.finished() {
            io.finished = true;
        }

        Ok(())
    }
}
