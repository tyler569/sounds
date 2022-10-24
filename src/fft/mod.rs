mod fftdecoder;
mod fftpoint;

pub use fftdecoder::FftDecoder;
pub use fftpoint::FftPoint;

pub fn fbucket(sample_rate: f32, points: usize) -> f32 {
    sample_rate / points as f32
}
