use super::Result;

pub trait SoundWrite {
    fn write(&mut self, buffer: &[f32]) -> Result<usize>;
}