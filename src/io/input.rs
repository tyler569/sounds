use cpal::{traits::*, BufferSize, Device, Stream, StreamConfig};
use crossbeam::channel;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::mpsc::{channel, SendError};

use super::types::{Fft, Samples};
use crate::color::Reset;

fn device() -> Device {
    // cpal::default_host().default_input_device().unwrap()

    cpal::default_host()
        .input_devices()
        .unwrap()
        .find(|d| d.name().unwrap() == "MacBook Air Microphone")
        .unwrap()
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

pub struct InputStream {
    stream: Stream,
    samples_channel: channel::Receiver<Samples>,
    offset_channel: channel::Sender<usize>,
    sample_rate: u32,
}

impl InputStream {
    pub fn new(sample_rate_try: u32, buffer_size: usize) -> Self {
        let device = device();
        let config = config(&device, sample_rate_try, buffer_size);

        let channels = config.channels as usize;
        let sample_rate = config.sample_rate.0;
        let (samples_send, samples_channel) = channel::unbounded();

        let (offset_channel, offset_recv) = channel::bounded(0);
        let mut offset_buffer = vec![0.; buffer_size * channels];
        let mut offset = 0;

        let stream = device
            .build_input_stream(
                &config,
                move |buf, _info| {
                    if let Ok(relative_offset) = offset_recv.try_recv() {
                        offset += relative_offset * channels;
                        offset %= buf.len();

                        let up_to = buf.len() - offset;
                        offset_buffer[..offset].copy_from_slice(&buf[up_to..]);
                    } else {
                        let up_to = buf.len() - offset;
                        offset_buffer[offset..].copy_from_slice(&buf[..up_to]);
                        samples_send.send(Samples::new(&offset_buffer, channels));
                        offset_buffer[..offset].copy_from_slice(&buf[up_to..]);
                    }
                },
                move |err| eprintln!("error: {:?}", err),
            )
            .unwrap();
        stream.play();

        Self {
            stream,
            samples_channel,
            offset_channel,
            sample_rate,
        }
    }

    pub fn channel(&self) -> &channel::Receiver<Samples> {
        &self.samples_channel
    }

    pub fn add_offset(&self, offset: usize) {
        self.offset_channel.send(offset);
    }
}
