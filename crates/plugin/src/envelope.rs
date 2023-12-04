use std::fmt::Debug;

use nih_plug::prelude::*;

#[derive(Debug, Clone, Copy)]
enum Phase {
    Attack,
    Decay,
    Release,
}

#[derive(Debug, Clone, Default)]
pub struct Envelope<T>
where
    T: Smoothable + Debug + Clone,
    T::Atomic: Default + Debug,
{
    gain: Smoother<T>,
    phase: Option<Phase>,
}

impl<T> Envelope<T>
where
    T: Smoothable + Debug + PartialEq,
    T::Atomic: Default + Debug,
{
    pub fn note_on(&mut self, sample_rate: f32, attack: f32) {
        self.gain = Smoother::new(SmoothingStyle::Exponential(attack));
        self.gain.reset(T::from_f32(0.0));
        self.gain.set_target(sample_rate, T::from_f32(1.0));

        self.phase = Some(Phase::Attack);
    }

    pub fn note_off(&mut self, sample_rate: f32, release: f32) {
        self.phase = Some(Phase::Release);
        self.gain.style = SmoothingStyle::Exponential(release);
        self.gain.set_target(sample_rate, T::from_f32(0.0));
    }

    pub fn is_released(&self) -> bool {
        matches!(self.phase, Some(Phase::Release)) && self.gain.previous_value() == T::from_f32(0.0)
    }

    pub fn next_decay(&mut self, sample_rate: f32, decay: f32, sustain: T) {
        let is_max = self.gain.previous_value() == T::from_f32(1.0);
        let is_attack = matches!(self.phase, Some(Phase::Attack));

        if is_max && is_attack {
            self.phase = Some(Phase::Decay);
            self.gain.style = SmoothingStyle::Exponential(decay);
            self.gain.set_target(sample_rate, sustain);
        }
    }

    /// Produce smoothed values for an entire block of audio. This is useful when iterating the same
    /// block of audio multiple times. For instance when summing voices for a synthesizer.
    /// `block_values[..block_len]` will be filled with the smoothed values. This is simply a
    /// convenient function for [`next_block_exact()`][Self::next_block_exact()] when iterating over
    /// variable length blocks with a known maximum size.
    ///
    /// # Panics
    ///
    /// Panics if `block_len > block_values.len()`.
    pub fn next_block(&mut self, block_values: &mut [T], block_len: usize) {
        self.gain.next_block(block_values, block_len)
    }
}
