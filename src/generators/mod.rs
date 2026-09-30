pub mod bbs;
pub mod chacha20;
pub mod sha256;

pub trait PseudoRandomGenerator {
    fn fill_bytes(&mut self, output: &mut [u8]);
    fn name(&self) -> &'static str;
}