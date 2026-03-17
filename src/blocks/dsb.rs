use futuresdr::blocks::Apply;
use futuresdr::num_complex::Complex32;

/// Creates a Double Sideband (DSB) modulator block.
///
/// Converts a real `f32` signal to `Complex32` by setting the imaginary (Q)
/// part to the given `q_value` (default `0.0`).
///
/// This corresponds to `csdr dsb_fc` in the csdr toolchain.
pub fn dsb_fc(
    q_value: f32,
) -> Apply<impl FnMut(&f32) -> Complex32 + Send + 'static, f32, Complex32> {
    let q = q_value;
    Apply::<_, f32, Complex32>::new(move |v: &f32| Complex32::new(*v, q))
}
