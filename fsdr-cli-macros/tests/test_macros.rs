use fsdr_cli_macros::{eval_math, fsdr_instantiate};
use futuresdr::num_complex::Complex32;
use quote::{quote, ToTokens, TokenStreamExt};
use proc_macro2::TokenStream;

#[test]
fn test_eval_math() {
    let result = eval_math!(2 + 3 * 4);
    assert_eq!(result, 14);
}

// 1. Handling simple types (f32) works natively because f32: ToTokens
#[fsdr_instantiate]
fn multiply_by_two(val: f32) -> f32 {
    val * 2.0
}

#[test]
fn test_simple_interpolation() {
    let input = 21.0;
    let codegen_tokens = multiply_by_two_codegen(input);
    let codegen_string = codegen_tokens.to_string();
    println!("Simple generated tokens: {}", codegen_string);
    assert!(codegen_string.contains("21"));
}

// 2. Handling Complex types: We need a wrapper that implements ToTokens
#[derive(Clone, Copy)]
pub struct CodegenComplex(pub Complex32);

impl ToTokens for CodegenComplex {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let re = self.0.re;
        let im = self.0.im;
        tokens.extend(quote!(futuresdr::num_complex::Complex32::new(#re, #im)));
    }
}

// 3. Handling Vec/Taps: We need a wrapper that implements ToTokens
#[derive(Clone)]
pub struct CodegenTaps(pub Vec<f32>);

impl ToTokens for CodegenTaps {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let values = &self.0;
        tokens.extend(quote!(vec![ #(#values),* ]));
    }
}

// Use a NON-MACRO body to confirm interpolation works for complex types
#[fsdr_instantiate]
fn get_fir_metadata(taps: CodegenTaps, center: CodegenComplex) -> (usize, f32) {
    (taps.0.len(), center.0.re)
}

#[test]
fn test_complex_type_handling() {
    let taps = CodegenTaps(vec![0.1, 0.2, 0.3]);
    let center = CodegenComplex(Complex32::new(1.0, -1.0));

    let codegen_tokens = get_fir_metadata_codegen(taps, center);
    let codegen_string = codegen_tokens.to_string();
    println!("Complex generated tokens: {}", codegen_string);
    
    // Now it should be interpolated!
    assert!(codegen_string.contains("vec ! [0.1f32 , 0.2f32 , 0.3f32]"));
    assert!(codegen_string.contains("Complex32 :: new (1.0f32 , - 1.0f32)"));
    
    // And NO literal "taps" identifier remaining in the tuple
    assert!(!codegen_string.contains("( taps"));
}
