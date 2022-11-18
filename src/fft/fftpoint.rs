use num_complex::{Complex, ComplexFloat};
use std::f32::consts::PI;
use std::fmt::{Debug, Display, Formatter, Result};
use crate::color::{Color, color, reset};

#[derive(Copy, Clone, PartialEq)]
pub struct FftPoint(Complex<f32>);

impl FftPoint {
    pub fn new(value: Complex<f32>) -> Self {
        Self(value)
    }

    pub fn amplitude(&self) -> f32 {
        self.0.abs()
    }

    pub fn phase(&self) -> f32 {
        let mut phase = 0.0;
        if self.amplitude() > 0.01 {
            phase = self.0.im.atan2(self.0.re);
        }
        phase
    }

    fn character(&self) -> char {
        match self.amplitude() {
            x if x < 0.0 => '?',
            x if x < 2.0 => ' ',
            x if x < 4.0 => '.',
            x if x < 10.0 => '*',
            x if x < 20.0 => '#',
            _ => '■',
        }
    }

    fn color(&self) -> Color {
        self.phase().into()
    }
}

impl Debug for FftPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "(a:{:.3}, p:{:.3})", self.amplitude(), self.phase())
    }
}

impl Display for FftPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let c = self.character();
        if c == ' ' {
            write!(f, "{}", self.character())
        } else {
            color(f, self.color())?;
            write!(f, "{}", self.character())?;
            reset(f)
        }
    }
}
