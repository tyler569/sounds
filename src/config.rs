use std::{time::Duration, ops::{Range, RangeInclusive}};

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
        self.channel_base - 2 .. self.channel_top() + 1
    }
    
}