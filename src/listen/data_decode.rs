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

    fn cache_coalesce(&mut self, i: usize, v: DecodeResult) -> Option<u64> {
        match v {
            DecodeResult::Noise => {
                self.cache[i] = None;
                None
            }
            DecodeResult::Signal(v) => {
                self.cache[i] = Some(v);
                self.cache[i]
            }
            DecodeResult::SameSignal => {
                self.cache[i]
            }
        }
    }

    fn fold_channels_to_symbol(bits: u32, acc: Option<u64>, v: Option<u64>) -> Option<u64> {
        if acc.is_some() && v.is_some() {
            Some((acc.unwrap() << bits) + v.unwrap())
        } else {
            None
        }
    }

    pub fn sample(&mut self, sample_rate: f32, buffer: &[f32]) -> Option<char> {
        let fft = FftDecoder::perform(sample_rate, buffer);
        let mut decoded = Vec::with_capacity(self.decoders.len());

        fft.print_channel_range(self.config.channels_range());

        self.config.channels().enumerate().for_each(|(i, c)|
            decoded.push(self.decoders[i].sample(&fft.point(c)))
        );

        // eprint!(" {:?}", decoded);
        let bits_per_channel = self.config.bits_per_channel();

        let symbol = decoded
            .iter()
            .enumerate()
            .map(|(i, &v)| self.cache_coalesce(i, v))
            // .map(|v| { eprint!(" {:?}", v); v })
            .rev()
            .fold(Some(0), |a, v| Self::fold_channels_to_symbol(bits_per_channel, a, v));

        if symbol.is_none() {
            self.last_symbol = None;
        }

        let result = if symbol == self.last_symbol {
            None
        } else {
            self.last_symbol = symbol;
            symbol
        };

        eprintln!(" {:?}", result.and_then(|v| char::from_u32(v as u32)));

        result.and_then(|v| char::from_u32(v as u32))
    }
}