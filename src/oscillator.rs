use std::f32::consts;

use nih_plug::{
    params::smoothing::{Smoother, SmoothingStyle},
    util,
};

/// A stateful oscillator that
#[derive(Default)]
pub struct Sine {
    pub phase: f32,
}

impl Sine {
    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    pub fn next(&mut self, frequency: f32, sample_rate: f32) {
        let phase_delta = frequency / sample_rate;
        self.phase += phase_delta;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
    }

    /// Calculate the current sample at the current position of the sine wave by keeping track of the phase.
    pub fn calculate_sample(&mut self, frequency: f32, sample_rate: f32) -> f32 {
        let sine = (self.phase * consts::TAU).sin();
        self.next(frequency, sample_rate);
        sine
    }
}

pub struct Note {
    pub id: u8,
    pub frequency: f32,
    pub gain: Smoother<f32>,
}

impl Note {
    pub fn reset(&mut self) {
        self.id = 0;
        self.frequency = 1.0;
        self.gain.reset(0.0);
    }
}

impl Default for Note {
    fn default() -> Self {
        Self {
            id: 0,
            frequency: 1.0,
            gain: Smoother::new(SmoothingStyle::Linear(5.0)),
        }
    }
}

#[derive(Default)]
pub struct Oscillator {
    pub note: Note,
    pub sine: Sine,
}

impl Oscillator {
    pub fn reset(&mut self) {
        self.sine.reset();
        self.note.reset();
    }

    pub fn calculate_sample(&mut self, sample_rate: f32) -> f32 {
        self.sine.calculate_sample(self.note.frequency, sample_rate)
            * util::db_to_gain_fast(self.note.gain.next())
    }
}
