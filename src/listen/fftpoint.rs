use num_complex::{Complex, ComplexFloat};
use std::f32::consts::PI;

pub struct FftPoint {
    pub amplitude: f32,
    pub complex: Complex<f32>,
    pub frequency: f32,
    pub phase: f32, // positive, * 1/PI
}

impl FftPoint {
    pub fn new(fbucket: f32, value: (usize, Complex<f32>)) -> Self {
        let mut phase = value.1.im.atan2(value.1.re) / PI;
        if phase < 0.0 {
            phase += 1.0
        }

        Self {
            complex: value.1,
            amplitude: value.1.abs(),
            frequency: value.0 as f32 * fbucket,
            phase,
        }
    }

    fn character(&self) -> char {
        match self.amplitude {
            x if x < 0.0 => '?',
            x if x < 2.0 => ' ',
            x if x < 4.0 => '.',
            x if x < 10.0 => '*',
            x if x < 20.0 => '#',
            _ => '■',
        }
    }

    fn color(&self) -> Color {
        self.phase.into()
    }
}

impl std::fmt::Debug for FftPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(
                f,
                "Frequency: {:9.2} Phase: {:5.3} Amplitude: {:6.3}",
                self.frequency, self.phase, self.amplitude
            )
        } else {
            write!(
                f,
                "f:{:.2} p:{:.3} a:{:.3}",
                self.frequency, self.phase, self.amplitude
            )
        }
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

fn color(f: &mut std::fmt::Formatter<'_>, c: Color) -> std::fmt::Result {
    write!(f, "\x1b[38;2;{};{};{}m", c.0, c.1, c.2)
}

fn reset(f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\x1b[0m")
}

impl std::fmt::Display for FftPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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