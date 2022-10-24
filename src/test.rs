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

    let fft = FftDecoder::perform(&buf);

    assert_eq!(fft.peak().channel, 25);
}

fn test_encode_and_decode(config: ChannelConfig, buffer_len: usize) {
    let sample_rate: f32 = 48000.0;
    let mut buffer = vec![0.0; buffer_len];
    let fbucket = fbucket(sample_rate, buffer_len);

    let mut encoder = DifferentialEncoder2::new_config(sample_rate as f64, fbucket, config);
    encoder.send_calibration();
    encoder.write(b"Hello World");

    let mut decoder = DataDecoder::new(config);

    let mut s = Vec::new();

    while !encoder.done() {
        encoder.read(&mut buffer);
        let v = decoder.sample(&buffer);
        if v.is_some() {
            s.push(v.unwrap());
        }
    }

    assert_eq!(&s[1..], &[72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100]);
}

#[test]
fn test_lf_encode_and_decode() {
    test_encode_and_decode(ChannelConfig::new(), 2048);
}

#[test]
fn test_hf_encode_and_decode() {
    let config = ChannelConfig {
        channel_base: 160, /* ~ 15kHz */
        channel_step: 2,
        channels: 4,

        symbol_duration: Duration::from_millis(30),
        pause_duration: Duration::from_millis(20),

        phase_bits: 2,
        amplitude_bits: 0,

        volume: 0.25,
    };

    test_encode_and_decode(config, 512);
}

#[test]
fn fuzz_hf_encode_and_decode_settings() {
    for base in (160..240).step_by(10) {
        let config = ChannelConfig {
            channel_base: base,
            channel_step: 2,
            channels: 4,

            symbol_duration: Duration::from_millis(50),
            pause_duration: Duration::from_millis(50),

            phase_bits: 2,
            amplitude_bits: 0,

            volume: 0.25,
        };

        test_encode_and_decode(config, 512);
    }
}


