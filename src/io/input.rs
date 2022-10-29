use std::thread::sleep;
use std::time::Duration;

use crate::io::{Result, SoundRead};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, InputCallbackInfo, InputStreamTimestamp, Stream, StreamConfig};
use ringbuf::HeapRb;

const RINGBUF_SIZE: usize = 32 * 1024;

fn device() -> Device {
    cpal::default_host().default_input_device().unwrap()
}

fn config(device: &Device, sample_rate_try: u32) -> StreamConfig {
    device
        .supported_input_configs()
        .unwrap()
        .min_by_key(|c| (c.max_sample_rate().0 as i64 - sample_rate_try as i64).abs())
        .unwrap()
        .with_max_sample_rate()
        .config()
}

fn buffer() -> HeapRb<f32> {
    HeapRb::new(RINGBUF_SIZE)
}

pub struct InputStream {
    stream: Stream,
    config: StreamConfig,
    ringbuf: ringbuf::HeapConsumer<f32>,

    limit: i32,
}

impl InputStream {
    pub fn pop(&mut self) -> f32 {
        loop {
            match self.ringbuf.pop() {
                Some(v) => return v,
                None => sleep(Duration::from_millis(1)),
            }
        }
    }

    pub fn pop_slice(&mut self, buffer: &mut [f32]) -> usize {
        let mut total = 0;
        loop {
            total += self.ringbuf.pop_slice(&mut buffer[total..]);
            if total == buffer.len() {
                break;
            }
            sleep(Duration::from_millis(1));
        }
        total
    }
}

impl SoundRead for InputStream {
    fn read(&mut self, buffer: &mut [f32]) -> Result<usize> {
        self.limit -= 1;
        if self.limit < 0 {
            // return Ok(0);
        }

        Ok(self.pop_slice(buffer))
    }

    fn sample_rate(&self) -> u32 {
        self.config.sample_rate.0
    }

    fn channels(&self) -> u32 {
        self.config.channels.into()
    }
}

pub fn input(sample_rate_try: u32) -> InputStream {
    let device = device();
    let config = config(&device, sample_rate_try);

    let mut ring = buffer();
    let (mut inp, mut out) = ring.split();

    let stream = device
        .build_input_stream(
            &config,
            move |buf, info| {
                inp.push_slice(buf);
            },
            move |err| {
                eprintln!("input stream error: {:?}", err);
            },
        )
        .unwrap();

    InputStream {
        stream,
        config,
        ringbuf: out,

        limit: 10,
    }
}
