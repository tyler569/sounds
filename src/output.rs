use crate::soundgen::{FrequencyComponent, SoundCommand, SoundGenerator};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig, StreamError};
use crossbeam::channel;

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

fn make_output_stream<F>(
    device: Device,
    config: &StreamConfig,
    mut on_window: F,
    mut generator: SoundGenerator,
) -> Stream
where
    F: FnMut(&mut [f32], &cpal::OutputCallbackInfo, &mut SoundGenerator) + Send + Sync + 'static,
{
    device
        .build_output_stream(
            config,
            move |samples: &mut [f32], info: &cpal::OutputCallbackInfo| {
                on_window(samples, info, &mut generator);
            },
            error_callback,
        )
        .unwrap()
}

fn error_callback(err: StreamError) {
    println!("An error occurred: {:?}", err);
}

fn on_window(samples: &mut [f32], info: &cpal::OutputCallbackInfo, generator: &mut SoundGenerator) {
    for chunk in samples.chunks_mut(2) {
        let value = generator.tick();
        for sample in chunk.iter_mut() {
            *sample = value
        }
    }
}

pub fn output() -> (Stream, channel::Sender<SoundCommand>) {
    let (snd, rcv) = channel::unbounded();
    let (device, config) = make_device_and_config();
    let mut generator = SoundGenerator::new(config.sample_rate.0 as f32, Some(rcv));
    println!("{:?}", generator);

    let stream = make_output_stream(device, &config, on_window, generator);

    stream.play().unwrap();

    (stream, snd)
}
