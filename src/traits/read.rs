use super::Result;

pub trait SoundRead {
    fn read(&mut self, buffer: &mut [f32]) -> Result<usize>;
}
