use std::time::Duration;

use super::soundgen::FrequencyComponent;

#[derive(Debug, Copy, Clone)]
pub enum Operation {
    AddWaveform(FrequencyComponent),
    ClearWaveform,
}

#[derive(Debug, Copy, Clone)]
pub struct Command {
    timestamp: Duration,
    operation: Operation,
}
