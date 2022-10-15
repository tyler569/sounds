use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Stream,
};
use rustfft::FftPlanner;
use num_complex::{Complex, ComplexFloat};

fn to_complexes(f: &[f32]) -> Vec<Complex<f32>> {
    f.iter().map(|v| Complex::new(*v, 0.0)).collect()
}

fn max_index(c: &[Complex<f32>]) -> (usize, f32) {
    let mut highest_value = 0.0;
    let mut highest_index = 0;

    for (i, v) in c.iter().enumerate() {
        if v.abs() > highest_value {
            highest_index = i;
            highest_value = v.abs();
        }
    }

    (highest_index, highest_value)
}

pub fn visualize(fft: &[Complex<f32>]) {
    fn character(absv: f32) -> char {
        match absv {
            0.0..=1.0 => ' ',
            1.0..=4.0 => '.',
            4.0..=10.0 => ':',
            10.0..=20.0 => '*',
            20.0.. => '#',
            _ => '?',
        }
    }

    for i in 0..80 {
        eprint!("{}", character(fft[i].abs()))
    }
    eprintln!()
}

fn frequency_step(sample_rate: u32, samples: usize) -> f32 {
    sample_rate as f32 / samples as f32
}

fn to_frequency(step: f32, bucket: usize) -> f32 {
    step * bucket as f32
}

fn display_frequency(step: f32, bucket: usize) {
    eprint!("{:9.2}Hz", to_frequency(step, bucket))
}

pub fn listen() -> Stream {
    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();

    let config = device
        .default_input_config()
        .unwrap()
        .config();

    let stream = device.build_input_stream(
        &config,
        move |samples: &[f32], info: &cpal::InputCallbackInfo| {
            let mut planner = FftPlanner::<f32>::new();
            let fft = planner.plan_fft_forward(samples.len());
            let mut buffer = to_complexes(samples);

            fft.process(&mut buffer);

            let step = frequency_step(config.sample_rate.0, samples.len());

            let max = max_index(&buffer[1..samples.len()/2]);
            display_frequency(step, max.0);
            eprint!("  {:<4} {:>6?} {:>10.4?}  ", samples.len(), max.0, max.1);
            visualize(&buffer);
        },
        move |err| eprintln!("{:?}", err),
    ).unwrap();

    stream.play().unwrap();

    stream
}