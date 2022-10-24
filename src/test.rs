use std::io::Write;

use crate::{
    fft::{fbucket, FftDecoder},
    listen::{differential_decode::DifferentialDecoder, Decoder},
    output::{
        differential_encode2::DifferentialEncoder2,
        soundgen::{FrequencyComponent, SoundGenerator},
    },
    traits::SoundRead,
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
    let mut buf = [0.0; 2048];
    let fbucket = fbucket(sample_rate, buf.len());

    let mut encoder = DifferentialEncoder2::new(sample_rate as f64, fbucket);
    encoder.send_calibration();
    encoder.write(b"Hello World");

    let mut decoders = [
        DifferentialDecoder::new(4),
        DifferentialDecoder::new(4),
        DifferentialDecoder::new(4),
        DifferentialDecoder::new(4),
    ];

    let mut s = String::new();

    while !encoder.done() {
        encoder.read(&mut buf);
        let fft = FftDecoder::perform(sample_rate, &buf);
        fft.print_frequency_range(300..575);

        let decoded = [
            decoders[0].sample(&fft.point(14)),
            decoders[1].sample(&fft.point(16)),
            decoders[2].sample(&fft.point(18)),
            decoders[3].sample(&fft.point(20)),
        ];

        eprint!(" {:?}", decoded);

        if decoded.iter().all(|v| v.is_some()) {
            let v = decoded
                .iter()
                .map(|v| v.unwrap())
                .rev()
                .fold(0, |a, v| (a << 2) + v);
            let c = char::from_u32(v as u32).unwrap();
            eprint!(" {:?}", c);
            s.push(c);
        }

        eprintln!();
    }

    assert_eq!(&s[1..], "Hello World");
}
