use std::time::Duration;

use super::FrequencyComponent;

#[derive(Debug, Copy, Clone)]
enum Operation {
    AddWaveform(FrequencyComponent),
    ClearWaveform,
}

#[derive(Debug, Copy, Clone)]
struct Command {
    timestamp: Duration,
    operation: Operation,
}
