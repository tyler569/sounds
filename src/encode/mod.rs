use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig, StreamError};
use crossbeam::channel;

pub mod differential_encode;

fn make_device_and_config() -> (Device, StreamConfig) {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output available");

    let config = device
        .supported_output_configs()
        .expect("Error querying output configs")
        .next()
        .expect("No supported config")
        .with_max_sample_rate()
        .config();

    println!("{:?}", config);

    (device, config)
}

#[derive(Debug, Copy, Clone)]
struct FrequencyComponent {
    frequency: f64,
    phase: f64,
    relative_volume: f64,
}

impl FrequencyComponent {
    fn new_simple(f: impl Into<f64>) -> Self {
        Self {
            frequency: f.into(),
            phase: 0.0,
            relative_volume: 1.0,
        }
    }

    fn new(f: impl Into<f64>, p: impl Into<f64>, a: impl Into<f64>) -> Self {
        Self {
            frequency: f.into(),
            phase: p.into(),
            relative_volume: a.into(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum SoundCommand {
    SetVolume(f64),
    TransitionVolume(f64),
    AddWaveform(FrequencyComponent),
    RemoveWaveform(f64),
    ClearWaveform,
}

#[derive(Debug, Copy, Clone)]
enum Operation {
    AddWaveform(FrequencyComponent),
    ClearWaveform,
}

#[derive(Debug, Copy, Clone)]
struct Command {
    timestamp: Duration,
    operation: Operation,
}
