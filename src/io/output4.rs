use std::{f32::consts::PI, iter::repeat};
use cpal::{traits::*, BufferSize, Device, Stream, StreamConfig};
use crossbeam::channel;
use super::types4::{Fft, Samples, FftPoint};

fn device() -> Device {
    cpal::default_host()
        .default_output_device()
        .unwrap()
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

pub struct OutputStream {
    stream: Stream,
    samples_channel: channel::Sender<Samples>,
    sample_rate: u32,
}

impl OutputStream {
    pub fn new(sample_rate_try: u32, buffer_size: usize) -> Self {
        let device = device();
        let config = config(&device, sample_rate_try, buffer_size);
        let channels = config.channels as usize;

        let channels = config.channels as usize;
        let sample_rate = config.sample_rate.0;
        let (samples_channel, samples_recv) = channel::bounded::<Samples>(32);

        let stream = device
            .build_output_stream(
                &config,
                move |buf, _info| {
                    if let Ok(samples) = samples_recv.try_recv() {
                        samples.output(buf, channels);
                    }
                },
                |err| eprintln!("output error: {:?}", err),
            )
            .unwrap();
        stream.play();

        Self {
            stream,
            samples_channel,
            sample_rate,
        }
    }

    pub fn channel(&self) -> &channel::Sender<Samples> {
        &self.samples_channel
    }
}
