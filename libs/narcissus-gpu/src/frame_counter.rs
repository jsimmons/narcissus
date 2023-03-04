use std::{
    marker::PhantomData,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::Frame;

pub struct FrameCounter {
    value: AtomicUsize,
}

impl FrameCounter {
    pub fn new() -> Self {
        Self {
            // Start the frame id at 1 so that the first `begin_frame` ticks us
            // over to a new frame index.
            value: AtomicUsize::new(1),
        }
    }

    pub fn load(&self) -> usize {
        self.value.load(Ordering::Relaxed)
    }

    pub fn acquire(&self, device_addr: usize) -> Frame {
        let old_frame_counter = self.value.fetch_add(1, Ordering::SeqCst);
        assert!(
            old_frame_counter & 1 == 1,
            "acquiring a frame before previous frame has been released"
        );

        let frame_counter = old_frame_counter + 1;
        let frame_index = frame_counter >> 1;

        Frame {
            device_addr,
            frame_index,
            _phantom: &PhantomData,
        }
    }

    pub fn release(&self, frame: Frame) {
        let old_frame_counter = self.value.fetch_add(1, Ordering::SeqCst);
        frame.check_frame_counter(old_frame_counter);
    }
}
