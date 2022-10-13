use cpal::StreamConfig;
use rand::seq::index::sample;

#[derive(Debug)]
pub struct SoundGenerator {
    sample_clock: f32,
    sample_rate: f32,

    volume: f32,
    volume_target: f32,
    volume_transition: f32,
    
    frequency: Vec<f32>,
}

impl SoundGenerator {
    pub fn new(sample_rate: f32) -> Self {
        SoundGenerator {
            sample_rate,
            sample_clock: 0.,
            volume: 0.1,
            volume_target: 0.1,
            volume_transition: 0.,
            // frequency: vec![1000., 2060., 2970., 4010., 5150.],
            frequency: vec![
                350. * 240. / 441.,
                440. * 240. / 441.,
            ],
        }
    }

    pub fn push_frequency(&mut self, frequency: f32) {
        self.frequency.push(frequency);
    }

    pub fn tick(&mut self) -> f32 {
        self.sample_clock += 1.;

        if self.volume_transition > 0. {
            let volume_diff = self.volume_target - self.volume;
            let volume_step = volume_diff / (self.volume_transition * self.sample_rate);

            self.volume += volume_step;
            self.volume_transition -= 1./self.sample_rate;
        }

        self.frequency.iter().map(|f| {
            (self.sample_clock * f * 10. * std::f32::consts::FRAC_2_PI / self.sample_rate).sin() * self.volume / (self.frequency.len() as f32)
        }).sum()
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