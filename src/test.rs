use std::{io::{Write, Read}, time::Duration};

use crate::{
    fft::{fbucket, FftDecoder},
    decode::{differential_decode::DifferentialDecoder, Decoder, data_decode::DataDecoder},
    encode::differential_encode::DifferentialEncoder,
    io::{SoundRead, SoundWrite},
    config::ChannelConfig,
};

fn test_encode_and_decode(config: ChannelConfig, buffer_len: usize) -> bool {
    const DATA: &[u8] = b"Hello World";
    let data_u64: Vec<u64> = DATA.iter().map(|&v| v as u64).collect();

    let sample_rate: f32 = 48000.0;
    let mut buffer = vec![0.0; buffer_len];
    let fbucket = fbucket(sample_rate, buffer_len);

    let mut encoder = DifferentialEncoder::new_config(sample_rate as f64, fbucket, config);
    encoder.send_calibration();
    encoder.write(DATA);

    let mut decoder = DataDecoder::new(config);

    let mut s = Vec::new();

    while !encoder.done() {
        encoder.read(&mut buffer);
        let v = decoder.sample(&buffer);
        if v.is_some() {
            s.push(v.unwrap());
        }
    }

    if &s[1..] != data_u64 {
        eprintln!("{:?} != {:?}", &s[1..], data_u64);
        false
    } else {
        true
    }
}

#[test]
fn test_lf_encode_and_decode() {
    assert!(test_encode_and_decode(ChannelConfig::new(), 2048));
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

    assert!(test_encode_and_decode(config, 512));
}

#[test]
#[ignore = "not working"]
fn test_sweep_encode_and_decode() {
    const SAMPLES: usize = 512;

    for channel in 1..SAMPLES/2 - 4 {
        let config = ChannelConfig {
            channel_base: channel,
            channel_step: 1,
            channels: 4,

            symbol_duration: Duration::from_millis(21),
            pause_duration: Duration::from_millis(14),

            phase_bits: 2,
            amplitude_bits: 0,

            volume: 0.25,
        };

        if !test_encode_and_decode(config, SAMPLES) {
            eprintln!("sweep failed at {}", channel);
            assert!(false);
        }
    }
}

#[test]
#[ignore = "slow"]
fn fuzz_hf_encode_and_decode_settings() {
    let mut best = (100, 100);

    for sym in 6..30 {
        for pause in 6..30 {
            let config = ChannelConfig {
                channel_base: 24,
                channel_step: 1,
                channels: 4,

                symbol_duration: Duration::from_millis(sym),
                pause_duration: Duration::from_millis(pause),

                phase_bits: 2,
                amplitude_bits: 0,

                volume: 0.25,
            };
            if test_encode_and_decode(config, 512) {
                if sym + pause < best.0 + best.1 {
                    best = (sym, pause);
                }
            }
        }
    }

    eprintln!("best: {}, {}", best.0, best.1);
}

#[test]
#[ignore = "not working, fixing encoder"]
fn test_encode_and_decode_to_bytes() {
    const DATA: &[u8] = b"Hello World";
    const BUFFER_LEN: usize = 512;

    let sample_rate: f32 = 48000.0;
    let mut buffer = vec![0.0; BUFFER_LEN];
    let fbucket = fbucket(sample_rate, BUFFER_LEN);
    let mut output = [0u8; 64];

    let config = ChannelConfig {
        channel_base: 150,
        channel_step: 1,
        channels: 7,
        symbol_duration: Duration::from_millis(50),
        pause_duration: Duration::from_millis(50),
        phase_bits: 2,
        amplitude_bits: 0,
        volume: 0.4,
    };

    let mut encoder = DifferentialEncoder::new_config(sample_rate as f64, fbucket, config);
    encoder.send_calibration();
    encoder.write(DATA);

    let mut decoder = DataDecoder::new(config);

    while !encoder.done() {
        encoder.read(&mut buffer);

        decoder.write(&buffer);
    }

    decoder.read(&mut output);
    eprintln!("{:?}", output);
    assert_eq!(&output[1..DATA.len()+1], DATA);
}
