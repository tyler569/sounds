use std::f32::consts::PI;

use crossbeam::channel::RecvTimeoutError;
use num_complex::{Complex, ComplexFloat};
use rustfft::FftPlanner;

use crate::color::{Color, Reset};

#[derive(Clone, Debug)]
pub struct Samples(Vec<Complex<f32>>);

impl Samples {
    pub fn new(buf: &[f32], channels: usize) -> Self {
        Self(
            buf.iter()
                .step_by(channels)
                .map(|&v| Complex::new(v, 0.0))
                .collect(),
        )
    }

    pub fn into_fft(mut self) -> Fft {
        let mut planner = FftPlanner::new();
        let len = self.0.len();
        let fft = planner.plan_fft_forward(len);
        fft.process(&mut self.0);
        self.0.iter_mut().for_each(|v| *v *= 1. / len as f32);
        Fft(self.0)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn window_null(&mut self) {
        self.0.iter_mut().for_each(|v| *v *= 0.);
    }

    pub fn window_sin(&mut self) {
        let n = self.len() as f32;

        self.0.iter_mut().enumerate()
            .for_each(|(i, v)| *v *= ((PI * i as f32) / n).sin());
    }

    pub fn window_plank(&mut self) {
        fn plank(i: usize, n: usize) -> f32 {
            let epsilon = n / 10;
            let fepsilon = epsilon as f32;
            let fi = i as f32;

            if i == 0 {
                0.
            } else if i < epsilon {
                1. / (1. + (fepsilon / fi - fepsilon / (fepsilon - fi)).exp())
            } else if i < n / 2 {
                1.
            } else {
                plank(n - i, i)
            }
        }

        let n = self.len();

        self.0.iter_mut().enumerate()
            .map(|(i, v)| *v *= plank(i, n));
    }

    pub fn volume(&mut self, volume: f32) {
        self.0.iter_mut().for_each(|v| *v *= volume);
    }

    pub fn is_real(&self) -> bool {
        const SIGNAL_EPSILON: f32 = 0.000001;
        self.0.iter().all(|v| v.im.abs() < SIGNAL_EPSILON)
    }

    pub fn output(&self, buf: &mut [f32], channels: usize) {
        let mut repeated = self.0.iter().flat_map(|v| std::iter::repeat(v).take(channels));
        buf.iter_mut().zip(repeated).for_each(|(b, v)| *b = v.re);
    }

    pub fn iter(&self) -> impl Iterator<Item = f32> + '_ {
        self.0.iter().map(|v| v.re)
    }

    pub fn offset(&mut self, next: &Samples, offset: usize) {
        self.0.rotate_left(offset);
        self.0[offset..].copy_from_slice(&next.0[..offset]);
    }
}

fn sample_char(sample: f32) -> char {
    match sample.abs() {
        a if a > 0.3 => '#',
        a if a > 0.2 => '*',
        a if a > 0.1 => '^',
        a if a > 0.05 => ':',
        a if a > 0.01 => '.',
        _ => ' ',
    }
}

impl std::fmt::Display for Samples {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        let positive = Color::rgb(250, 100, 100);
        let negative = Color::rgb(50, 200, 50);
        for sample in &self.0 {
            if sample.im.abs() < 0.1 {
                if sample.re > 0. {
                    write!(f, "{}{}", positive, sample_char(sample.re))?;
                } else {
                    write!(f, "{}{}", negative, sample_char(sample.re))?;
                }
            } else {
                // large imaginary component
                write!(f, "{}?", Color::rgb(50, 50, 200))?;
            }
        }
        write!(f, "{}}}", Reset)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Fft(Vec<Complex<f32>>);

impl Fft {
    pub fn new(size: usize) -> Self {
        Self(vec![Complex::new(0., 0.); size])
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn positive_len(&self) -> usize {
        self.len() / 2
    }

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

    fn conj_index(&self, index: usize) -> usize {
        self.len() - index
    }

    pub fn set_point(&mut self, index: usize, point: FftPoint) {
        assert!(index < self.positive_len());
        let num = point.as_complex();
        let conj_index = self.conj_index(index);

        self.0[index] = num;
        self.0[conj_index] = num.conj();
    }
}

#[derive(Copy, Clone)]
pub struct FftPoint(f32, f32);

impl FftPoint {
    pub fn new(amplitude: f32, phase: f32) -> Self {
        Self(amplitude, phase)
    }

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
            a if a > 1. => '!',
            a if a > 0.25 => '#',
            a if a > 0.125 => '%',
            a if a > 0.0625 => '*',
            a if a > 0.0312 => '^',
            a if a > 0.0151 => ':',
            a if a > 0.0075 => '.',
            a if a >= 0. => ' ',
            _ => '?',
        }
    }

    fn color(&self) -> Color {
        Color::from_phase(self.phase())
    }

    fn as_complex(&self) -> Complex<f32> {
        Complex::new(self.0 * self.1.sin(), self.0 * self.1.cos())
    }
}

impl std::fmt::Debug for FftPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 > 0.01 {
            write!(f, "FftPoint({:+.5}, {:+.5})", self.0, self.1)
        } else {
            write!(f, "FftPoint({:+.5},        x)", self.0)
        }
    }
}

impl std::fmt::Display for FftPoint {
    /// Display an FftPoint
    /// 
    /// By default, this will print an ANSI color reset character after the point,
    /// if this is not needed (for example to print many points), you can specify
    /// `alternate()` (with `#`), which will not print an ANSI color reset.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.amplitude() > 0.0075 {
            if f.alternate() {
                write!(f, "{}{}", self.color(), self.amp_char())?;
            } else {
                write!(f, "{}{}{}", self.color(), self.amp_char(), Reset)?;
            }
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
            write!(f, "{:#}", point)?;
        }

        write!(f, "{}]", Reset)?;
        Ok(())
    }
}
