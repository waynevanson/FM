use crate::params::FmSynthParams;
use nih_plug::context::gui::GuiContext;
use nih_plug_iced::*;
use std::sync::Arc;

struct FmSynthEditorState {}

pub struct FmSynthEditor {
    params: Arc<FmSynthParams>,
    state: Arc<FmSynthEditorState>,
    context: Arc<dyn GuiContext>,
}

impl IcedEditor for FmSynthEditor {
    type Executor = executor::Default;
    type Message = ();
    type InitializationFlags = Arc<FmSynthParams>;

    fn new(
        initialization_fags: Self::InitializationFlags,
        context: Arc<dyn GuiContext>,
    ) -> (Self, Command<Self::Message>) {
        (
            Self {
                context,
                params: initialization_fags,
                state: Arc::new(FmSynthEditorState {}),
            },
            Command::none(),
        )
    }

    fn context(&self) -> &dyn GuiContext {
        self.context.as_ref()
    }

    fn view(&mut self) -> nih_plug_iced::Element<'_, Self::Message> {
        Column::new()
            .align_items(Alignment::Center)
            .push(Text::new("Hello, World!"))
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
