use super::Result;

pub trait SoundRead {
    fn read(&mut self, buffer: &mut [f32]) -> Result<usize>;

    fn sample_rate(&self) -> u32;
    fn channels(&self) -> u32;
}
