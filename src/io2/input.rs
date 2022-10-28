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
use std::sync::{Arc, Condvar, Mutex};
use crate::traits::{Result, SoundRead};

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
    arc: Arc<(Condvar, Mutex<bool>)>,
    on_block: Option<Box<dyn FnMut()>>,
}

impl InputStream {
    fn wait(&mut self) {
        let &(ref cvar, ref mtx) = &*self.arc;
        let guard = mtx.lock().unwrap();
        cvar.wait(guard);
    }

    pub fn pop(&mut self) -> f32 {
        let mut v = self.ringbuf.pop();
        while v.is_none() {
            self.wait();
            v = self.ringbuf.pop();
        }
        v.unwrap()
    }

    pub fn pop_slice(&mut self, buffer: &mut [f32]) -> usize {
        let mut total = 0;

        while total < buffer.len() {
            total += self.ringbuf.pop_slice(&mut buffer[total..]);
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
    let cvar = Arc::new((Condvar::new(), Mutex::new(false)));
    let cvar_res = cvar.clone();

    let stream = device.build_input_stream(
        &config,
        move |buf, info| {
            inp.push_slice(buf);
            cvar.0.notify_all();
        },
        move |err| {
            eprintln!("input stream error: {:?}", err);
        }
    ).unwrap();

    InputStream {
        stream,
        config,
        ringbuf: out,
        arc: cvar_res,
        on_block: None,
    }
}