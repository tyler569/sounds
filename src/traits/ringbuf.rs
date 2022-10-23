use std::cmp::{max, min};

use super::{Result, SoundRead, SoundWrite};

pub struct RingBuf {
    data: Vec<f32>,
    begin: usize,
    end: usize,
}

impl RingBuf {
    const SIZE: usize = 65536;

    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(Self::SIZE),
            begin: 0,
            end: 0,
        }
    }

    pub fn len(&self) -> usize {
        if self.begin <= self.end {
            self.end - self.begin
        } else {
            Self::SIZE - self.begin + self.end
        }
    }

    pub fn i_read(&mut self, buffer: &mut [f32]) -> usize {
        if self.end < self.begin {
            let count = min(Self::SIZE - self.begin, buffer.len());
        }
        
        if self.begin < self.end {
            buffer.copy_from_slice(&self.data[self.begin..])
        }

        0
    }

    pub fn i_write(&mut self, buffer: &[f32]) -> usize {
        0
    }
}

impl SoundRead for RingBuf {
    fn read(&mut self, buffer: &mut [f32]) -> Result<usize> {
        Ok(self.i_read(buffer))
    }
}

impl SoundWrite for RingBuf {
    fn write(&mut self, buffer: &[f32]) -> Result<usize> {
        Ok(self.i_write(buffer))
    }
}