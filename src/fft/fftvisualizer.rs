use std::{collections::VecDeque, ops::Range};

use num_complex::Complex;
use rustfft::{Fft, FftPlanner};

use crate::{io::SoundRead, fft::FftPoint};

pub struct FftVisualizer<'a> {
    vis_channels: Range<usize>,
    inner: &'a mut dyn SoundRead,
    cplx_buffer: Vec<Complex<f32>>,
}

impl<'a> FftVisualizer<'a> {
    pub fn new(inner: &'a mut dyn SoundRead, channels: Range<usize>) -> Self {
        Self {
            vis_channels: channels,
            inner,
            cplx_buffer: Vec::with_capacity(4096),
        }
    }

    fn visualize(&mut self, buffer: &[f32]) {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(buffer.len());

        self.cplx_buffer.resize(buffer.len(), Complex::new(0.0, 0.0));
        for (i, v) in self.cplx_buffer.iter_mut().enumerate() {
            *v = Complex::new(buffer[i], 0.0);
        }

        let mut cplxs = buffer.iter().map(|&r| Complex::new(r, 0.0)).collect::<Vec<_>>();
        fft.process(&mut cplxs);

        print!("[");

        for channel in self.vis_channels.clone() {
            let p = FftPoint::new(cplxs[channel]);
            print!("{}", p);
        }

        println!("]");
    }
}

impl<'a> SoundRead for FftVisualizer<'a> {
    fn read(&mut self, buffer: &mut [f32]) -> crate::io::Result<usize> {
        let v = self.inner.read(buffer);
        self.visualize(buffer);
        v
    }
}