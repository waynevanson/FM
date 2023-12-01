use std::sync::Arc;

use nih_plug::prelude::{FloatParam, FloatRange, Params, SmoothingStyle};

use crate::envelope::Envelope;

#[derive(Params)]
pub struct FmSynthParams {
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "attack"]
    pub attack: FloatParam,
    #[id = "decay"]
    pub decay: FloatParam,
    #[id = "sustain"]
    pub sustain: FloatParam,
    #[id = "release"]
    pub release: FloatParam,
}

impl Default for FmSynthParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                -10.0,
                FloatRange::Linear {
                    min: -30.0,
                    max: 0.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(3.0))
            .with_step_size(0.01)
            .with_unit(" dB"),
            attack: FloatParam::new(
                "Attack",
                0.0,
                FloatRange::Skewed {
                    min: 0.0,
                    max: 1000.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_smoother(SmoothingStyle::Linear(5.0))
            .with_step_size(1.0)
            .with_unit(" Milliseconds"),
            decay: FloatParam::new(
                "Decay",
                40.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 5000.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(5.0))
            .with_step_size(1.0)
            .with_unit(" Milliseconds"),
            sustain: FloatParam::new("Sustain", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(5.0))
                .with_step_size(0.01)
                .with_unit("%"),
            release: FloatParam::new(
                "Release",
                40.0,
                FloatRange::Skewed {
                    min: 0.0,
                    max: 5000.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_smoother(SmoothingStyle::Linear(5.0))
            .with_step_size(1.0)
            .with_unit(" Milliseconds"),
        }
    }
}

pub struct FmSynthParamsSample {
    pub gain: f32,
    pub envelope: Envelope,
}

impl From<&Arc<FmSynthParams>> for FmSynthParamsSample {
    fn from(params: &Arc<FmSynthParams>) -> Self {
        Self {
            gain: params.gain.smoothed.next(),
            envelope: Envelope {
                attack: params.attack.smoothed.next(),
                decay: params.decay.smoothed.next(),
                sustain: params.sustain.smoothed.next(),
                release: params.release.smoothed.next(),
            },
        }
    }
}
