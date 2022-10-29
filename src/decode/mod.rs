use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize, InputStreamTimestamp, Stream, StreamConfig,
};
use num_complex::{Complex, ComplexFloat};
use rustfft::FftPlanner;
use std::{f32::consts::PI, ops::Range};

pub mod data_decode;
pub mod differential_decode;

use crate::decode::differential_decode::DifferentialDecoder;
use crate::{
    decode::data_decode::DataDecoder,
    fft::{FftDecoder, FftPoint},
};

pub trait Decoder {
    fn sample(&mut self, point: &FftPoint) -> Option<u64>;
}

fn choose_buffer_size(config: &StreamConfig, target_fbucket: f32) -> u32 {
    let sample_rate = config.sample_rate.0 as f32;
    let possible_fbuckets = (0..18).map(|v| sample_rate / 2.0 / ((2.0).powf(v as f32)));

    println!(
        "possible buckets: {:?}",
        possible_fbuckets.clone().collect::<Vec<_>>()
    );

    possible_fbuckets
        .enumerate()
        .min_by_key(|(i, v)| ((target_fbucket - v).abs() * 10000.0) as i64)
        .map(|(i, _)| 2u32.pow(i as u32 + 1))
        .unwrap()
}

pub fn listen(target_fbucket: f32) -> (Stream, f32) {
    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();

    let mut config = device.default_input_config().unwrap().config();
    let sample_rate = config.sample_rate.0 as f32;

    let best_buffer = choose_buffer_size(&config, target_fbucket);
    config.buffer_size = BufferSize::Fixed(best_buffer);
    println!("{:?}", config);

    let fbucket = sample_rate / 2.0 / (best_buffer / 2) as f32;
    println!("real fbucket: {}", fbucket);

    let mut first = true;

    // let mut decoder = DataDecoder::new();

    let stream = device
        .build_input_stream(
            &config,
            move |samples: &[f32], info: &cpal::InputCallbackInfo| {
                // decoder.decode
            },
            move |err| eprintln!("{:?}", err),
        )
        .unwrap();

    stream.play().unwrap();

    (stream, fbucket)
}
