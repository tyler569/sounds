use cpal::{traits::*, BufferSize, Device, Stream, StreamConfig};
use crossbeam::channel;
use std::io::{Read, Write};
use std::sync::mpsc::{channel, SendError};

use super::types4::{Fft, Samples};
use crate::color::Reset;

fn device() -> Device {
    cpal::default_host().default_input_device().unwrap()
}

fn config(device: &Device, sample_rate_try: u32, buffer_size: usize) -> StreamConfig {
    let mut config = device
        .supported_input_configs()
        .unwrap()
        .min_by_key(|c| (c.max_sample_rate().0 as i64 - sample_rate_try as i64).abs())
        .unwrap()
        .with_max_sample_rate()
        .config();

    config.buffer_size = BufferSize::Fixed(buffer_size as u32);
    config
}

pub fn main() {
    let sample_rate = 48000;
    let buffer_size = 128;

    let device = device();
    let config = config(&device, sample_rate, buffer_size);
    let channels = config.channels as usize;

    let sample_rate = config.sample_rate.0;

    let (send, recv) = channel::unbounded();

    println!("---");
    println!("input v4");
    println!("using {} with config {:?}", device.name().unwrap(), config);
    println!("f0 = {}", sample_rate as f32 / buffer_size as f32);

    let (c_send, c_recv) = channel::bounded::<usize>(0);
    let mut offset = 0;
    let mut last_offset = 0;
    let mut offset_buffer = vec![0f32; buffer_size * channels];

    let stream = device
        .build_input_stream(
            &config,
            move |buf, _info| {
                if let Ok(v) = c_recv.try_recv() {
                    offset += v * channels;
                    offset %= (buf.len());

                    offset_buffer[..offset].copy_from_slice(&buf[buf.len() - offset..])
                } else {
                    offset_buffer[offset..].copy_from_slice(&buf[..buf.len() - offset]);
                    let samples = Samples::new(&offset_buffer, channels);
                    send.send(samples);
                    offset_buffer[..offset].copy_from_slice(&buf[buf.len() - offset..])
                }
            },
            move |err| eprintln!("error: {:?}", err),
        )
        .unwrap();
    stream.play();

    let mut offset = 0;
    let mut last = false;

    for samples in recv.into_iter() {
        let fft = samples.into_fft();
        print!("{} ", fft);

        let point = fft.point(1);
        if point.amplitude() > 2.0 {
            let diff = ((1.0 - point.phase_01()) * buffer_size as f32) as usize;
            if diff > 0 {
                if last {
                    c_send.send(diff as usize);
                }
                last = true;
            } else {
                last = false;
            }
        }

        println!("{:?} {:?}", point, fft.point(2));
    }
}
