mod voices;

use nih_plug::prelude::*;
use std::{num::NonZeroU32, sync::Arc};
use voices::Voices;

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
    voices: Voices,
}

impl Default for FmSynth {
    fn default() -> Self {
        FmSynth {
            params: Arc::new(FmSynthParams::default()),
            sample_rate: 1.0,
            voices: Voices::default(),
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
        self.voices = Voices::default();
    }

    fn process(
        &mut self,
        buffer: &mut nih_plug::prelude::Buffer,
        _aux: &mut nih_plug::prelude::AuxiliaryBuffers,
        context: &mut impl nih_plug::prelude::ProcessContext<Self>,
    ) -> nih_plug::prelude::ProcessStatus {
        let mut next_event = context.next_event();
        for (sample_id, channel_samples) in buffer.iter_samples().enumerate() {
            // Save the data from MIDI events that we need.
            while let Some(note_event) = next_event {
                if note_event.timing() > sample_id as u32 {
                    break;
                }

                self.voices.from_note_event(note_event, self.sample_rate);

                next_event = context.next_event();
            }

            let sine = self.voices.calculate_sines(self.sample_rate);
            let gain = self.params.gain.smoothed.next();

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
