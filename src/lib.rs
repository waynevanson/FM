use nih_plug::prelude::*;
use std::{f32::consts, num::NonZeroU32, sync::Arc};

#[derive(Params)]
struct FmSynthParams {
    #[id = "gain"]
    pub gain: FloatParam,
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
        }
    }
}

struct FmSynth {
    params: Arc<FmSynthParams>,
    sample_rate: f32,
    phase: f32,

    midi_note_id: u8,
    midi_note_freq: f32,
    midi_note_gain: Smoother<f32>,
}

impl FmSynth {
    fn calculate_sine(&mut self, frequency: f32) -> f32 {
        let phase_delta = frequency / self.sample_rate;
        let sine = (self.phase * consts::TAU).sin();

        self.phase += phase_delta;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        sine
    }
}

impl Default for FmSynth {
    fn default() -> Self {
        FmSynth {
            params: Arc::new(FmSynthParams::default()),
            sample_rate: 1.0,

            phase: 0.0,

            midi_note_id: 0,
            midi_note_freq: 1.0,
            midi_note_gain: Smoother::new(SmoothingStyle::Linear(5.0)),
        }
    }
}

impl Plugin for FmSynth {
    type SysExMessage = ();
    type BackgroundTask = ();

    const NAME: &'static str = "FM";
    const VENDOR: &'static str = "Wayne Van Son";
    const URL: &'static str = "https://github.com/waynevanson/fm";
    const EMAIL: &'static str = "waynevanson@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [nih_plug::prelude::AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: nih_plug::prelude::MidiConfig = MidiConfig::Basic;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // default
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;
    // default
    const HARD_REALTIME_ONLY: bool = false;

    fn params(&self) -> std::sync::Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;

        true
    }

    fn reset(&mut self) {
        self.phase = 0.0;
        self.midi_note_id = 0;
        self.midi_note_freq = 1.0;
        self.midi_note_gain.reset(0.0);
    }

    fn process(
        &mut self,
        buffer: &mut nih_plug::prelude::Buffer,
        _aux: &mut nih_plug::prelude::AuxiliaryBuffers,
        context: &mut impl nih_plug::prelude::ProcessContext<Self>,
    ) -> nih_plug::prelude::ProcessStatus {
        let mut next_event = context.next_event();

        for (sample_id, channel_samples) in buffer.iter_samples().enumerate() {
            let gain = self.params.gain.smoothed.next();

            let sine = {
                while let Some(event) = next_event {
                    if event.timing() > sample_id as u32 {
                        break;
                    }

                    match event {
                        NoteEvent::NoteOn { note, velocity, .. } => {
                            self.midi_note_id = note;
                            self.midi_note_freq = util::midi_note_to_freq(note);
                            self.midi_note_gain.set_target(self.sample_rate, velocity);
                        }
                        NoteEvent::NoteOff { note, .. } => {
                            if note == self.midi_note_id {
                                self.midi_note_gain.set_target(self.sample_rate, 0.0);
                            }
                        }
                        NoteEvent::PolyPressure { note, pressure, .. } => {
                            if note == self.midi_note_id {
                                self.midi_note_gain.set_target(self.sample_rate, pressure);
                            }
                        }
                        _ => (),
                    };

                    next_event = context.next_event();
                }

                self.calculate_sine(self.midi_note_freq) * self.midi_note_gain.next()
            };

            for sample in channel_samples {
                *sample = sine * util::db_to_gain_fast(gain)
            }
        }

        ProcessStatus::KeepAlive
    }
}

impl ClapPlugin for FmSynth {
    const CLAP_FEATURES: &'static [nih_plug::prelude::ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Custom("FM"),
        ClapFeature::Custom("Frequency Modulation"),
    ];
    const CLAP_ID: &'static str = "com.waynevanson.fm";
    const CLAP_DESCRIPTION: Option<&'static str> = None;
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_POLY_MODULATION_CONFIG: Option<nih_plug::prelude::PolyModulationConfig> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
}

impl Vst3Plugin for FmSynth {
    const VST3_CLASS_ID: [u8; 16] = *b"waynevansonfm...";
    const VST3_SUBCATEGORIES: &'static [nih_plug::prelude::Vst3SubCategory] = &[
        Vst3SubCategory::Synth,
        Vst3SubCategory::Instrument,
        Vst3SubCategory::Custom("FM"),
        Vst3SubCategory::Custom("Frequency Modulation"),
    ];
}

nih_export_clap!(FmSynth);
nih_export_vst3!(FmSynth);
