use std::io::Write;

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

#[test]
fn test_encode_and_decode() {
    let sample_rate: f32 = 48000.0;
    let mut buffer = [0.0; 2048];
    let fbucket = fbucket(sample_rate, buffer.len());

    let config = ChannelConfig::new(fbucket);

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
