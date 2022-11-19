use std::{f32::consts::PI, iter::repeat};
use cpal::{traits::*, BufferSize, Device, Stream, StreamConfig};
use crossbeam::channel;
use super::types4::{Fft, Samples, FftPoint};

fn device() -> Device {
    cpal::default_host().default_output_device().unwrap()
}

fn config(device: &Device, sample_rate_try: u32, buffer_size: usize) -> StreamConfig {
    let mut config = device
        .supported_output_configs()
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

    let (send, recv) = channel::bounded::<Samples>(32);

    let stream = device
        .build_output_stream(
            &config,
            move |buf, _info| {
                if let Ok(samples) = recv.try_recv() {
                    samples.output(buf, channels);
                } else {
                    // eprintln!("nothing to output!");
                }
            },
            |err| eprintln!("output error: {:?}", err),
        )
        .unwrap();
    stream.play();

    std::thread::sleep(std::time::Duration::from_millis(20));

    for i in (1..=2) {
        let mut fft = Fft::new(128);
        fft.set_point(i, FftPoint::new(1., 0.));

        println!("{} {:?} {:?}", fft, fft.point(1), fft.point(2));

        let mut samples = fft.into_samples();
        samples.volume(0.02);
        // samples.window_plank();


        for i in 0..20 {
            send.send(samples.clone());
        }
    }

    std::thread::sleep(std::time::Duration::from_millis(150));
    std::process::exit(0);
}