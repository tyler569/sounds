use std::{collections::VecDeque, io::Write, thread::sleep, time::Duration};

use cpal::SampleRate;
use crossbeam::channel::Sender;

use crate::traits::{Result, SoundRead};

use super::soundgen::{FrequencyComponent, SoundCommand, SoundGenerator};

#[derive(Debug, Copy, Clone)]
struct TimedCommand {
    command: SoundCommand,
    time: Duration,
}

pub struct DifferentialEncoder2 {
    fbucket: f32,

    channels: usize,
    base_channel: usize,
    step: usize,

    amplitude_buckets: usize,
    phase_buckets: usize,

    symbol_duration: Duration,
    pause_duration: Duration,

    previous_symbol: Vec<u64>,

    queued_duration: Duration,
    current_duration: Duration,

    volume: f64,

    command_queue: VecDeque<TimedCommand>,

    sample_clock: f64,
    sample_rate: f64,

    volume_target: f64,
    volume_transition: f64,

    waveform: Vec<FrequencyComponent>,
}

impl DifferentialEncoder2 {
    pub fn new(sample_rate: f64, fbucket: f32, phase_buckets: usize) -> Self {
        let mut encoder = Self {
            fbucket,

            channels: 4,
            base_channel: 14,
            step: 2,

            amplitude_buckets: 1,
            phase_buckets,

            symbol_duration: Duration::from_millis(150),
            pause_duration: Duration::from_millis(100),

            previous_symbol: Vec::new(),

            queued_duration: Duration::ZERO,
            current_duration: Duration::ZERO,

            volume: 0.1,

            command_queue: VecDeque::new(),

            sample_clock: 0.0,
            sample_rate,

            volume_target: 0.1,
            volume_transition: 0.0,

            waveform: Vec::new(),
        };

        for i in 0..encoder.channels {
            encoder.previous_symbol.push(0);
        }

        println!("DifferentialEncoder2!");

        assert!(encoder.amplitude_buckets.is_power_of_two());
        assert!(encoder.phase_buckets.is_power_of_two());

        encoder
    }

    fn enqueue_action(&mut self, command: SoundCommand, duration: Duration) {
        self.command_queue.push_back(TimedCommand {
            command,
            time: self.queued_duration,
        });
        self.queued_duration += duration;
    }

    fn try_dequeue_command(&mut self) -> Option<SoundCommand> {
        if self.command_queue.is_empty() {
            return None;
        }

        if self.command_queue[0].time <= self.current_duration {
            self.command_queue.pop_front().map(|v| v.command)
        } else {
            None
        }
    }

    pub fn done(&self) -> bool {
        self.current_duration >= self.queued_duration
    }

    // Taken from DifferentialEncoder

    pub fn send_calibration(&mut self) {
        self.off(Duration::from_millis(100));
        self.clear();

        for i in 0..self.channels {
            let freq = self.channel_frequency(i);
            let wave = FrequencyComponent::new_simple(freq);
            self.add(wave);
        }

        self.on(self.symbol_duration * 2);
        self.off(self.pause_duration);
    }

    /// Invariants: when you call this method, the volume is 0 and the quiet
    /// period has already passed.
    pub fn send_symbol(&mut self, mut data: u64) {
        self.clear();

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

            let phase_bucket = channel_data as i64;
            let differential_phase_bucket = (self.previous_symbol[channel] as i64 - phase_bucket)
                .rem_euclid(self.channels as i64)
                as u64;

            assert!(differential_phase_bucket < self.phase_buckets as u64);

            let phase =
                differential_phase_bucket as f32 / self.phase_buckets as f32 * std::f32::consts::PI;

            let wave = FrequencyComponent::new(self.channel_frequency(channel), phase, 1.0);
            self.add(wave);

            self.previous_symbol[channel] = differential_phase_bucket;
        }

        self.on(self.symbol_duration);
        self.off(self.pause_duration);
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

    fn on(&mut self, duration: Duration) {
        self.enqueue_action(SoundCommand::TransitionVolume(self.volume), duration)
    }

    fn off(&mut self, duration: Duration) {
        self.enqueue_action(SoundCommand::TransitionVolume(0.0), duration);
    }

    fn add(&mut self, wave: FrequencyComponent) {
        self.enqueue_action(SoundCommand::AddWaveform(wave), Duration::ZERO);
    }

    fn clear(&mut self) {
        self.enqueue_action(SoundCommand::ClearWaveform, Duration::ZERO);
    }

    // Taken from SoundGenerator

    pub fn push_frequency(&mut self, frequency: f64) {
        self.waveform
            .push(FrequencyComponent::new_simple(frequency));
    }

    pub fn push(&mut self, c: FrequencyComponent) {
        self.waveform.push(c);
    }

    fn sample(&self, waveform: &[FrequencyComponent]) -> f64 {
        let total_volume: f64 = waveform.iter().map(|w| w.relative_volume).sum();
        if total_volume == 0.0 {
            return 0.0;
        }

        let sample_single = |w: &FrequencyComponent| -> f64 {
            (self.sample_clock * w.frequency * 2.0 * std::f64::consts::PI / self.sample_rate
                + w.phase)
                .sin()
                * w.relative_volume
                / total_volume
        };

        waveform.iter().map(sample_single).sum()
    }

    fn receive_command(&mut self) {
        if let Some(command) = self.try_dequeue_command() {
            match command {
                SoundCommand::TransitionVolume(v) => {
                    self.volume_target = v;
                    self.volume_transition = 0.005;
                }
                SoundCommand::SetVolume(v) => self.volume = v,
                SoundCommand::AddWaveform(w) => self.waveform.push(w),
                SoundCommand::RemoveWaveform(f) => self.waveform.retain(|w| w.frequency != f),
                SoundCommand::ClearWaveform => self.waveform.clear(),
            }
        }
    }

    fn do_volume_transition(&mut self) {
        if self.volume_transition > 0.0 {
            let volume_diff = self.volume_target - self.volume;
            let volume_step = volume_diff / (self.volume_transition * self.sample_rate);

            self.volume += volume_step;
            self.volume_transition -= 1. / self.sample_rate;
        }
    }

    fn tick_duration(&self) -> Duration {
        Duration::from_secs_f64(1.0 / self.sample_rate)
    }

    pub fn tick(&mut self) -> f32 {
        self.receive_command();

        self.sample_clock += 1.;

        self.do_volume_transition();

        let raw_sample = self.sample(&self.waveform);

        if !((-1.0..=1.0).contains(&raw_sample)) {
            eprintln!("illegal sample: {}", raw_sample);
            eprintln!("waveform: {:?}", self.waveform);
            assert!(false);
        }

        self.current_duration += self.tick_duration();

        (raw_sample * self.volume) as f32
    }
}

impl Write for DifferentialEncoder2 {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        buf.iter().for_each(|&v| self.send_symbol(v as u64));

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl SoundRead for DifferentialEncoder2 {
    fn read(&mut self, buffer: &mut [f32]) -> Result<usize> {
        if self.done() {
            return Ok(0);
        }

        buffer.iter_mut().for_each(|s| *s = self.tick());

        Ok(buffer.len())
    }
}

fn log2(v: usize) -> u32 {
    std::mem::size_of::<usize>() as u32 * 8 - v.leading_zeros() - 1
}
