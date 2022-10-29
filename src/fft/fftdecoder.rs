use std::ops::Range;

use super::FftPoint;
use num_complex::{Complex, ComplexFloat};
use rustfft::FftPlanner;

pub struct FftDecoder {
    complexes: Vec<Complex<f32>>,
}

impl FftDecoder {
    pub fn perform(buffer: &[f32]) -> Self {
        let mut complexes: Vec<Complex<f32>> =
            buffer.iter().map(|&b| Complex::new(b, 0.0)).collect();

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(complexes.len());
        fft.process(&mut complexes);

        Self { complexes }
    }

    pub fn positive_len(&self) -> usize {
        self.complexes.len() / 2
    }

    pub fn point(&self, index: usize) -> FftPoint {
        FftPoint::new(self.complexes[index])
    }

    pub fn peak(&self) -> FftPoint {
        let peak_index = self
            .complexes
            .iter()
            .take(self.positive_len())
            .enumerate()
            .max_by_key(|(i, &v)| (v.abs() * 1000000.0) as u64)
            .unwrap()
            .0;

        self.point(peak_index)
    }

    pub fn print_channel_range(&self, f: Range<usize>) {
        eprint!("[");
        f.for_each(|v| eprint!("{}", self.point(v)));
        eprint!("]");
    }
}
