use std::{time::Duration, io::Write, thread::sleep};

use crossbeam::channel::Sender;

use super::soundgen::{SoundCommand, FrequencyComponent};

struct Encoder {
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
        Self {
            commands,

            fbucket,

            channels: 1,
            base_channel: 14,
            step: 5,

            amplitude_buckets: 1,
            phase_buckets: 4,

            symbol_duration: Duration::from_secs(1),
            pause_duration: Duration::from_millis(500),
        }
    }

    fn send_calibration(&self) {
        self.commands.send(SoundCommand::ClearWaveform);
        sleep(Duration::from_millis(100));
        self.commands.send(SoundCommand::SetVolume(0.0));

        for i in 0..self.channels {
            let freq = self.bucket_frequency(i);
            let wave = FrequencyComponent::new_simple(freq);
            self.commands.send(SoundCommand::AddWaveform(wave));
        }

        self.commands.send(SoundCommand::TransitionVolume(0.1));
        sleep(self.symbol_duration);
        self.commands.send(SoundCommand::TransitionVolume(0.0));
        sleep(self.pause_duration);
    }

    /// Invariants: when you call this method, the volume is 0 and the quiet
    /// period has already passed.
    fn send_symbol(&self, mut data: u64) {
        self.commands.send(SoundCommand::ClearWaveform);

        let bits = self.bits_per_symbol();
        let mask = 2_u64.pow(bits) - 1;

        for i in 0..self.channels {
            let bits = data & mask;
            data >>= bits;
        }
    }

    fn bits_per_symbol(&self) -> u32 {
        log2(self.amplitude_buckets) * log2(self.phase_buckets) * self.channels as u32
    }

    fn bucket_frequency(&self, bucket: usize) -> f32 {
        (self.base_channel + bucket * self.step) as f32 * self.fbucket
    }
}

impl Write for Encoder {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(0)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn log2(v: usize) -> u32 {
    std::mem::size_of::<usize>() as u32 * 8 - v.leading_zeros() - 1
}