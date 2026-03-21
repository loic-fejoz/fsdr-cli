use anyhow::Result;
use fsdr_cli_macros::fsdr_instantiate;
use futuresdr::blocks::{Apply, ApplyNM, FileSource, FirBuilder, Sink};
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::{BlockId, Flowgraph, Kernel, KernelInterface};
use quote::{quote, ToTokens};
use std::ops::Deref;

/// A wrapper for Complex32 that implements ToTokens for code generation.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct CodegenComplex(pub Complex32);

impl From<Complex32> for CodegenComplex {
    fn from(c: Complex32) -> Self {
        Self(c)
    }
}

impl Deref for CodegenComplex {
    type Target = Complex32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ToTokens for CodegenComplex {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let re = self.0.re;
        let im = self.0.im;
        tokens.extend(quote!(futuresdr::num_complex::Complex32::new(#re, #im)));
    }
}

/// A wrapper for Vec<f32> that implements ToTokens for code generation.
#[derive(Clone, Debug)]
pub struct CodegenTaps(pub Vec<f32>);

impl CodegenTaps {
    pub fn to_vec(&self) -> Vec<f32> {
        self.0.clone()
    }
}

impl From<Vec<f32>> for CodegenTaps {
    fn from(v: Vec<f32>) -> Self {
        Self(v)
    }
}

impl Deref for CodegenTaps {
    type Target = Vec<f32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ToTokens for CodegenTaps {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let taps = &self.0;
        tokens.extend(quote!(vec![#(#taps),*]));
    }
}

/// A wrapper for Vec<u8> that implements ToTokens for code generation.
#[derive(Clone, Debug)]
pub struct CodegenPattern(pub Vec<u8>);

impl CodegenPattern {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.clone()
    }
}

impl From<Vec<u8>> for CodegenPattern {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl Deref for CodegenPattern {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ToTokens for CodegenPattern {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let pattern = &self.0;
        tokens.extend(quote!(vec![#(#pattern),*]));
    }
}

/// The Abstract Backend Trait for FutureSDR flowgraph construction.
pub trait FsdrBackend {
    type BlockRef: Clone + 'static;

    fn add_multiply_const_f32(&mut self, constant: f32) -> Result<Self::BlockRef>;
    fn add_fir_filter_ccc(
        &mut self,
        decimation: usize,
        taps: CodegenTaps,
    ) -> Result<Self::BlockRef>;

    fn add_file_source_f32(&mut self, filename: String, repeat: bool) -> Result<Self::BlockRef>;
    fn add_file_source_u8(&mut self, filename: String, repeat: bool) -> Result<Self::BlockRef>;
    fn add_file_source_c32(&mut self, filename: String, repeat: bool) -> Result<Self::BlockRef>;

    fn add_dump_f32(&mut self) -> Result<Self::BlockRef>;
    fn add_dump_u8(&mut self) -> Result<Self::BlockRef>;
    fn add_dump_c32(&mut self) -> Result<Self::BlockRef>;

    // New blocks for the complex command
    fn add_uchar_to_float(&mut self) -> Result<Self::BlockRef>;
    fn add_f32_to_s16(&mut self) -> Result<Self::BlockRef>;
    #[allow(dead_code)]
    fn add_float_to_complex(&mut self) -> Result<Self::BlockRef>;
    fn add_freq_shift_cc(&mut self, freq: f32, sample_rate: f32) -> Result<Self::BlockRef>;
    fn add_quadrature_demod_cf(&mut self, gain: f32) -> Result<Self::BlockRef>;
    fn add_rational_resampler_ff(
        &mut self,
        interp: usize,
        decim: usize,
        taps: CodegenTaps,
    ) -> Result<Self::BlockRef>;
    fn add_dsb_fc(&mut self, q_value: f32) -> Result<Self::BlockRef>;
    fn add_timing_recovery_cc(&mut self, decim: usize) -> Result<Self::BlockRef>;
    fn add_complex_to_real(&mut self) -> Result<Self::BlockRef>;
    fn add_binary_slicer_fb(&mut self) -> Result<Self::BlockRef>;
    fn add_pattern_search_u8(
        &mut self,
        values_after: usize,
        pattern: CodegenPattern,
    ) -> Result<Self::BlockRef>;
    fn add_pack_bits_8to1(&mut self) -> Result<Self::BlockRef>;
    fn add_fixedlen_to_pdu(&mut self, packet_len: usize) -> Result<Self::BlockRef>;
    fn add_kiss_file_sink(&mut self, filename: String) -> Result<Self::BlockRef>;

    fn connect(
        &mut self,
        src: &Self::BlockRef,
        src_port: &str,
        dst: &Self::BlockRef,
        dst_port: &str,
    ) -> Result<()>;

    /// Temporary helper for Phase 1 to allow incremental migration.
    fn add_block_runtime<K: Kernel + KernelInterface + 'static>(
        &mut self,
        block: K,
    ) -> Result<Self::BlockRef>;

    /// Temporary helper for Phase 1 to allow incremental migration.
    #[allow(dead_code)]
    fn as_runtime_mut(&mut self) -> Option<&mut Flowgraph> {
        None
    }
}

// --- Implementation Functions (One Source of Truth) ---

#[fsdr_instantiate]
fn build_multiply_const_f32(
    constant: f32,
) -> Apply<impl FnMut(&f32) -> f32 + Send + 'static, f32, f32> {
    Apply::<_, f32, f32>::new(move |v: &f32| v * constant)
}

#[fsdr_instantiate]
fn build_fir_filter_ccc(decimation: usize, taps: CodegenTaps) -> impl Kernel + KernelInterface {
    FirBuilder::resampling_with_taps::<Complex32, Complex32, Vec<f32>>(1, decimation, taps.to_vec())
}

#[fsdr_instantiate]
fn build_file_source_f32(filename: String, repeat: bool) -> FileSource<f32> {
    FileSource::<f32>::new(filename, repeat)
}

#[fsdr_instantiate]
fn build_file_source_u8(filename: String, repeat: bool) -> FileSource<u8> {
    FileSource::<u8>::new(filename, repeat)
}

#[fsdr_instantiate]
fn build_file_source_c32(filename: String, repeat: bool) -> FileSource<Complex32> {
    FileSource::<Complex32>::new(filename, repeat)
}

#[fsdr_instantiate]
fn build_dump_f32() -> Sink<impl FnMut(&f32) + Send + 'static, f32> {
    Sink::<_, f32>::new(|x: &f32| print!("{:e} ", *x))
}

#[fsdr_instantiate]
fn build_dump_u8() -> Sink<impl FnMut(&u8) + Send + 'static, u8> {
    Sink::<_, u8>::new(|x: &u8| print!("{:02x} ", *x))
}

#[fsdr_instantiate]
fn build_dump_c32() -> Sink<impl FnMut(&Complex32) + Send + 'static, Complex32> {
    Sink::<_, Complex32>::new(|x: &Complex32| print!("{:e}+{:e}i ", x.re, x.im))
}

#[fsdr_instantiate]
fn build_uchar_to_float() -> Apply<impl FnMut(&u8) -> f32 + Send + 'static, u8, f32> {
    Apply::<_, u8, f32>::new(|x| (*x as f32 / 128.0) - 1.0)
}

#[fsdr_instantiate]
fn build_f32_to_s16() -> Apply<impl FnMut(&f32) -> i16 + Send + 'static, f32, i16> {
    Apply::<_, f32, i16>::new(|x| (*x * 32767.0) as i16)
}

#[fsdr_instantiate]
#[allow(dead_code)]
fn build_float_to_complex() -> Apply<impl FnMut(&f32) -> Complex32 + Send + 'static, f32, Complex32>
{
    Apply::<_, f32, Complex32>::new(|x| Complex32::new(*x, 0.0))
}

#[fsdr_instantiate]
fn build_freq_shift_cc(
    freq: f32,
    sample_rate: f32,
) -> fsdr_blocks::math::FrequencyShifter<Complex32> {
    fsdr_blocks::math::FrequencyShifter::<Complex32>::new(freq, sample_rate)
}

#[fsdr_instantiate]
fn build_quadrature_demod_cf(
    gain: f32,
) -> Apply<impl FnMut(&Complex32) -> f32 + Send + 'static, Complex32, f32> {
    // Wrapped in a block to be used as expression in codegen
    {
        let mut last = Complex32::new(0.0, 0.0);
        Apply::<_, Complex32, f32>::new(move |v: &Complex32| -> f32 {
            let v = *v;
            let res = (v * last.conj()).arg() * gain;
            last = v;
            res
        })
    }
}

#[fsdr_instantiate]
fn build_rational_resampler_ff(
    interp: usize,
    decim: usize,
    taps: CodegenTaps,
) -> impl Kernel + KernelInterface {
    FirBuilder::resampling_with_taps::<f32, f32, Vec<f32>>(interp, decim, taps.to_vec())
}

#[fsdr_instantiate]
fn build_dsb_fc(
    q_value: f32,
) -> Apply<impl FnMut(&f32) -> Complex32 + Send + 'static, f32, Complex32> {
    // Manual implementation of dsb_fc logic to avoid path issues in codegen
    Apply::<_, f32, Complex32>::new(move |v: &f32| -> Complex32 {
        Complex32::new(*v, *v * q_value)
    })
}

#[fsdr_instantiate]
fn build_timing_recovery_cc(
    decim: usize,
) -> crate::blocks::synchronizers::timing_recovery::TimingRecovery<Complex32> {
    crate::blocks::synchronizers::timing_recovery::TimingRecovery::new(
        crate::blocks::synchronizers::timing_recovery::TimingAlgorithm::GARDNER,
        decim,
        0.1,
        0.1,
    )
}

#[fsdr_instantiate]
fn build_complex_to_real() -> Apply<impl FnMut(&Complex32) -> f32 + Send + 'static, Complex32, f32>
{
    Apply::<_, Complex32, f32>::new(|x| x.re)
}

#[fsdr_instantiate]
fn build_binary_slicer_fb() -> Apply<impl FnMut(&f32) -> u8 + Send + 'static, f32, u8> {
    Apply::<_, f32, u8>::new(|x| if *x > 0.0 { 1 } else { 0 })
}

#[fsdr_instantiate]
fn build_pattern_search_u8(
    values_after: usize,
    pattern: CodegenPattern,
) -> crate::blocks::pattern_search::PatternSearch<u8> {
    crate::blocks::pattern_search::PatternSearch::<u8>::new(values_after, pattern.to_vec())
}

#[fsdr_instantiate]
fn build_pack_bits_8to1() -> ApplyNM<impl FnMut(&[u8], &mut [u8]) + Send + 'static, u8, u8, 8, 1> {
    ApplyNM::<_, u8, u8, 8, 1>::new(move |v: &[u8], res: &mut [u8]| {
        let mut val = 0u8;
        for item in v.iter().take(8) {
            val = (val << 1) | (item & 1);
        }
        res[0] = val;
    })
}

#[fsdr_instantiate]
fn build_fixedlen_to_pdu(
    packet_len: usize,
) -> crate::blocks::fixedlen_to_pdu::FixedlenToPdu<futuresdr::prelude::DefaultCpuReader<u8>> {
    crate::blocks::fixedlen_to_pdu::FixedlenToPdu::<futuresdr::prelude::DefaultCpuReader<u8>>::new(
        packet_len,
    )
}

#[fsdr_instantiate]
fn build_kiss_file_sink(filename: String) -> crate::blocks::kiss_file_sink::KissFileSink {
    crate::blocks::kiss_file_sink::KissFileSink::new(&filename).unwrap()
}

// --- Runtime Backend ---

pub struct RuntimeBackend<'a> {
    pub fg: &'a mut Flowgraph,
}

impl<'a> FsdrBackend for RuntimeBackend<'a> {
    type BlockRef = BlockId;

    fn add_multiply_const_f32(&mut self, constant: f32) -> Result<BlockId> {
        let blk = build_multiply_const_f32(constant);
        Ok(self.fg.add_block(blk).into())
    }

    fn add_fir_filter_ccc(&mut self, decimation: usize, taps: CodegenTaps) -> Result<BlockId> {
        let blk = build_fir_filter_ccc(decimation, taps);
        Ok(self.fg.add_block(blk).into())
    }

    fn add_file_source_f32(&mut self, filename: String, repeat: bool) -> Result<BlockId> {
        let blk = build_file_source_f32(filename, repeat);
        Ok(self.fg.add_block(blk).into())
    }

    fn add_file_source_u8(&mut self, filename: String, repeat: bool) -> Result<BlockId> {
        let blk = build_file_source_u8(filename, repeat);
        Ok(self.fg.add_block(blk).into())
    }

    fn add_file_source_c32(&mut self, filename: String, repeat: bool) -> Result<BlockId> {
        let blk = build_file_source_c32(filename, repeat);
        Ok(self.fg.add_block(blk).into())
    }

    fn add_dump_f32(&mut self) -> Result<BlockId> {
        let blk = build_dump_f32();
        Ok(self.fg.add_block(blk).into())
    }

    fn add_dump_u8(&mut self) -> Result<BlockId> {
        let blk = build_dump_u8();
        Ok(self.fg.add_block(blk).into())
    }

    fn add_dump_c32(&mut self) -> Result<BlockId> {
        let blk = build_dump_c32();
        Ok(self.fg.add_block(blk).into())
    }

    fn add_uchar_to_float(&mut self) -> Result<BlockId> {
        let blk = build_uchar_to_float();
        Ok(self.fg.add_block(blk).into())
    }

    fn add_f32_to_s16(&mut self) -> Result<BlockId> {
        let blk = build_f32_to_s16();
        Ok(self.fg.add_block(blk).into())
    }

    fn add_float_to_complex(&mut self) -> Result<BlockId> {
        let blk = build_float_to_complex();
        Ok(self.fg.add_block(blk).into())
    }

    fn add_freq_shift_cc(&mut self, freq: f32, sample_rate: f32) -> Result<BlockId> {
        let blk = build_freq_shift_cc(freq, sample_rate);
        Ok(self.fg.add_block(blk).into())
    }

    fn add_quadrature_demod_cf(&mut self, gain: f32) -> Result<BlockId> {
        let blk = build_quadrature_demod_cf(gain);
        Ok(self.fg.add_block(blk).into())
    }

    fn add_rational_resampler_ff(
        &mut self,
        interp: usize,
        decim: usize,
        taps: CodegenTaps,
    ) -> Result<BlockId> {
        let blk = build_rational_resampler_ff(interp, decim, taps);
        Ok(self.fg.add_block(blk).into())
    }

    fn add_dsb_fc(&mut self, q_value: f32) -> Result<BlockId> {
        let blk = build_dsb_fc(q_value);
        Ok(self.fg.add_block(blk).into())
    }

    fn add_timing_recovery_cc(&mut self, decim: usize) -> Result<BlockId> {
        let blk = build_timing_recovery_cc(decim);
        Ok(self.fg.add_block(blk).into())
    }

    fn add_complex_to_real(&mut self) -> Result<BlockId> {
        let blk = build_complex_to_real();
        Ok(self.fg.add_block(blk).into())
    }

    fn add_binary_slicer_fb(&mut self) -> Result<BlockId> {
        let blk = build_binary_slicer_fb();
        Ok(self.fg.add_block(blk).into())
    }

    fn add_pattern_search_u8(
        &mut self,
        values_after: usize,
        pattern: CodegenPattern,
    ) -> Result<BlockId> {
        let blk = build_pattern_search_u8(values_after, pattern);
        Ok(self.fg.add_block(blk).into())
    }

    fn add_pack_bits_8to1(&mut self) -> Result<BlockId> {
        let blk = build_pack_bits_8to1();
        Ok(self.fg.add_block(blk).into())
    }

    fn add_fixedlen_to_pdu(&mut self, packet_len: usize) -> Result<BlockId> {
        let blk = build_fixedlen_to_pdu(packet_len);
        Ok(self.fg.add_block(blk).into())
    }

    fn add_kiss_file_sink(&mut self, filename: String) -> Result<BlockId> {
        let blk = build_kiss_file_sink(filename);
        Ok(self.fg.add_block(blk).into())
    }

    fn connect(
        &mut self,
        src: &BlockId,
        src_port: &str,
        dst: &BlockId,
        dst_port: &str,
    ) -> Result<()> {
        if let Err(e) = self.fg.connect_dyn(*src, src_port, *dst, dst_port) {
            // If it's not a stream port, try message port
            self.fg
                .connect_message(*src, src_port, *dst, dst_port)
                .map_err(|_| e)?;
        }
        Ok(())
    }

    fn add_block_runtime<K: Kernel + KernelInterface + 'static>(
        &mut self,
        block: K,
    ) -> Result<BlockId> {
        Ok(self.fg.add_block(block).into())
    }

    fn as_runtime_mut(&mut self) -> Option<&mut Flowgraph> {
        Some(self.fg)
    }
}

// --- Codegen Backend ---

pub struct CodegenBackend {
    pub tokens: proc_macro2::TokenStream,
    block_count: usize,
}

impl CodegenBackend {
    pub fn new() -> Self {
        Self {
            tokens: quote! {
                let mut fg = Flowgraph::new();
            },
            block_count: 0,
        }
    }

    fn next_block_id(&mut self) -> String {
        let id = format!("blk_{}", self.block_count);
        self.block_count += 1;
        id
    }
}

impl Default for CodegenBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl FsdrBackend for CodegenBackend {
    type BlockRef = String;

    fn add_multiply_const_f32(&mut self, constant: f32) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_multiply_const_f32_codegen(constant);
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_fir_filter_ccc(&mut self, decimation: usize, taps: CodegenTaps) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_fir_filter_ccc_codegen(decimation, taps);
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_file_source_f32(&mut self, filename: String, repeat: bool) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_file_source_f32_codegen(filename, repeat);
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_file_source_u8(&mut self, filename: String, repeat: bool) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_file_source_u8_codegen(filename, repeat);
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_file_source_c32(&mut self, filename: String, repeat: bool) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_file_source_c32_codegen(filename, repeat);
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_dump_f32(&mut self) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_dump_f32_codegen();
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_dump_u8(&mut self) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_dump_u8_codegen();
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_dump_c32(&mut self) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_dump_c32_codegen();
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_uchar_to_float(&mut self) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_uchar_to_float_codegen();
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_f32_to_s16(&mut self) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_f32_to_s16_codegen();
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_float_to_complex(&mut self) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_float_to_complex_codegen();
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_freq_shift_cc(&mut self, freq: f32, sample_rate: f32) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_freq_shift_cc_codegen(freq, sample_rate);
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_quadrature_demod_cf(&mut self, gain: f32) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_quadrature_demod_cf_codegen(gain);
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_rational_resampler_ff(
        &mut self,
        interp: usize,
        decim: usize,
        taps: CodegenTaps,
    ) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_rational_resampler_ff_codegen(interp, decim, taps);
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_dsb_fc(&mut self, q_value: f32) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_dsb_fc_codegen(q_value);
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_timing_recovery_cc(&mut self, decim: usize) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_timing_recovery_cc_codegen(decim);
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_complex_to_real(&mut self) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_complex_to_real_codegen();
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_binary_slicer_fb(&mut self) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_binary_slicer_fb_codegen();
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_pattern_search_u8(
        &mut self,
        values_after: usize,
        pattern: CodegenPattern,
    ) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_pattern_search_u8_codegen(values_after, pattern);
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_pack_bits_8to1(&mut self) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_pack_bits_8to1_codegen();
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_fixedlen_to_pdu(&mut self, packet_len: usize) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_fixedlen_to_pdu_codegen(packet_len);
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn add_kiss_file_sink(&mut self, filename: String) -> Result<String> {
        let id = self.next_block_id();
        let id_ident = quote::format_ident!("{}", id);
        let block_tokens = build_kiss_file_sink_codegen(filename);
        self.tokens.extend(quote! {
            let #id_ident = fg.add_block(#block_tokens);
        });
        Ok(id)
    }

    fn connect(
        &mut self,
        src: &String,
        src_port: &str,
        dst: &String,
        dst_port: &str,
    ) -> Result<()> {
        let src_ident = quote::format_ident!("{}", src);
        let dst_ident = quote::format_ident!("{}", dst);
        self.tokens.extend(quote! {
            if let Err(_) = fg.connect_dyn(&#src_ident, #src_port, &#dst_ident, #dst_port) {
                fg.connect_message(&#src_ident, #src_port, &#dst_ident, #dst_port).unwrap();
            }
        });
        Ok(())
    }

    fn add_block_runtime<K: Kernel + KernelInterface + 'static>(
        &mut self,
        _block: K,
    ) -> Result<String> {
        Ok(self.next_block_id())
    }

    fn as_runtime_mut(&mut self) -> Option<&mut Flowgraph> {
        None
    }
}
