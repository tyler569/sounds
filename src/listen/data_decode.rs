use crate::{config::ChannelConfig, fft::FftDecoder, listen::Decoder};

use super::differential_decode::DifferentialDecoder;

pub struct DataDecoder {
    config: ChannelConfig,

    decoders: Vec<DifferentialDecoder>,

    cache: Vec<Option<char>>,
}

impl DataDecoder {
    pub fn new(config: ChannelConfig) -> Self {
        let mut s = Self {
            config,

            decoders: Vec::new(),

            cache: Vec::new(),
        };

        for i in s.config.channels() {
            s.decoders.push(DifferentialDecoder::new(config.phase_buckets()))
        }

        s
    }

    pub fn sample(&mut self, sample_rate: f32, buffer: &[f32]) -> Option<char> {
        let fft = FftDecoder::perform(sample_rate, buffer);
        let mut decoded = Vec::with_capacity(self.decoders.len());

        self.config.channels().enumerate().for_each(|(i, c)|
            decoded.push(self.decoders[i].sample(&fft.point(c)))
        );

        eprint!("DataDecoder: {:?}", decoded);

        if decoded.iter().all(Option::is_some) {
            let v = decoded
                .iter()
                .map(|v| v.unwrap())
                .rev()
                .fold(0, |a, v| (a << 2) + v);
            Some(char::from_u32(v as u32).unwrap())
        } else {
            None
        }
    }
}