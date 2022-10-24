use crate::config::ChannelConfig;

use super::differential_decode::DifferentialDecoder;

pub struct DataDecoder {
    config: ChannelConfig,

    decoders: Vec<DifferentialDecoder>,
}

impl DataDecoder {
    pub fn new(config: ChannelConfig) -> Self {
        let mut s = Self {
            config,

            decoders: Vec::new(),
        };

        for i in 0..config.channels {
            s.decoders.push(DifferentialDecoder::new(config.phase_buckets()))
        }
    }
}