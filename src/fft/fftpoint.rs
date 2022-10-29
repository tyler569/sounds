use num_complex::{Complex, ComplexFloat};
use std::f32::consts::PI;
use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Copy, Clone, PartialEq)]
pub struct FftPoint(Complex<f32>);

impl FftPoint {
    pub fn new(value: Complex<f32>) -> Self {
        Self(value)
    }

    pub fn amplitude(&self) -> f32 {
        self.0.abs()
    }

    pub fn phase(&self) -> f32  {
        let mut phase = 0.0;
        if self.amplitude() > 0.01 {
            phase = self.0.im.atan2(self.0.re) / PI;
            if phase < 0.0 {
                phase += 1.0
            }
        }
        phase
    }

    pub fn phase_pi(&self) -> f32 {
        self.phase() * PI
    }

    pub fn phase_2pi(&self) -> f32 {
        self.phase() * 2.0 * PI
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
        write!(
            f,
            "(a:{:.3}, p:{:.3})",
            self.amplitude(), self.phase()
        )
    }
}

#[derive(Copy, Clone, Debug)]
struct Color(u8, u8, u8);

impl From<f32> for Color {
    fn from(phase: f32) -> Self {
        let circle = phase * PI * 2.0;
        let r = ((circle + 0.0).sin() * 127.0 + 128.0) as u8;
        let g = ((circle + 2.0).sin() * 127.0 + 128.0) as u8;
        let b = ((circle + 4.0).sin() * 127.0 + 128.0) as u8;
        Color(r, g, b)
    }
}

fn color(f: &mut Formatter<'_>, c: Color) -> Result {
    write!(f, "\x1b[38;2;{};{};{}m", c.0, c.1, c.2)
}

fn reset(f: &mut Formatter<'_>) -> Result {
    write!(f, "\x1b[0m")
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
