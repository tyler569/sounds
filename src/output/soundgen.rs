use std::f64::consts::PI;

use cpal::StreamConfig;
use crossbeam::channel::Receiver;

use crate::traits::{Result, SoundRead};

#[derive(Debug, Copy, Clone)]
pub struct FrequencyComponent {
    pub(super) frequency: f64,
    pub(super) phase: f64,
    pub(super) relative_volume: f64,
}

impl FrequencyComponent {
    pub fn new_simple(f: impl Into<f64>) -> Self {
        Self {
            frequency: f.into(),
            phase: 0.0,
            relative_volume: 1.0,
        }
    }

    pub fn new_volume(f: impl Into<f64>, v: impl Into<f64>) -> Self {
        Self {
            frequency: f.into(),
            phase: 0.0,
            relative_volume: v.into(),
        }
    }

    pub fn new(f: impl Into<f64>, p: impl Into<f64>, v: impl Into<f64>) -> Self {
        Self {
            frequency: f.into(),
            phase: p.into(),
            relative_volume: v.into(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SoundCommand {
    SetVolume(f64),
    TransitionVolume(f64),
    AddWaveform(FrequencyComponent),
    RemoveWaveform(f64),
    ClearWaveform,
}

#[derive(Debug)]
pub struct SoundGenerator {
    sample_clock: f64,
    sample_rate: f64,

    volume: f64,
    volume_target: f64,
    volume_transition: f64,

    waveform: Vec<FrequencyComponent>,

    commands: Option<Receiver<SoundCommand>>,
}

impl SoundGenerator {
    pub fn new(sample_rate: f32, commands: Option<Receiver<SoundCommand>>) -> Self {
        SoundGenerator {
            sample_rate: sample_rate as f64,
            sample_clock: 0.0,
            volume: 0.1,
            volume_target: 0.1,
            volume_transition: 0.0,
            waveform: vec![],
            commands,
        }
    }

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
            (self.sample_clock * w.frequency * 2.0 * PI / self.sample_rate + w.phase).sin()
                * w.relative_volume
                / total_volume
        };

        waveform.iter().map(sample_single).sum()
    }

    fn receive_command(&mut self) {
        if let Some(ref receiver) = self.commands {
            if let Ok(command) = receiver.try_recv() {
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
    }

    fn do_volume_transition(&mut self) {
        if self.volume_transition > 0.0 {
            let volume_diff = self.volume_target - self.volume;
            let volume_step = volume_diff / (self.volume_transition * self.sample_rate);

            self.volume += volume_step;
            self.volume_transition -= 1. / self.sample_rate;
        }
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

        (raw_sample * self.volume) as f32
    }
}

impl SoundRead for SoundGenerator {
    fn read(&mut self, buffer: &mut [f32]) -> Result<usize> {
        buffer.iter_mut().for_each(|v| *v = self.tick());
        Ok(buffer.len())
    }
}
