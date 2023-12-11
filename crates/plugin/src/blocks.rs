use crate::event::{EventsIntoIter, IntoEventsIntoIter};
use nih_plug::{
    buffer::Buffer, context::process::ProcessContext, midi::PluginNoteEvent,
    plugin::Plugin as PluginTrait,
};
use std::{iter::FusedIterator, marker::PhantomData, slice::from_raw_parts_mut};

// optional event goes in, block comes out.
pub struct BlocksIterMut<'process, 'samples, Context, Plugin>
where
    Context: ProcessContext<Plugin>,
    Plugin: PluginTrait,
{
    events_iter: EventsIntoIter<'process, Context, Plugin>,
    _phantom: PhantomData<&'samples mut ()>,
    block_start: usize,
    block_end: usize,
    previous_event: Option<PluginNoteEvent<Plugin>>,
    buffer_channels: &'process mut [&'samples mut [f32]],
    max_block_size: usize,
}

impl<'process, 'samples, Context, Plugin> BlocksIterMut<'process, 'samples, Context, Plugin>
where
    Context: ProcessContext<Plugin>,
    Plugin: PluginTrait,
{
    pub fn new(
        buffer: &'process mut Buffer<'samples>,
        context: &'process mut Context,
        max_block_size: usize,
    ) -> Self {
        let block_end = buffer.samples().min(max_block_size);
        let buffer_channels = buffer.as_slice();
        let block_start = 0;

        Self {
            events_iter: context.events_into_iter(),
            _phantom: PhantomData,
            block_start,
            block_end,
            previous_event: None,
            buffer_channels,
            max_block_size,
        }
    }

    pub unsafe fn block_mut(&mut self) -> &'process mut [&'samples mut [f32]] {
        let buffers_pointer = self.buffer_channels.as_mut_ptr();
        let buffers_length = self.buffer_channels.len();
        let channels = unsafe { from_raw_parts_mut(buffers_pointer, buffers_length) };

        for index in 0..buffers_length {
            let block_length = self.block_end - self.block_start;
            let channel = &mut channels[index];
            let channel_pointer = channel.as_mut_ptr();
            let block =
                unsafe { from_raw_parts_mut(channel_pointer.add(self.block_start), block_length) };

            *channel = block;
        }

        channels
    }
}

pub enum EventBlock<'process, 'samples, Plugin>
where
    Plugin: PluginTrait,
{
    Event(PluginNoteEvent<Plugin>),
    Block(&'process mut [&'samples mut [f32]]),
}

impl<'process, 'samples, Context, Plugin> Iterator
    for BlocksIterMut<'process, 'samples, Context, Plugin>
where
    Context: ProcessContext<Plugin>,
    Plugin: PluginTrait,
{
    type Item = EventBlock<'process, 'samples, Plugin>;

    fn next(&mut self) -> Option<Self::Item> {
        // Use the previous event, or we'll try get the next one.
        let event = self
            .previous_event
            .clone()
            .or_else(|| self.events_iter.next());

        if self.block_start >= self.buffer_channels.len() {
            return None;
        }

        match event {
            Some(event) if (event.timing() as usize) <= self.block_start => {
                self.previous_event = None;
                Some(EventBlock::Event(event))
            }
            Some(event) if (event.timing() as usize) < self.block_end => {
                self.block_end = event.timing() as usize;
                self.previous_event = Some(event);
                let block = unsafe { self.block_mut() };
                Some(EventBlock::Block(block))
            }
            _ => {
                let block = unsafe { self.block_mut() };
                self.previous_event = None;
                self.block_start = self.block_end;
                self.block_end = self.buffer_channels.len().max(self.max_block_size);
                Some(EventBlock::Block(block))
            }
        }
    }
}

impl<'process, 'samples, Context, Plugin> FusedIterator
    for BlocksIterMut<'process, 'samples, Context, Plugin>
where
    Context: ProcessContext<Plugin>,
    Plugin: PluginTrait,
{
}

pub trait IntoBlocksIterMut<'process, 'samples, Context, Plugin>
where
    Context: ProcessContext<Plugin>,
    Plugin: PluginTrait,
{
    fn events_blocks_iter_mut(
        self,
        context: &'process mut Context,
        max_block_size: usize,
    ) -> BlocksIterMut<'process, 'samples, Context, Plugin>;
}

impl<'process, 'samples, Context, Plugin> IntoBlocksIterMut<'process, 'samples, Context, Plugin>
    for &'process mut Buffer<'samples>
where
    Context: ProcessContext<Plugin>,
    Plugin: PluginTrait,
{
    fn events_blocks_iter_mut(
        self,
        context: &'process mut Context,
        max_block_size: usize,
    ) -> BlocksIterMut<'process, 'samples, Context, Plugin> {
        BlocksIterMut::new(self, context, max_block_size)
    }
}
