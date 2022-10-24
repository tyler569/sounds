use num_complex::Complex;
use rustfft::FftPlanner;

pub fn encode() {
    let samples: usize = 128;

    let fbucket = 44100.0 / samples as f32;
    let frequency = fbucket * 4.0;

    let mut buffer = vec![Complex::new(0.0, 0.0); samples];
    buffer[2].re = 0.5;
    buffer[samples - 2].re = 0.5;

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_inverse(buffer.len());
    fft.process(&mut buffer);

    for v in buffer {
        println!("{:?}", v);
    }
}