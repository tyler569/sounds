use std::{
    ops::{Range, RangeInclusive},
    time::Duration,
};

use crate::fft;

#[derive(Copy, Clone, Debug)]
pub struct ChannelConfig {
    pub channel_base: usize,
    pub channel_step: usize,
    pub channels: usize,

    pub symbol_duration: Duration,
    pub pause_duration: Duration,

    pub phase_bits: u32,
    pub amplitude_bits: u32,

    pub volume: f64,
}

impl ChannelConfig {
    pub fn new() -> Self {
        Self {
            channel_base: 14,
            channel_step: 2,
            channels: 4,

            symbol_duration: Duration::from_millis(200),
            pause_duration: Duration::from_millis(100),

            phase_bits: 2,
            amplitude_bits: 0,

            volume: 0.1,
        }
    }

    pub fn phase_buckets(&self) -> usize {
        2_usize.pow(self.phase_bits)
    }

    pub fn amplitude_buckets(&self) -> usize {
        2_usize.pow(self.amplitude_bits)
    }

    fn channel_top(&self) -> usize {
        self.channel_base + self.channel_step * self.channels
    }

    pub fn channels(&self) -> impl Iterator<Item = usize> + '_ {
        (self.channel_base..self.channel_top()).step_by(self.channel_step)
    }

    pub fn channels_range(&self) -> Range<usize> {
        self.channel_base.saturating_sub(2)..self.channel_top().saturating_add(1)
    }

    pub fn bits_per_channel(&self) -> u32 {
        self.phase_bits + self.amplitude_bits
    }

    pub fn bits_per_symbol(&self) -> u32 {
        self.bits_per_channel() * self.channels as u32
    }
}

#[derive(Debug, Clone)]
pub enum SoundRange {
    Channels(Range<usize>),
    Frequencies(Range<usize>),
}

impl SoundRange {
    pub fn channels(&self, sample_rate: u32, fft_len: usize) -> Range<usize> {
        match self.clone() {
            Self::Channels(v) => v,
            Self::Frequencies(Range {
                start: min,
                end: max,
            }) => {
                let fbucket = sample_rate as f64 / fft_len as f64;
                let min = (min as f64 / fbucket) as usize;
                let max = (max as f64 / fbucket) as usize + 1;
                min..max
            }
        }
    }

    pub fn channels_side(&self, sample_rate: u32, fft_len: usize, side: usize) -> Range<usize> {
        let Range {
            start: min,
            end: max,
        } = self.channels(sample_rate, fft_len);
        min.saturating_sub(side)..max.saturating_add(side)
    }
}
