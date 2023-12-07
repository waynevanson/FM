//! Contains helpers to manage different parts of the buffer processing lifecycle.
//!
//! A buffer can be broken down as follows from largest to smallest:
//! - Block (contains many samples)
//! - Sample
//!
//! In almost all cases these are mutable and take `FnMut` because in the audio
//! thread we're all about performance, therefore reuse a lot of resources.

use nih_plug::{
    buffer::Buffer,
    midi::PluginNoteEvent,
    prelude::{Plugin as PluginTrait, ProcessContext},
};
use std::marker::PhantomData;

/// Just like nih_plug's `BlockIter` but additionally handles MIDI events.
pub trait IterNoteEventBlocks<'context, 'sample> {
    fn iter_note_event_blocks<'slice, Context, Plugin>(
        &'slice mut self,
        context: &'context mut Context,
        max_block_size: usize,
    ) -> NoteEventBlocksIter<'context, 'slice, 'sample, Context, Plugin>
    where
        Context: ProcessContext<Plugin>,
        Plugin: PluginTrait,
        'sample: 'slice;
}

impl<'context, 'sample> IterNoteEventBlocks<'context, 'sample> for Buffer<'sample> {
    fn iter_note_event_blocks<'slice, Context, Plugin>(
        &'slice mut self,
        context: &'context mut Context,
        max_block_size: usize,
    ) -> NoteEventBlocksIter<'context, 'slice, 'sample, Context, Plugin>
    where
        Context: ProcessContext<Plugin>,
        Plugin: PluginTrait,
        'sample: 'slice,
    {
        let current_block_end = max_block_size.min(self.samples());
        NoteEventBlocksIter {
            context,
            buffer: self,
            current_block_end,
            current_block_start: 0,
            previous_event: None,
            phantom: PhantomData,
        }
    }
}

pub struct NoteEventBlocksIter<'context, 'slice, 'sample: 'slice, Context, Plugin>
where
    Context: ProcessContext<Plugin>,
    Plugin: PluginTrait,
{
    context: &'context mut Context,
    // Which lifetime to use here?
    buffer: &'slice mut Buffer<'sample>,
    previous_event: Option<PluginNoteEvent<Plugin>>,
    current_block_start: usize,
    current_block_end: usize,
    phantom: PhantomData<(&'slice mut [&'sample mut [f32]], Plugin)>,
}

impl<'context, 'sample, 'slice, Context, Plugin>
    NoteEventBlocksIter<'context, 'slice, 'sample, Context, Plugin>
where
    Context: ProcessContext<Plugin>,
    Plugin: PluginTrait,
{
    pub fn iter_mut(&mut self) -> NoteEventBlockChannelsIter<'slice, 'sample> {
        NoteEventBlockChannelsIter {
            current_channel: 0,
            buffers: self.buffer.as_slice(),
            current_block_start: self.current_block_start,
            current_block_end: self.current_block_end,
            phantom: PhantomData,
        }
    }
}

impl<'context, 'sample, 'slice, Context, Plugin> Iterator
    for NoteEventBlocksIter<'context, 'slice, 'sample, Context, Plugin>
where
    Context: ProcessContext<Plugin>,
    Plugin: PluginTrait,
{
    type Item = (usize, Block<'slice, 'sample, Plugin>);

    fn next(&mut self) -> Option<Self::Item> {
        // We actually want to process the current event, not the previous.
        self.previous_event = self.context.next_event();

        // There are no more blocks to process.
        if self.current_block_start < self.buffer.samples() {
            return None;
        }

        // this is now the current event
        match &self.previous_event {
            // Event has happened before the start of the block,
            // so we can consume it.
            Some(event) if (event.timing() as usize) <= self.current_block_start => Some((
                self.current_block_start,
                Block {
                    // TODO - Is the refactor effort worth borrowing this?
                    note_event: Some(event.to_owned()),
                    phantom: PhantomData,
                },
            )),

            // We haven't run out of blocks, we want to start a new one instead.
            Some(event) if (event.timing() as usize) < self.current_block_end => {
                self.current_block_end = event.timing() as usize;
                // TODO - Can we not have this be recursive on the stack?
                // Surely we won't repeat this for long...
                self.next()
            }

            // Stop iterating events.
            _ => None,
        }
    }
}

pub struct Block<'slice, 'sample, Plugin>
where
    Plugin: PluginTrait,
{
    pub note_event: Option<PluginNoteEvent<Plugin>>,
    phantom: PhantomData<&'slice mut [&'sample mut f32]>,
}

pub struct NoteEventBlockChannelsIter<'slice, 'sample: 'slice> {
    current_channel: usize,
    buffers: *mut [&'sample mut [f32]],
    current_block_start: usize,
    current_block_end: usize,
    phantom: PhantomData<&'slice mut [&'sample mut [f32]]>,
}

impl<'slice, 'sample> Iterator for NoteEventBlockChannelsIter<'slice, 'sample> {
    type Item = &'sample mut [f32];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_channel < unsafe { (*self.buffers).len() } {
            // SAFETY: These bounds have already been checked
            // SAFETY: It is also not possible to have multiple mutable references to the same
            //         sample at the same time
            let slice = unsafe {
                (*self.buffers)
                    .get_unchecked_mut(self.current_channel)
                    .get_unchecked_mut(self.current_block_start..self.current_block_end)
            };

            self.current_channel += 1;

            Some(slice)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = unsafe { (*self.buffers).len() } - self.current_channel;

        (remaining, Some(remaining))
    }
}
