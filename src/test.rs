use std::{io::Write, time::Duration};

use crate::{
    fft::{fbucket, FftDecoder},
    listen::{differential_decode::DifferentialDecoder, Decoder, data_decode::DataDecoder},
    output::{
        differential_encode2::DifferentialEncoder2,
        soundgen::{FrequencyComponent, SoundGenerator},
    },
    traits::SoundRead, config::ChannelConfig,
};

#[test]
fn test_gen_and_fft() {
    let sample_rate: f32 = 48000.0;
    let mut gen = SoundGenerator::new(sample_rate, None);
    let mut buf = [0.0; 2048];

    let fbucket = fbucket(sample_rate, buf.len());

    gen.push(FrequencyComponent::new_simple(fbucket * 25.0));
    gen.read(&mut buf);

    let fft = FftDecoder::perform(sample_rate, &buf);

    assert_eq!(fft.peak().frequency, fbucket * 25.0);
}

fn test_encode_and_decode(config: ChannelConfig, buffer_len: usize) {
    let sample_rate: f32 = 48000.0;
    let mut buffer = vec![0.0; buffer_len];
    let fbucket = fbucket(sample_rate, buffer_len);

    let mut encoder = DifferentialEncoder2::new_config(sample_rate as f64, config);
    encoder.send_calibration();
    encoder.write(b"Hello World");

    let mut decoder = DataDecoder::new(config);

    let mut s = String::new();

    while !encoder.done() {
        encoder.read(&mut buffer);
        let v = decoder.sample(sample_rate, &buffer);
        if v.is_some() {
            s.push(v.unwrap());
        }
    }

    assert_eq!(&s[1..], "Hello World");
}

#[test]
fn test_lf_encode_and_decode() {
    let sample_rate: f32 = 48000.0;
    let buffer_len = 2048;
    let fbucket = fbucket(sample_rate, buffer_len);
    test_encode_and_decode(ChannelConfig::new(fbucket), buffer_len);
}

#[test]
fn test_hf_encode_and_decode() {
    let sample_rate: f32 = 48000.0;
    let buffer_len = 512;
    let fbucket = fbucket(sample_rate, buffer_len);

    let config = ChannelConfig {
        fbucket,
        
        channel_base: 160, /* ~ 15kHz */
        channel_step: 2,
        channels: 4,

        symbol_duration: Duration::from_millis(30),
        pause_duration: Duration::from_millis(20),

        phase_bits: 2,
        amplitude_bits: 0,

        volume: 0.25,
    };

    test_encode_and_decode(config, buffer_len);
}


