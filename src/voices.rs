use crate::envelope::{Envelope, EnvelopePosition};
use nih_plug::{
    midi::NoteEvent,
    params::smoothing::{Smoother, SmoothingStyle},
    util,
};
use std::{collections::HashMap, f32::consts};
use typed_floats::tf32::PositiveFinite;

pub struct Voice {
    envelope: EnvelopePosition,
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
        let delta_ms = PositiveFinite::new(100.0 / sample_rate).unwrap();

        // Multiplying the gain here reduces clipping, somehow.
        sine * self.gain.next() * self.envelope.next_sample(delta_ms, envelope)
    }

    pub fn release(&mut self, envelope: &Envelope) {
        self.envelope.release(envelope);
    }

    pub fn completed(&self) -> bool {
        matches!(self.envelope, EnvelopePosition::Completed)
    }
}

pub struct Voices(HashMap<u8, Voice>);

impl Default for Voices {
    fn default() -> Self {
        // allocating...
        Self(HashMap::with_capacity(16))
    }
}

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
                        envelope: EnvelopePosition::from(envelope),
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
            .filter(|(_, voice)| voice.completed())
            .map(|(note, _)| *note)
            .collect::<Vec<u8>>()
            .into_iter()
            .for_each(|note| {
                self.0.remove(&note);
            });
    }
}
