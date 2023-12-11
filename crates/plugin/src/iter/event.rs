use nih_plug::{
    context::process::ProcessContext, midi::PluginNoteEvent, plugin::Plugin as PluginTrait,
};
use std::marker::PhantomData;

pub struct EventsIntoIter<'process, Context, Plugin>
where
    Context: ProcessContext<Plugin>,
    Plugin: PluginTrait,
{
    context: &'process mut Context,
    _phantom: PhantomData<Plugin>,
}

impl<'process, Context, Plugin> EventsIntoIter<'process, Context, Plugin>
where
    Context: ProcessContext<Plugin>,
    Plugin: PluginTrait,
{
    pub fn new(context: &'process mut Context) -> Self {
        Self {
            context,
            _phantom: PhantomData,
        }
    }
}

impl<'process, Context, Plugin> Iterator for EventsIntoIter<'process, Context, Plugin>
where
    Context: ProcessContext<Plugin>,
    Plugin: PluginTrait,
{
    type Item = PluginNoteEvent<Plugin>;

    fn next(&mut self) -> Option<Self::Item> {
        self.context.next_event()
    }
}

pub trait IntoEventsIntoIter<'process, Context, Plugin>
where
    Context: ProcessContext<Plugin>,
    Plugin: PluginTrait,
{
    fn events_into_iter(self) -> EventsIntoIter<'process, Context, Plugin>;
}

impl<'process, Context, Plugin> IntoEventsIntoIter<'process, Context, Plugin>
    for &'process mut Context
where
    Context: ProcessContext<Plugin>,
    Plugin: PluginTrait,
{
    fn events_into_iter(self) -> EventsIntoIter<'process, Context, Plugin> {
        EventsIntoIter::new(self)
    }
}
