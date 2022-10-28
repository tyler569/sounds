use std::collections::btree_set::SymmetricDifference;

use crate::{config::ChannelConfig, fft::FftDecoder, listen::Decoder, traits::SoundWrite, bit_org::BitOrg};

use super::differential_decode::{DifferentialDecoder, DecodeResult};

pub struct DataDecoder {
    config: ChannelConfig,
    decoders: Vec<DifferentialDecoder>,
    cache: Vec<Option<u64>>,
    last_symbol: Option<u64>,

    data: BitOrg,
}

impl DataDecoder {
    pub fn new(config: ChannelConfig) -> Self {
        let mut s = Self {
            config,
            decoders: Vec::new(),
            cache: Vec::new(),
            last_symbol: None,

            data: BitOrg::new(),
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

    pub fn sample(&mut self, buffer: &[f32]) -> Option<u64> {
        let fft = FftDecoder::perform(buffer);
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
            .map(|v| { eprint!(" {:?}", v); v })
            .rev()
            .fold(Some(0), |a, v| Self::fold_channels_to_symbol(bits_per_channel, a, v));

        if symbol.is_none() {
            self.last_symbol = None;
        }

        eprintln!(" {:?}", symbol);

        if symbol == self.last_symbol {
            None
        } else {
            self.last_symbol = symbol;
            symbol
        }
    }

    fn push_unread_data(&mut self, symbol: u64) {
    }
}
        

impl SoundWrite for DataDecoder {
    fn write(&mut self, buffer: &[f32]) -> crate::traits::Result<usize> {
        let symbol = self.sample(buffer);

        if let Some(symbol) = symbol {
            self.data.push_bits(self.config.bits_per_symbol(), symbol);
        }

        Ok(buffer.len())
    }
}

impl std::io::Read for DataDecoder {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.data.read(buf)
    }
}