use futuresdr::num_complex::Complex32;

pub trait Constellation {
    fn decision_maker(&self, x: Complex32) -> usize;
    fn map_to_points(&self, index: usize, tgt: &mut [Complex32]);
    fn dimensionality(&self) -> usize;
}
