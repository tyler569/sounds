use std::thread::sleep;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    Device,
    Host,
    OutputCallbackInfo,
    OutputStreamTimestamp,
    StreamConfig,
    Stream,
};
use ringbuf::HeapRb;
use crate::types::{Result, SoundWrite};

const RINGBUF_SIZE: usize = 32 * 1024;

fn device() -> Device {
    cpal::default_host().default_output_device().unwrap()
}

fn config(device: &Device) -> StreamConfig {
    device
        .supported_output_configs()
        .unwrap()
        .max_by_key(|c| c.max_sample_rate())
        .unwrap()
        .with_max_sample_rate()
        .config()
}

fn buffer() -> HeapRb<f32> {
    HeapRb::new(RINGBUF_SIZE)
}

pub struct OutputStream {
    stream: Stream,
    config: StreamConfig,
    ringbuf: ringbuf::HeapProducer<f32>,
}

impl OutputStream {
    pub fn push(&mut self, sample: f32) {
        loop {
            match self.ringbuf.push(sample) {
                Ok(_) => return,
                Err(_) => sleep(Duration::from_millis(1)),
            }
        }
    }

    pub fn push_slice(&mut self, buffer: &[f32]) -> usize {
        let mut total = 0;
        loop {
            total += self.ringbuf.push_slice(&buffer[total..]);
            if total == buffer.len() {
                break
            }
            sleep(Duration::from_millis(1))
        }
        total
    }
}

impl SoundWrite for OutputStream {
    fn write(&mut self, buffer: &[f32]) -> Result<usize> {
        Ok(self.push_slice(buffer))
    }
}

fn pop_full(buffer: &mut [f32], ring: &mut ringbuf::HeapConsumer<f32>) {
    let mut total = 0;

    while total < buffer.len() {
        total += ring.pop_slice(&mut buffer[total..]);
        sleep(Duration::from_millis(1))
    }
}

pub fn output() -> OutputStream {
    let device = device();
    let config = config(&device);

    let mut ring = buffer();
    let (mut inp, mut out) = ring.split();

    let stream = device.build_output_stream(
        &config,
        move |buf, info| {
            pop_full(buf, &mut out);
        },
        move |err| {
            eprintln!("input stream error: {:?}", err);
        }
    ).unwrap();

    OutputStream {
        stream,
        config,
        ringbuf: inp,
    }
}