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
use std::sync::{Arc, Condvar, Mutex};
use crate::traits::{Result, SoundWrite};

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
    arc: Arc<(Condvar, Mutex<bool>)>,
}

impl OutputStream {
    fn wait(&mut self) {
        let &(ref cvar, ref mtx) = &*self.arc;
        let guard = mtx.lock().unwrap();
        cvar.wait(guard);
    }

    pub fn push(&mut self, sample: f32) {
        let mut v = self.ringbuf.push(sample);
        while v.is_err() {
            self.wait();
            v = self.ringbuf.push(sample);
        }
        v.unwrap()
    }

    pub fn push_slice(&mut self, buffer: &[f32]) -> usize {
        let mut total = 0;

        while total < buffer.len() {
            total += self.ringbuf.push_slice(&buffer[total..]);
        }

        total
    }
}

impl SoundWrite for OutputStream {
    fn write(&mut self, buffer: &[f32]) -> Result<usize> {
        Ok(self.push_slice(buffer))
    }
}

fn pop_full(buffer: &mut [f32], ring: &mut ringbuf::HeapConsumer<f32>, cvar: &Condvar) {
    let mut total = 0;

    // Could this loop not spin infinitely
    while total < buffer.len() {
        total += ring.pop_slice(&mut buffer[total..]);
        cvar.notify_all()
    }
}

pub fn output() -> OutputStream {
    let device = device();
    let config = config(&device);

    let mut ring = buffer();
    let (mut inp, mut out) = ring.split();
    let cvar = Arc::new((Condvar::new(), Mutex::new(false)));
    let cvar_res = cvar.clone();

    let stream = device.build_output_stream(
        &config,
        move |buf, info| {
            pop_full(buf, &mut out, &cvar.0);
        },
        move |err| {
            eprintln!("input stream error: {:?}", err);
        }
    ).unwrap();

    OutputStream {
        stream,
        config,
        ringbuf: inp,
        arc: cvar_res,
    }
}