use crate::{envelope::Envelope, utility::scale_float};
use nih_plug::{
    midi::NoteEvent,
    params::smoothing::{Smoother, SmoothingStyle},
    util,
};
use std::{collections::HashMap, f32::consts};

pub enum EnvelopeRemaining {
    // >= 0
    Attack(f32),
    // >= 0
    Decay(f32),
    Sustain,
    // >= 0
    Release(f32),
    Completed,
}

impl EnvelopeRemaining {
    pub fn completed(&self) -> bool {
        matches!(self, Self::Completed)
    }

    fn factor(&mut self, envelope: &Envelope) -> f32 {
        match *self {
            // [0, 1]
            Self::Attack(ms) => {
                if ms > 0.0 {
                    scale_float(ms, envelope.attack..=0.0, 0.0..=1.0)
                } else {
                    1.0
                }
            }
            // [1, sustain]
            Self::Decay(ms) => {
                if ms > 0.0 {
                    scale_float(ms, envelope.decay..=0.0, 1.0..=envelope.sustain)
                } else {
                    envelope.sustain
                }
            }
            Self::Sustain => envelope.sustain,
            // [sustain, 0]
            Self::Release(ms) => {
                if ms > 0.0 {
                    scale_float(ms, envelope.release..=0.0, envelope.sustain..=0.0)
                } else {
                    0.0
                }
            }
            Self::Completed => 0.0,
        }
    }

    fn next(&mut self, delta_ms: f32) {
        match self {
            Self::Attack(ms) => {
                *ms -= delta_ms;
                if ms <= &mut 0.0 {
                    *self = Self::Decay(-*ms)
                }
            }
            Self::Decay(ms) => {
                *ms -= delta_ms;
                if ms <= &mut 0.0 {
                    *self = Self::Sustain
                }
            }
            Self::Release(ms) => {
                *ms -= delta_ms;
                if ms <= &mut 0.0 {
                    *self = Self::Completed
                }
            }
            _ => (),
        }
    }

    pub fn release(&mut self, envelope: &Envelope) {
        *self = Self::Release(envelope.release)
    }

    /// Calculate the next sample. Should be called exactly once per sample.
    pub fn next_factor(&mut self, sample_rate: f32, envelope: &Envelope) -> f32 {
        let factor = self.factor(envelope);
        let delta = 1000.0 / sample_rate;
        self.next(delta);
        factor
    }
}

pub struct Voice {
    envelope_duration: EnvelopeRemaining,
    phase: f32,
    frequency: f32,
    gain: Smoother<f32>,
}

impl Voice {
    fn next_phase(&mut self, sample_rate: f32) -> f32 {
        let phase_delta = self.frequency / sample_rate;
        self.phase += phase_delta;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        self.phase
    }

    pub fn next_sine(&mut self, sample_rate: f32, envelope: &Envelope) -> f32 {
        let sine = (self.phase * consts::TAU).sin();
        self.next_phase(sample_rate);
        // Multiplying the gain here reduces clipping, somehow.
        sine * self.gain.next() * self.envelope_duration.next_factor(sample_rate, envelope)
    }

    pub fn release(&mut self, envelope: &Envelope) {
        self.envelope_duration.release(envelope)
    }
}

#[derive(Default)]
pub struct Voices(HashMap<u8, Voice>);

impl Voices {
    pub fn from_note_event(
        &mut self,
        note_event: NoteEvent<()>,
        sample_rate: f32,
        envelope: &Envelope,
    ) {
        match note_event {
            NoteEvent::NoteOn { note, velocity, .. } => {
                // todo - handle note that exists but is releasing
                if !self.0.contains_key(&note) {
                    let gain = Smoother::new(SmoothingStyle::Linear(5.0));
                    gain.set_target(sample_rate, velocity);

                    let voice = Voice {
                        envelope_duration: EnvelopeRemaining::Attack(envelope.attack),
                        frequency: util::midi_note_to_freq(note),
                        gain,
                        phase: 0.0,
                    };
                    self.0.insert(note, voice);
                }
            }
            // how to remove a note?
            NoteEvent::NoteOff { note, .. } => {
                let voice = self.0.get_mut(&note);
                if let Some(voice) = voice {
                    voice.release(&envelope)
                };
            }
            NoteEvent::PolyPressure { note, pressure, .. } => {
                let voice = self.0.get_mut(&note);
                if let Some(voice) = voice {
                    voice.gain.set_target(sample_rate, pressure);
                };
            }
            _ => (),
        };
    }

    pub fn calculate_sines(&mut self, sample_rate: f32, envelope: &Envelope) -> f32 {
        self.0
            .values_mut()
            .map(|voice| voice.next_sine(sample_rate, envelope))
            .fold(0.0, |first, second| first + second)
    }

    pub fn cleanup_voices(&mut self) {
        self.0
            .iter()
            .filter(|(_, voice)| voice.envelope_duration.completed())
            .map(|(note, _)| *note)
            .collect::<Vec<u8>>()
            .into_iter()
            .for_each(|note| {
                self.0.remove(&note);
            });
    }
}
