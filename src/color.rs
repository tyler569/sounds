use std::fmt::{Formatter, Result, Display};

#[derive(Copy, Clone, Debug)]
pub struct Color(u8, u8, u8);

impl Color {
    pub fn from_phase(phase: f32) -> Self {
        phase.into()
    }

    pub fn write(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "\x1b[38;2;{};{};{}m", self.0, self.1, self.2)
    }
}

impl From<f32> for Color {
    fn from(phase: f32) -> Self {
        let circle = phase;
        let r = ((circle + 0.0).sin() * 127.0 + 128.0) as u8;
        let g = ((circle + 2.0).sin() * 127.0 + 128.0) as u8;
        let b = ((circle + 4.0).sin() * 127.0 + 128.0) as u8;
        Color(r, g, b)
    }
}

pub fn color(f: &mut Formatter<'_>, c: Color) -> Result {
    write!(f, "\x1b[38;2;{};{};{}m", c.0, c.1, c.2)
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.write(f)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Reset;

impl Reset {
    pub fn write(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "\x1b[0m")
    }
}

impl Display for Reset {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.write(f)
    }
}

pub fn reset(f: &mut Formatter<'_>) -> Result {
    write!(f, "\x1b[0m")
}
