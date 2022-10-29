use std::thread::sleep;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    Device,
    Host,
    InputCallbackInfo,
    InputStreamTimestamp,
    StreamConfig,
    Stream,
};
use ringbuf::HeapRb;
use crate::types::{Result, SoundRead};

const RINGBUF_SIZE: usize = 32 * 1024;

fn device() -> Device {
    cpal::default_host().default_input_device().unwrap()
}

fn config(device: &Device) -> StreamConfig {
    device
        .supported_input_configs()
        .unwrap()
        .max_by_key(|c| c.max_sample_rate())
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
                break
            }
            sleep(Duration::from_millis(1));
        }
        total
    }
}

impl SoundRead for InputStream {
    fn read(&mut self, buffer: &mut [f32]) -> Result<usize> {
        Ok(self.pop_slice(buffer))
    }
}

pub fn input() -> InputStream {
    let device = device();
    let config = config(&device);

    let mut ring = buffer();
    let (mut inp, mut out) = ring.split();

    let stream = device.build_input_stream(
        &config,
        move |buf, info| {
            inp.push_slice(buf);
        },
        move |err| {
            eprintln!("input stream error: {:?}", err);
        }
    ).unwrap();

    InputStream {
        stream,
        config,
        ringbuf: out,
    }
}