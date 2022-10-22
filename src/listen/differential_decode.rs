use super::{FftPoint, Decoder};

pub struct DifferentialDecoder {
    phase_buckets: usize,

    last_phase: Option<f32>,
    in_a_row: usize,
}

impl DifferentialDecoder {
    const PHASE_SPECTRUM: f32 = 1.0;

    pub fn new(phase_buckets: usize) -> Self {
        Self {
            phase_buckets,
            in_a_row: 0,
            last_phase: None,
        }
    }

    pub fn is_signal(&mut self, point: &FftPoint) -> bool {
        if point.amplitude < 4.0 {
            self.in_a_row = 0;
            return false
        }

        self.in_a_row += 1;
        self.in_a_row == 2
    }

    fn phase_bucket_width(&self) -> f32 {
        Self::PHASE_SPECTRUM / self.phase_buckets as f32
    }

    fn phase_bucket_middle(&self, bucket: usize) -> f32 {
        self.phase_bucket_width() * (bucket as f32)
    }

    fn phase_find_bucket(&self, phase: f32) -> u64 {
        let width = self.phase_bucket_width();
        (mod_add(phase, width / 2.0) / width) as u64
    }
}

impl Decoder for DifferentialDecoder {
    fn sample(&mut self, point: &FftPoint) -> Option<u64> {
        if !self.is_signal(point) {
            return None
        }

        let mut dphase = 0.0;
        if let Some(phase) = self.last_phase {
            dphase = mod_sub(phase, point.phase);
        }

        self.last_phase = Some(point.phase);

        Some(self.phase_find_bucket(dphase))
    }
}

fn mod_add(a: f32, b: f32) -> f32 {
    (a + b).rem_euclid(DifferentialDecoder::PHASE_SPECTRUM)
}

fn mod_sub(a: f32, b: f32) -> f32 {
    mod_add(a, -b)
}