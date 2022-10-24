use std::ops::Range;

use super::FftPoint;
use num_complex::{Complex, ComplexFloat};
use rustfft::FftPlanner;

pub struct FftDecoder {
    complexes: Vec<Complex<f32>>,
    sample_rate: f32,
}

impl FftDecoder {
    pub fn perform(sample_rate: f32, buffer: &[f32]) -> Self {
        let mut complexes: Vec<Complex<f32>> =
            buffer.iter().map(|&b| Complex::new(b, 0.0)).collect();

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(complexes.len());
        fft.process(&mut complexes);

        Self {
            complexes,
            sample_rate,
        }
    }

    pub fn fbucket(&self) -> f32 {
        self.sample_rate / self.complexes.len() as f32
    }

    pub fn positive_len(&self) -> usize {
        self.complexes.len() / 2
    }

    pub fn frequency(&self, index: usize) -> f32 {
        if index < self.positive_len() {
            self.fbucket() * index as f32
        } else {
            todo!()
        }
    }

    pub fn frequencies(&self) -> impl Iterator<Item = f32> + '_ {
        (0..self.positive_len())
            .into_iter()
            .map(|i| self.frequency(i))
    }

    pub fn point(&self, index: usize) -> FftPoint {
        FftPoint::new(self.frequency(index), self.complexes[index])
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

    pub fn print_frequency_range(&self, f: Range<usize>) {
        let bottom = (f.start as f32 / self.fbucket()) as usize;
        let len = ((f.end - f.start) as f32 / self.fbucket()) as usize;
        let top = bottom + len;

        eprint!("[");
        (bottom..top).for_each(|v| eprint!("{}", self.point(v)));
        eprint!("]");
    }
}
