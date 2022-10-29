use std::{collections::VecDeque, ops::Range};

use num_complex::Complex;
use rustfft::{Fft, FftPlanner};

use crate::{config::SoundRange, fft::FftPoint, io::SoundRead};

pub struct FftVisualizer<'a> {
    vis_channels: SoundRange,
    inner: &'a mut dyn SoundRead,
    fft_len: usize,
    cplx_buffer: Vec<Complex<f32>>,
    cplx_i: usize,
}

impl<'a> FftVisualizer<'a> {
    pub fn new(inner: &'a mut dyn SoundRead, fft_len: usize, channels: SoundRange) -> Self {
        Self {
            vis_channels: channels,
            inner,
            fft_len,
            cplx_buffer: vec![Complex::new(0.0, 0.0); fft_len],
            cplx_i: 0,
        }
    }

    fn visualize(&mut self) {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(self.fft_len);

        fft.process(&mut self.cplx_buffer);

        print!("[");

        for channel in self
            .vis_channels
            .channels_side(self.sample_rate(), self.fft_len, 2)
        {
            let p = FftPoint::new(self.cplx_buffer[channel]);
            print!("{}", p);
        }

        println!("]");
    }
}

impl<'a> SoundRead for FftVisualizer<'a> {
    fn read(&mut self, buffer: &mut [f32]) -> crate::io::Result<usize> {
        let result = self.inner.read(buffer);
        if let Ok(0) = result {
            return result;
        }

        let mut vis_ix = 0;
        let channels = self.channels() as usize;

        while vis_ix < buffer.len() {
            let vs = self.cplx_buffer[self.cplx_i..].iter_mut();
            let ss = buffer[vis_ix..].chunks(channels).map(|v| v[0]);

            for (v, s) in vs.zip(ss) {
                *v = Complex::new(s, 0.0);
                self.cplx_i += 1;
                vis_ix += channels;
            }

            if self.cplx_i == self.fft_len {
                self.visualize();
                self.cplx_i = 0;
            }
        }

        result
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    fn channels(&self) -> u32 {
        self.inner.channels()
    }
}
