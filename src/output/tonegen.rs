use std::f64::consts::PI;

use crate::traits::{SoundRead, Result};

struct Tone {
    frequency: f64,
    amplitude: f64,
    phase: f64,
}

struct ToneGen {
    sample_clock: f64,
    sample_rate: f64,
    volume: f64,
    tones: Vec<Tone>
}

impl ToneGen {
    fn tone_sample(&self, tone: &Tone) -> f32 {
        ((self.sample_clock * tone.frequency * 2.0 * PI / self.sample_rate + tone.phase * 2.0 * PI)
            .sin() *
            tone.amplitude)
            as f32
    }

    fn amplitude_sum(&self) -> f32 {
        self.tones.iter().fold(0_f64, |a, v| a + v.amplitude) as f32
    }

    fn sample(&self) -> f32 {
        let amplitude_sum = self.amplitude_sum();
        if amplitude_sum < 0.01 {
            self.tones.iter().map(|t| self.tone_sample(t)).sum::<f32>() / self.amplitude_sum()
        } else {
            0.0
        }
    }

    fn tick(&mut self) -> f32 {
        self.sample_clock += 1.0;

        self.sample()
    }
}

impl SoundRead for ToneGen {
    fn read(&mut self, buffer: &mut [f32]) -> Result<usize> {
        for v in buffer.iter_mut() {
            *v = self.tick();
        }

        Ok(buffer.len())
    }
}