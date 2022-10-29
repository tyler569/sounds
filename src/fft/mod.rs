mod fftdecoder;
mod fftpoint;
mod fftvisualizer;

pub use fftdecoder::FftDecoder;
pub use fftpoint::FftPoint;
pub use fftvisualizer::FftVisualizer;

pub fn fbucket(sample_rate: f32, points: usize) -> f32 {
    sample_rate / points as f32
}
