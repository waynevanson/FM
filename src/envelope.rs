use crate::{utility::scale_linearly, zero_to_one_float_32::ZeroToOneFloat32};
use nih_plug::nih_log;
use typed_floats::tf32::PositiveFinite;

type Factor = ZeroToOneFloat32;

/// The parameters used to calculate the envelope of a sound.
/// These could change on every sample by the user.
#[derive(Debug)]
pub struct Envelope {
    pub attack: PositiveFinite,
    pub decay: PositiveFinite,
    pub sustain: ZeroToOneFloat32,
    pub release: PositiveFinite,
}

#[derive(PartialEq, Eq, Debug)]
pub enum EnvelopePosition {
    Attack {
        factor: Factor,
        ms: PositiveFinite,
    },
    Decay {
        upper: Option<Factor>,
        ms: PositiveFinite,
    },
    Sustain,
    Release {
        upper: Option<Factor>,
        ms: PositiveFinite,
    },
    Completed,
}

impl EnvelopePosition {
    fn factor(&self, envelope: &Envelope) -> Factor {
        match self {
            EnvelopePosition::Completed => Factor::new(0.0).unwrap(),
            EnvelopePosition::Sustain => envelope.sustain,
            EnvelopePosition::Release { upper, ms } => {
                let source_start = envelope.release.get();
                let source = ms.get();

                let target_start: f32 = upper.unwrap_or(envelope.sustain).into();

                let target = scale_linearly(source, source_start..=0.0, target_start..=0.0);
                Factor::new(target).unwrap_or_else(|_| Factor::new(0.0).unwrap())
            }
            EnvelopePosition::Decay { upper, ms } => {
                let source_start = envelope.decay.get();
                let source = ms.get();

                let target_start: f32 = upper.unwrap_or(Factor::new(1.0).unwrap()).into();
                let target_end = envelope.sustain.into();

                let target = scale_linearly(source, source_start..=0.0, target_start..=target_end);
                Factor::new(target).unwrap_or_else(|_| Factor::new(0.0).unwrap())
            }
            EnvelopePosition::Attack { factor, .. } => *factor,
        }
    }

    pub fn release(&mut self, envelope: &Envelope) -> Option<()> {
        let upper = match self {
            Self::Sustain => Some(None),
            Self::Decay { .. } => Some(Some(self.factor(envelope))),
            Self::Attack { factor, .. } => Some(Some(*factor)),
            _ => None,
        }?;

        *self = Self::Release {
            upper,
            ms: envelope.release,
        };

        Some(())
    }

    fn increment(&mut self, delta_ms: PositiveFinite, envelope: &Envelope) {
        match self {
            Self::Release { ms, .. } => {
                let next_ms: f32 = (*ms - delta_ms).into();
                if next_ms <= 0.0 {
                    *self = Self::Completed
                } else {
                    *ms = PositiveFinite::new(next_ms).unwrap();
                }
            }
            Self::Decay { ms, .. } => {
                let next_ms: f32 = (*ms - delta_ms).into();
                if next_ms <= 0.0 {
                    *self = Self::Sustain
                } else {
                    *ms = PositiveFinite::new(next_ms).unwrap()
                }
            }
            Self::Attack { factor, ms } => {
                let next_ms = (*ms - delta_ms).get();
                if next_ms <= 0.0 {
                    *ms = PositiveFinite::new(next_ms).unwrap();
                    return;
                }

                let decay = envelope.decay.get() - -next_ms;

                *self = if decay <= 0.0 {
                    Self::Decay {
                        upper: Some(*factor),
                        ms: *ms,
                    }
                } else {
                    Self::Sustain
                }
            }
            _ => (),
        }
    }

    pub fn next_sample(&mut self, delta_ms: PositiveFinite, envelope: &Envelope) -> f32 {
        nih_log!("{:?}", self);
        let factor = self.factor(envelope);
        self.increment(delta_ms, envelope);
        factor.into()
    }
}

impl From<&Envelope> for EnvelopePosition {
    fn from(envelope: &Envelope) -> Self {
        let attack = envelope.attack.get();
        let decay = envelope.decay.get();

        // match statement for float deprecated
        if attack <= 0.0 && decay <= 0.0 {
            Self::Sustain
        } else if attack <= 0.0 {
            Self::Decay {
                upper: None,
                ms: envelope.decay,
            }
        } else {
            Self::Attack {
                factor: Factor::new(0.0).unwrap(),
                ms: envelope.attack,
            }
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn release_should() {
        let envelope = Envelope {
            attack: PositiveFinite::new(0.0).unwrap(),
            decay: PositiveFinite::new(0.0).unwrap(),
            sustain: Factor::new(0.0).unwrap(),
            release: PositiveFinite::new(0.0).unwrap(),
        };
    }
}
