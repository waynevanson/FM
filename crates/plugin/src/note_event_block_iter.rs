//! Contains helpers to manage different parts of the buffer processing lifecycle.
//!
//! A buffer can be broken down as follows from largest to smallest:
//! - Block (contains many samples)
//! - Sample
//!
//! In almost all cases these are mutable and take `FnMut` because in the audio
//! thread we're all about performance, therefore reuse a lot of resources.

use fixed_capacity_vec::FixedCapacityVec;
use nih_plug::prelude::{Buffer, Plugin as PluginTrait, PluginNoteEvent, ProcessContext};
use std::marker::PhantomData;

pub type NoteEvents<Plugin> = FixedCapacityVec<Option<PluginNoteEvent<Plugin>>, 64>;

pub struct BufferContext<'a, 'b, Context, Plugin> {
    channels: &'a mut [&'b mut [f32]],
    context: &'a mut Context,
    phantom: PhantomData<Plugin>,
}

impl<'a, 'b, Context, Plugin> BufferContext<'a, 'b, Context, Plugin>
where
    Context: ProcessContext<Plugin>,
    Plugin: PluginTrait,
{
    pub fn new(buffer: &'a mut Buffer<'b>, context: &'a mut Context) -> Self {
        Self {
            channels: buffer.as_slice(),
            context,
            phantom: PhantomData,
        }
    }
}

impl<'a, 'b, Context, Plugin> Iterator for BufferContext<'a, 'b, Context, Plugin>
where
    Context: ProcessContext<Plugin>,
    Plugin: PluginTrait,
{
    type Item = &'a mut Block<'a, 'b, Context, Plugin>;

    fn next(&mut self) -> Option<Self::Item> {
        let note_events = NoteEvents::<Plugin>::new();
        todo!()
    }
}

pub struct Block<'a, 'b, Context, Plugin>
where
    Plugin: PluginTrait,
{
    channels: &'a mut [&'b mut f32],
    context: &'a mut Context,
    note_events: NoteEvents<Plugin>,
}

impl<'a, 'b, Context, Plugin> Iterator for Block<'a, 'b, Context, Plugin>
where
    Plugin: PluginTrait,
{
    type Item = &'a mut Channel<'a, 'b, Context>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

pub struct Channel<'a, 'b, Context> {
    samples: &'b mut [f32],
    context: &'a mut Context,
}

impl<'a, 'b, Context> Iterator for Channel<'a, 'b, Context> {
    type Item = &'a mut [f32];

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
