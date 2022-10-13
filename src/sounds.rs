use cpal::StreamConfig;
use rand::seq::index::sample;

#[derive(Debug)]
pub struct FrequencyComponent {
    frequency: f32,
    phase: f32,
    relative_volume: f32,
}

impl FrequencyComponent {
    pub const fn new_simple(frequency: f32) -> Self {
        Self {
            frequency,
            phase: 0.0,
            relative_volume: 1.0,
        }
    }

    pub const fn new_volume(frequency: f32, relative_volume: f32) -> Self {
        Self {
            frequency,
            phase: 0.0,
            relative_volume,
        }
    }

    pub const fn new(frequency: f32, phase: f32, relative_volume: f32) -> Self {
        Self {
            frequency,
            phase,
            relative_volume,
        }
    }
}

#[derive(Debug)]
pub struct SoundGenerator {
    sample_clock: f32,
    sample_rate: f32,

    volume: f32,
    volume_target: f32,
    volume_transition: f32,
    
    waveform: Vec<FrequencyComponent>,
}

impl SoundGenerator {
    pub fn new(sample_rate: f32) -> Self {
        SoundGenerator {
            sample_rate,
            sample_clock: 0.0,
            volume: 0.1,
            volume_target: 0.1,
            volume_transition: 0.0,
            waveform: vec![
                FrequencyComponent::new_simple(190.4762),
                FrequencyComponent::new_simple(239.4558),
            ],
        }
    }

    pub fn push_frequency(&mut self, frequency: f32) {
        self.waveform.push(FrequencyComponent::new_simple(frequency));
    }

    pub fn tick(&mut self) -> f32 {
        self.sample_clock += 1.;

        if self.volume_transition > 0.0 {
            let volume_diff = self.volume_target - self.volume;
            let volume_step = volume_diff / (self.volume_transition * self.sample_rate);

            self.volume += volume_step;
            self.volume_transition -= 1./self.sample_rate;
        }

        let total_volume: f32 = self.waveform.iter().map(|w| w.relative_volume).sum();

        let raw_sample: f32 = self.waveform.iter().map(|w| {
            (self.sample_clock * w.frequency * 2. * std::f32::consts::PI / self.sample_rate + w.phase).sin() * w.relative_volume / total_volume
        }).sum();

        assert!(raw_sample <= 1.0 && raw_sample >= -1.0);

        raw_sample * self.volume
    }
}

pub fn test_sound_generator() {
    let mut generator = SoundGenerator::new(44100.);

    for i in 0..100 {
        let mut buffer = [0f32; 4410];
        for sample in buffer.iter_mut() {
            *sample = generator.tick();
        }

        buffer.map(|c| println!("{}", c));
    }
}