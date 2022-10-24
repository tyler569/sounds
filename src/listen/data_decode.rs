use std::collections::btree_set::SymmetricDifference;

use crate::{config::ChannelConfig, fft::FftDecoder, listen::Decoder};

use super::differential_decode::{DifferentialDecoder, DecodeResult};

pub struct DataDecoder {
    config: ChannelConfig,

    decoders: Vec<DifferentialDecoder>,

    cache: Vec<Option<u64>>,

    last_symbol: Option<u64>,

    acc: u64,
    bits_in_acc: usize,
}

impl DataDecoder {
    pub fn new(config: ChannelConfig) -> Self {
        let mut s = Self {
            config,

            decoders: Vec::new(),

            cache: Vec::new(),

            last_symbol: None,

            acc: 0,
            bits_in_acc: 0,
        };

        for i in s.config.channels() {
            s.decoders.push(DifferentialDecoder::new(config.phase_buckets()));
            s.cache.push(None);
        }

        s
    }

    pub fn sample(&mut self, sample_rate: f32, buffer: &[f32]) -> Option<char> {
        let fft = FftDecoder::perform(sample_rate, buffer);
        let mut decoded = Vec::with_capacity(self.decoders.len());

        fft.print_channel_range(self.config.channels_range());

        self.config.channels().enumerate().for_each(|(i, c)|
            decoded.push(self.decoders[i].sample(&fft.point(c)))
        );

        eprint!(" {:?}", decoded);

        let mut decoded = decoded.iter().enumerate().map(|(i, v)| {
            match v {
                DecodeResult::Noise => {
                    self.cache[i] = None;
                    None
                }
                DecodeResult::Signal(v) => {
                    self.cache[i] = Some(*v);
                    self.cache[i]
                }
                DecodeResult::SameSignal => {
                    self.cache[i]
                }
            }
        }).collect::<Vec<_>>();

        eprint!(" {:?}", decoded);

        let symbol = if decoded.iter().all(|v| v.is_some()) {
            Some(decoded
                .iter()
                .map(|v| v.unwrap())
                .rev()
                .fold(0, |a, v| (a << 2) + v))
        } else {
            None
        };

        if symbol.is_none() {
            self.last_symbol = None;
        }

        let u = if symbol == self.last_symbol {
            None
        } else {
            self.last_symbol = symbol;
            symbol
        };

        u.and_then(|v| char::from_u32(v as u32))
    }
}