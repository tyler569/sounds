use num_complex::{Complex, ComplexFloat};
use rustfft::FftPlanner;

use crate::color::{Color, Reset};

#[derive(Debug)]
pub struct Samples(Vec<Complex<f32>>);

impl Samples {
    pub fn new(buf: &[f32], channels: usize) -> Self {
        Self(buf.iter().step_by(channels).map(|&v| Complex::new(v, 0.0)).collect())
    }

    pub fn into_fft(mut self) -> Fft {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(self.0.len());
        fft.process(&mut self.0);
        Fft(self.0)
    }
}

#[derive(Debug)]
pub struct Fft(Vec<Complex<f32>>);

impl Fft {
    pub fn into_samples(mut self) -> Samples {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_inverse(self.0.len());
        fft.process(&mut self.0);
        Samples(self.0)
    }

    pub fn point(&self, index: usize) -> FftPoint {
        let p = self.0[index];
        FftPoint(p.abs(), p.re.atan2(p.im))
    }

    pub fn points(&self) -> impl Iterator<Item = FftPoint> + '_ {
        (0..self.0.len() / 2).map(|i| self.point(i))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FftPoint(f32, f32);

impl FftPoint {
    pub fn amplitude(&self) -> f32 {
        self.0
    }

    pub fn phase(&self) -> f32 {
        self.1
    }

    pub fn phase_01(&self) -> f32 {
        let positive = if self.1 < 0.0 {
            self.1 + std::f32::consts::PI * 2.0
        } else {
            self.1
        };

        positive / (std::f32::consts::PI * 2.0)
    }

    fn amp_char(&self) -> char {
        match self.amplitude() {
            a if a < 0.0 => '?',
            a if a < 1.0 => ' ',
            a if a < 2.0 => '.',
            a if a < 4.0 => ':',
            a if a < 8.0 => '^',
            a if a < 16.0 => '*',
            a if a < 24.0 => '#',
            _ => '?',
        }
    }

    fn color(&self) -> Color {
        Color::from_phase(self.phase())
    }
}

impl std::fmt::Display for FftPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.amplitude() > 1.0 {
            write!(f, "{}{}", self.color(), self.amp_char())?;
        } else {
            write!(f, " ")?;
        }

        Ok(())
    }
}


impl std::fmt::Display for Fft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;

        for point in self.points() {
            write!(f, "{}", point)?;
        }

        write!(f, "{}]", Reset)?;
        Ok(())
    }
}
