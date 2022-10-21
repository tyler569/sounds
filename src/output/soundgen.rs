use std::f64::consts::PI;

use cpal::StreamConfig;
use crossbeam::channel::Receiver;

#[derive(Debug)]
pub struct FrequencyComponent {
    frequency: f64,
    phase: f64,
    relative_volume: f64,
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

#[derive(Debug)]
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
            waveform: vec![
                // FrequencyComponent::new_simple(340.0),
                // FrequencyComponent::new_simple(450.0),
            ],
            commands,
        }
    }

    pub fn push_frequency(&mut self, frequency: f64) {
        self.waveform
            .push(FrequencyComponent::new_simple(frequency));
    }

    fn sample(&self, waveform: &[FrequencyComponent]) -> Option<f64> {
        let total_volume: f64 = waveform.iter().map(|w| w.relative_volume).sum();
        if total_volume == 0.0 {
            return None;
        }

        let sample_single = |w: &FrequencyComponent| -> f64 {
            (self.sample_clock * w.frequency * 2.0 * PI / self.sample_rate + w.phase).sin()
                * w.relative_volume
                / total_volume
        };

        let sample: f64 = waveform.iter().map(sample_single).sum();

        Some(sample)
    }

    pub fn tick(&mut self) -> f32 {
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

        self.sample_clock += 1.;

        if self.volume_transition > 0.0 {
            let volume_diff = self.volume_target - self.volume;
            let volume_step = volume_diff / (self.volume_transition * self.sample_rate);

            self.volume += volume_step;
            self.volume_transition -= 1. / self.sample_rate;
        }

        let raw_sample = self.sample(&self.waveform);
        if raw_sample.is_none() {
            return 0.0;
        }
        let raw_sample = raw_sample.unwrap();

        if !(raw_sample <= 1.0 && raw_sample >= -1.0) {
            eprintln!("illegal sample: {}", raw_sample);
            eprintln!("waveform: {:?}", self.waveform);
        }

        assert!(raw_sample <= 1.0 && raw_sample >= -1.0);

        (raw_sample * self.volume) as f32
    }
}

pub fn test_sound_generator() {
    let mut generator = SoundGenerator::new(44100., None);

    for i in 0..100 {
        let mut buffer = [0f32; 4410];
        for sample in buffer.iter_mut() {
            *sample = generator.tick();
        }

        buffer.map(|c| println!("{}", c));
    }
}
