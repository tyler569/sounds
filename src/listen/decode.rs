use crate::listen::FftPoint;

pub struct Decoder {
    phase_buckets: usize,
    phase_offset: Option<f32>,

    amplitude_buckets: usize,
    amplitude_offset: Option<f32>,

    last_samples: Vec<FftPoint>,
}

impl Decoder {
    const PHASE_SPECTRUM: f32 = 1.0;

    pub fn new() -> Self {
        Self {
            phase_buckets: 4,
            phase_offset: None,
            amplitude_buckets: 1,
            amplitude_offset: None,
            last_samples: vec![],
        }
    }

    /// Take a point in the FFT spectruc corresponding to a particular
    /// frequency and analyze it to find possible sent data.
    pub fn sample(&mut self, point: &FftPoint) -> Option<u64> {
        if point.amplitude < 4.0 {
            return None
        }

        if self.phase_offset.is_none() {
            self.phase_offset = Some(mod_sub(1.0, point.phase));
        }

        let phase = self.offset_phase(point.phase);
        let bucket = self.phase_find_bucket(phase);
        // eprintln!("point {} offset {:?} phase {} bucket {}", point.phase, self.phase_offset, phase, bucket);
        self.adjust_phase_offset(phase, bucket);
        Some(bucket as u64)
    }

    /// Getter for phase_offset so I can visualize it outside this module
    pub fn phase_offset(&self) -> Option<f32> {
        self.phase_offset
    }

    /// Adjust FFT phase to be relative to the local baseline
    fn offset_phase(&self, phase: f32) -> f32 {
        mod_add(phase, self.phase_offset.expect("This is always set in `sample`"))
    }

    /// Adjust local phase baseline to account for drift over time
    /// 
    /// Takes an observed phase and a guessed bucket, adjusts the local
    /// `phase_offset` such that `self.offset_phase(phase)` would equal
    /// `self.phase_bucket_middle(bucket)`
    fn adjust_phase_offset(&mut self, phase: f32, bucket: usize) {
        let target = self.phase_bucket_middle(bucket);
        let diff = mod_sub(phase, target);
        assert!(diff < self.phase_bucket_width() / 2.0 ||
            diff > 1.0 - self.phase_bucket_width() / 2.0);

        self.phase_offset = Some(
            mod_sub(self.phase_offset.expect("Set in `sample`"), diff));

    }

    fn phase_bucket_width(&self) -> f32 {
        Self::PHASE_SPECTRUM / self.phase_buckets as f32
    }

    fn phase_bucket_middle(&self, bucket: usize) -> f32 {
        self.phase_bucket_width() * (bucket as f32)
    }

    fn phase_find_bucket(&self, phase: f32) -> usize {
        let width = self.phase_bucket_width();
        (mod_add(phase, width / 2.0) / width) as usize
    }
}

fn mod_add(a: f32, b: f32) -> f32 {
    (a + b).rem_euclid(Decoder::PHASE_SPECTRUM)
}

fn mod_sub(a: f32, b: f32) -> f32 {
    mod_add(a, -b)
}