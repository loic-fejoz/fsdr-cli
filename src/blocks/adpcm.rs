use anyhow::Result;
use futuresdr::runtime::dev::prelude::*;
use std::cmp;

pub const STEP_SIZE_TABLE: [i32; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66,
    73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449,
    494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272,
    2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630, 9493,
    10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794, 32767,
];

pub const INDEX_ADJUST_TABLE: [i32; 16] = [
    -1, -1, -1, -1, // +0 - +3, decrease the step size
    2, 4, 6, 8, // +4 - +7, increase the step size
    -1, -1, -1, -1, // -0 - -3, decrease the step size
    2, 4, 6, 8, // -4 - -7, increase the step size
];

pub const COMPRESS_FFT_PAD_N: usize = 10;

#[derive(Clone, Debug, Default)]
pub struct AdpcmCodec {
    pub index: i32,
    pub previous_value: i32,
}

impl AdpcmCodec {
    pub fn new() -> Self {
        Self {
            index: 0,
            previous_value: 0,
        }
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.previous_value = 0;
    }

    pub fn encode_sample(&mut self, sample: i16) -> u8 {
        let mut diff = sample as i32 - self.previous_value;
        let mut step = STEP_SIZE_TABLE[self.index as usize];
        let mut delta_code = 0u8;

        if diff < 0 {
            delta_code = 8;
            diff = -diff;
        }

        if diff >= step {
            delta_code |= 4;
            diff -= step;
        }
        step >>= 1;
        if diff >= step {
            delta_code |= 2;
            diff -= step;
        }
        step >>= 1;
        if diff >= step {
            delta_code |= 1;
        }

        self.decode_sample(delta_code);
        delta_code
    }

    pub fn decode_sample(&mut self, delta_code: u8) -> i16 {
        let step = STEP_SIZE_TABLE[self.index as usize];
        let mut difference = step >> 3;

        if (delta_code & 1) != 0 {
            difference += step >> 2;
        }
        if (delta_code & 2) != 0 {
            difference += step >> 1;
        }
        if (delta_code & 4) != 0 {
            difference += step;
        }
        if (delta_code & 8) != 0 {
            difference = -difference;
        }

        self.previous_value += difference;
        self.previous_value = self.previous_value.clamp(-32768, 32767);

        self.index += INDEX_ADJUST_TABLE[(delta_code & 0x0F) as usize];
        self.index = self.index.clamp(0, 88);

        self.previous_value as i16
    }
}

#[derive(Block)]
pub struct AdpcmEncoderI16U8<
    I: CpuBufferReader<Item = i16> = DefaultCpuReader<i16>,
    O: CpuBufferWriter<Item = u8> = DefaultCpuWriter<u8>,
> {
    codec: AdpcmCodec,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl AdpcmEncoderI16U8<DefaultCpuReader<i16>, DefaultCpuWriter<u8>> {
    pub fn new() -> Self {
        Self {
            codec: AdpcmCodec::new(),
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for AdpcmEncoderI16U8<DefaultCpuReader<i16>, DefaultCpuWriter<u8>> {
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
impl<I, O> Kernel for AdpcmEncoderI16U8<I, O>
where
    I: CpuBufferReader<Item = i16>,
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

        let pairs_in = i.len() / 2;
        let n = cmp::min(pairs_in, o.len());

        if n > 0 {
            for k in 0..n {
                let low = self.codec.encode_sample(i[2 * k]);
                let high = self.codec.encode_sample(i[2 * k + 1]);
                o[k] = (low & 0x0F) | ((high & 0x0F) << 4);
            }
            self.input.consume(n * 2);
            self.output.produce(n);
        }

        if self.input.slice().len() >= 2 && !self.output.slice().is_empty() {
            io.call_again = true;
        }

        if self.input.slice().len() < 2 && self.input.finished() {
            io.finished = true;
        }

        Ok(())
    }
}

#[derive(Block)]
pub struct AdpcmDecoderU8I16<
    I: CpuBufferReader<Item = u8> = DefaultCpuReader<u8>,
    O: CpuBufferWriter<Item = i16> = DefaultCpuWriter<i16>,
> {
    codec: AdpcmCodec,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl AdpcmDecoderU8I16<DefaultCpuReader<u8>, DefaultCpuWriter<i16>> {
    pub fn new() -> Self {
        Self {
            codec: AdpcmCodec::new(),
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

impl Default for AdpcmDecoderU8I16<DefaultCpuReader<u8>, DefaultCpuWriter<i16>> {
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
impl<I, O> Kernel for AdpcmDecoderU8I16<I, O>
where
    I: CpuBufferReader<Item = u8>,
    O: CpuBufferWriter<Item = i16>,
{
    async fn work(
        &mut self,
        io: &mut WorkIo,
        _mio: &mut MessageOutputs,
        _meta: &BlockMeta,
    ) -> Result<()> {
        let i = self.input.slice();
        let o = self.output.slice();

        let max_in_by_out = o.len() / 2;
        let n = cmp::min(i.len(), max_in_by_out);

        if n > 0 {
            for k in 0..n {
                let byte = i[k];
                let s1 = self.codec.decode_sample(byte & 0x0F);
                let s2 = self.codec.decode_sample(byte >> 4);
                o[2 * k] = s1;
                o[2 * k + 1] = s2;
            }
            self.input.consume(n);
            self.output.produce(n * 2);
        }

        if !self.input.slice().is_empty() && self.output.slice().len() >= 2 {
            io.call_again = true;
        }

        if self.input.slice().is_empty() && self.input.finished() {
            io.finished = true;
        }

        Ok(())
    }
}

#[derive(Block)]
pub struct CompressFftAdpcmFU8<
    I: CpuBufferReader<Item = f32> = DefaultCpuReader<f32>,
    O: CpuBufferWriter<Item = u8> = DefaultCpuWriter<u8>,
> {
    fft_size: usize,
    codec: AdpcmCodec,
    #[input]
    input: I,
    #[output]
    output: O,
}

impl CompressFftAdpcmFU8<DefaultCpuReader<f32>, DefaultCpuWriter<u8>> {
    pub fn new(fft_size: usize) -> Self {
        assert!(fft_size > 0, "fft_size must be > 0");
        Self {
            fft_size,
            codec: AdpcmCodec::new(),
            input: DefaultCpuReader::default(),
            output: DefaultCpuWriter::default(),
        }
    }
}

#[doc(hidden)]
impl<I, O> Kernel for CompressFftAdpcmFU8<I, O>
where
    I: CpuBufferReader<Item = f32>,
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

        let fft_size = self.fft_size;
        let pad_bytes = COMPRESS_FFT_PAD_N / 2;
        let data_bytes = fft_size / 2;
        let out_bytes_per_fft = pad_bytes + data_bytes;

        let num_ffts = cmp::min(i.len() / fft_size, o.len() / out_bytes_per_fft);

        if num_ffts > 0 {
            for f in 0..num_ffts {
                self.codec.reset();
                let in_offset = f * fft_size;
                let out_offset = f * out_bytes_per_fft;

                let first_val = (i[in_offset] * 100.0) as i16;

                // Pad bytes
                for p in 0..pad_bytes {
                    let low = self.codec.encode_sample(first_val);
                    let high = self.codec.encode_sample(first_val);
                    o[out_offset + p] = (low & 0x0F) | ((high & 0x0F) << 4);
                }

                // Data bytes
                for d in 0..data_bytes {
                    let s1 = (i[in_offset + 2 * d] * 100.0) as i16;
                    let s2 = (i[in_offset + 2 * d + 1] * 100.0) as i16;
                    let low = self.codec.encode_sample(s1);
                    let high = self.codec.encode_sample(s2);
                    o[out_offset + pad_bytes + d] = (low & 0x0F) | ((high & 0x0F) << 4);
                }
            }

            self.input.consume(num_ffts * fft_size);
            self.output.produce(num_ffts * out_bytes_per_fft);
        }

        if self.input.slice().len() >= fft_size && self.output.slice().len() >= out_bytes_per_fft {
            io.call_again = true;
        }

        if self.input.slice().len() < fft_size && self.input.finished() {
            io.finished = true;
        }

        Ok(())
    }
}
