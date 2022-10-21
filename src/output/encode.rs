use std::{f32::consts::PI, io::Write, thread::sleep, time::Duration};

use crossbeam::channel::Sender;

use super::soundgen::{FrequencyComponent, SoundCommand};

pub struct Encoder {
    commands: Sender<SoundCommand>,

    fbucket: f32,

    channels: usize,
    base_channel: usize,
    step: usize,

    amplitude_buckets: usize,
    phase_buckets: usize,

    symbol_duration: Duration,
    pause_duration: Duration,
}

impl Encoder {
    pub fn new(fbucket: f32, commands: Sender<SoundCommand>) -> Self {
        let encoder = Self {
            commands,

            fbucket,

            channels: 4,
            base_channel: 14,
            step: 2,

            amplitude_buckets: 1,
            phase_buckets: 4,

            symbol_duration: Duration::from_millis(200),
            pause_duration: Duration::from_millis(100),
        };

        println!("Encoder!");

        assert!(encoder.amplitude_buckets.is_power_of_two());
        assert!(encoder.phase_buckets.is_power_of_two());

        encoder
    }

    pub fn send_calibration(&self) {
        self.commands.send(SoundCommand::ClearWaveform);
        sleep(Duration::from_millis(100));
        self.commands.send(SoundCommand::SetVolume(0.0));

        for i in 0..self.channels {
            let freq = self.channel_frequency(i);
            let wave = FrequencyComponent::new_simple(freq);
            self.commands.send(SoundCommand::AddWaveform(wave));
        }

        self.commands.send(SoundCommand::TransitionVolume(0.1));
        sleep(self.symbol_duration * 2);
        self.commands.send(SoundCommand::TransitionVolume(0.0));
        sleep(self.pause_duration);
    }

    /// Invariants: when you call this method, the volume is 0 and the quiet
    /// period has already passed.
    pub fn send_symbol(&self, mut data: u64) {
        self.commands.send(SoundCommand::ClearWaveform);

        let bits = self.bits_per_symbol() / self.channels();
        let mask = 2_u64.pow(bits) - 1;

        // println!("send symbol: {:08b}", data);

        for channel in 0..self.channels {
            let mut channel_data = data & mask;
            data >>= bits;

            /*
            let amplitude_mask = self.amplitude_buckets as u64 - 1;
            let amplitude_bucket = channel_data & amplitude_mask;
            channel_data >>= self.amplitude_bits_per_channel();
            */

            let phase_bucket = channel_data;

            assert!(phase_bucket < self.phase_buckets as u64);

            let phase = phase_bucket as f32 / self.phase_buckets as f32 * PI;

            let wave = FrequencyComponent::new(self.channel_frequency(channel), phase, 1.0);
            self.commands.send(SoundCommand::AddWaveform(wave));

            // println!("    channel {}: a{:02b} p{:02b}", channel, 0, phase_bucket);
        }

        self.commands.send(SoundCommand::TransitionVolume(0.1));
        sleep(self.symbol_duration);
        self.commands.send(SoundCommand::TransitionVolume(0.0));
        sleep(self.pause_duration);
    }

    fn bits_per_symbol(&self) -> u32 {
        (self.amplitude_bits_per_channel() + self.phase_bits_per_channel()) * self.channels()
    }

    fn amplitude_bits_per_channel(&self) -> u32 {
        log2(self.amplitude_buckets)
    }

    fn phase_bits_per_channel(&self) -> u32 {
        log2(self.phase_buckets)
    }

    fn channels(&self) -> u32 {
        self.channels as u32
    }

    fn channel_frequency(&self, bucket: usize) -> f32 {
        (self.base_channel + bucket * self.step) as f32 * self.fbucket
    }
}

impl Write for Encoder {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.send_calibration();
        buf.iter().for_each(|v| self.send_symbol(*v as u64));
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn log2(v: usize) -> u32 {
    std::mem::size_of::<usize>() as u32 * 8 - v.leading_zeros() - 1
}
