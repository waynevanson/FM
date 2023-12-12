use crate::params::FmSynthParams;
use nih_plug::{context::gui::GuiContext, params::smoothing::AtomicF32, util};
use nih_plug_iced::*;
use std::{sync::Arc, time::Duration};

pub struct FmSynthEditorValues {
    pub peak_meter: AtomicF32,
}

impl Default for FmSynthEditorValues {
    fn default() -> Self {
        Self {
            peak_meter: AtomicF32::new(util::MINUS_INFINITY_DB),
        }
    }
}

#[derive(Default)]
pub struct FmSynthEditorState {
    pub peak_meter: widgets::peak_meter::State,
}

pub struct FmSynthEditor {
    params: Arc<FmSynthParams>,
    values: Arc<FmSynthEditorValues>,
    state: FmSynthEditorState,
    context: Arc<dyn GuiContext>,
}

impl IcedEditor for FmSynthEditor {
    type Executor = executor::Default;
    type Message = ();
    type InitializationFlags = (Arc<FmSynthParams>, Arc<FmSynthEditorValues>);

    fn new(
        (params, values): Self::InitializationFlags,
        context: Arc<dyn GuiContext>,
    ) -> (Self, Command<Self::Message>) {
        let editor = Self {
            context,
            params,
            values,
            state: FmSynthEditorState::default(),
        };

        (editor, Command::none())
    }

    fn context(&self) -> &dyn GuiContext {
        self.context.as_ref()
    }

    fn view(&mut self) -> nih_plug_iced::Element<'_, Self::Message> {
        Column::new()
            .align_items(Alignment::Center)
            .push(
                Text::new("Gain GUI")
                    .font(assets::NOTO_SANS_LIGHT)
                    .size(40)
                    .height(50.into())
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .vertical_alignment(alignment::Vertical::Bottom),
            )
            .push(
                Text::new("Gain")
                    .height(20.into())
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .vertical_alignment(alignment::Vertical::Center),
            )
            // .push(
            //     widgets::ParamSlider::new(&mut self.gain_slider_state, &self.params.gain)
            //         .map(Message::ParamUpdate),
            // )
            .push(Space::with_height(10.into()))
            .push(
                widgets::PeakMeter::new(
                    &mut self.state.peak_meter,
                    util::gain_to_db(
                        self.values
                            .peak_meter
                            .load(std::sync::atomic::Ordering::Relaxed),
                    ),
                )
                .hold_time(Duration::from_millis(600)),
            )
            .into()
    }

    fn update(
        &mut self,
        window: &mut nih_plug_iced::WindowQueue,
        message: Self::Message,
    ) -> Command<Self::Message> {
        Command::none()
    }

    fn background_color(&self) -> Color {
        Color {
            r: 0.98,
            g: 0.98,
            b: 0.98,
            a: 1.0,
        }
    }
}
