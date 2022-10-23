use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize, InputStreamTimestamp, Stream,
};
use num_complex::{Complex, ComplexFloat};
use rustfft::FftPlanner;
use std::{f32::consts::PI, ops::Range};

mod channel_decode;
mod differential_decode;
mod fftpoint;

use fftpoint::FftPoint;

use crate::listen::differential_decode::DifferentialDecoder;
use crate::fft::FftDecoder;

trait Decoder {
    fn sample(&mut self, point: &FftPoint) -> Option<u64>;
}

fn to_complexes(f: &[f32], channels: usize) -> Vec<Complex<f32>> {
    f.iter()
        .step_by(channels)
        .map(|v| Complex::new(*v, 0.0))
        .collect()
}

pub fn listen(target_fbucket: f32) -> (Stream, f32) {
    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();

    let mut config = device.default_input_config().unwrap().config();
    let sample_rate = config.sample_rate.0 as f32;

    let possible_fbuckets = (0..18).map(|v| sample_rate / 2.0 / ((2.0).powf(v as f32)));

    println!(
        "possible buckets: {:?}",
        possible_fbuckets.clone().collect::<Vec<_>>()
    );

    let best_buffer = possible_fbuckets
        .enumerate()
        .min_by_key(|(i, v)| ((target_fbucket - v).abs() * 10000.0) as i64)
        .map(|(i, _)| 2u32.pow(i as u32 + 1))
        .unwrap();

    config.buffer_size = BufferSize::Fixed(best_buffer);

    println!("{:?}", config);
    let fbucket = config.sample_rate.0 as f32 / 2.0 / (best_buffer / 2) as f32;
    println!("real fbucket: {}", fbucket);

    let mut decoders = vec![
        DifferentialDecoder::new(4),
        DifferentialDecoder::new(4),
        DifferentialDecoder::new(4),
        DifferentialDecoder::new(4),
    ];

    // std::process::exit(0);

    let mut first = true;

    let stream = device
        .build_input_stream(
            &config,
            move |samples: &[f32], info: &cpal::InputCallbackInfo| {
                let fft = FftDecoder::perform(sample_rate, samples);

                fft.print_frequency_range(400..800);
                eprintln!();
                
                // let decoded = [
                //     decoders[0].sample(&values[14]),
                //     decoders[1].sample(&values[16]),
                //     decoders[2].sample(&values[18]),
                //     decoders[3].sample(&values[20]),
                // ];

                // if decoded.iter().all(|v| v.is_some()) {
                //     let v = decoded
                //         .iter()
                //         .map(|v| v.unwrap())
                //         .rev()
                //         .fold(0, |a, v| (a << 2) + v);
                //     eprint!("({:>7?}) ", char::from_u32(v as u32).unwrap());
                //     // eprint!("{:?}) ", char::from_u32(v as u32 >> 8));
                // } else {
                //     eprint!{"          "}
                // }

                // decoded.iter().for_each(|v| eprint!("{:?} ", v));
                // eprintln!();
            },
            move |err| eprintln!("{:?}", err),
        )
        .unwrap();

    stream.play().unwrap();

    (stream, fbucket)
}
