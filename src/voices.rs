use crate::envelope::Envelope;
use nih_plug::{
    midi::NoteEvent,
    params::smoothing::{Smoother, SmoothingStyle},
    util,
};
use std::{collections::HashMap, f32::consts};

pub struct Voice {
    /// Milliseconds elapsed since the voice was created
    duration: f32,
    phase: f32,
    frequency: f32,
    gain: Smoother<f32>,
}

impl Voice {
    fn next_envelope_factor(&mut self, sample_rate: f32, envelope: &Envelope) -> f32 {
        let _factor = if 0.0 < self.duration && self.duration < envelope.attack {
            // milliseconds
            let complement = self.duration / envelope.attack;
            complement
        } else {
            1.0
        };
        let delta = 1000.0 / sample_rate;
        self.duration += delta;
        _factor
    }

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
        sine * self.gain.next() * self.next_envelope_factor(sample_rate, envelope)
    }
}

#[derive(Default)]
pub struct Voices(HashMap<u8, Voice>);

impl Voices {
    pub fn from_note_event(&mut self, note_event: NoteEvent<()>, sample_rate: f32) {
        match note_event {
            NoteEvent::NoteOn { note, velocity, .. } => {
                let gain = Smoother::new(SmoothingStyle::Linear(5.0));
                gain.set_target(sample_rate, velocity);

                if !self.0.contains_key(&note) {
                    let voice = Voice {
                        duration: 0.0,
                        frequency: util::midi_note_to_freq(note),
                        gain,
                        phase: 0.0,
                    };
                    self.0.insert(note, voice);
                }
            }
            NoteEvent::NoteOff { note, .. } => {
                self.0.remove(&note);
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
}
