//! Contains helpers to manage different parts of the buffer processing lifecycle.
//!
//! A buffer can be broken down as follows from largest to smallest:
//! - Block (contains many samples)
//! - Sample
//!
//! In almost all cases these are mutable and take `FnMut` because in the audio
//! thread we're all about performance, therefore reuse a lot of resources.

use nih_plug::{
    buffer::{self, Buffer},
    midi::{NoteEvent, PluginNoteEvent},
    prelude::{Plugin as PluginTrait, ProcessContext},
};
use std::marker::PhantomData;

use crate::note_event_sample_iter::NoteEventSampleIter;

/// Just like nih_plug's `BlockIter` but additionally handles MIDI events.
pub struct NoteEventBlockIter<'context, 'buffer, 'plugin, Context, Plugin>
where
    Plugin: PluginTrait,
{
    next_event: Option<PluginNoteEvent<Plugin>>,
    block_start: usize,
    block_end: usize,
    buffer: &'buffer mut Buffer<'plugin>,
    context: &'context mut Context,
    phantom: PhantomData<Plugin>,
}

impl<'plugin, 'context, 'buffer, Context, Plugin>
    NoteEventBlockIter<'plugin, 'context, 'buffer, Context, Plugin>
where
    Context: ProcessContext<Plugin>,
    Plugin: PluginTrait,
    'context: 'plugin,
{
    pub fn new(
        context: &'context mut Context,
        buffer: &'buffer mut Buffer<'plugin>,
        max_block_size: usize,
    ) -> Self {
        let num_samples = buffer.samples();
        Self {
            next_event: None,
            context,
            block_start: 0,
            buffer,
            block_end: max_block_size.min(num_samples),
            phantom: PhantomData,
        }
    }
}

impl<'plugin, 'context, 'buffer, Context, Plugin> Iterator
    for NoteEventBlockIter<'plugin, 'context, 'buffer, Context, Plugin>
where
    Context: ProcessContext<Plugin>,
    Plugin: PluginTrait,
{
    type Item = (PluginNoteEvent<Plugin>, NoteEventSampleIter);

    fn next(&mut self) -> Option<Self::Item> {
        // Always `None` when created, and we need `next_event` to maybe have a value.
        if let None = self.next_event {
            self.next_event = self.context.next_event();
        }

        // There are no more blocks to process.
        if self.block_start < self.buffer.samples() {
            return None;
        }

        match &self.next_event {
            // Event has happened before the start of the block,
            // so we can consume it.
            Some(event) if (event.timing() as usize) <= self.block_start => {
                let event = event.to_owned();
                self.next_event = self.context.next_event();
                Some((event, NoteEventSampleIter {}))
            }

            // Stop iterating events in the current block by cutting the block short.
            Some(event) if (event.timing() as usize) < self.block_end => {
                self.block_end = event.timing() as usize;
                None
            }

            // Stop iterating events.
            _ => None,
        }
    }
}
