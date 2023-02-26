use narcissus_core::{box_assume_init, uninit_box};
use stb_truetype_sys::{rectpack, stbrp_init_target, stbrp_pack_rects};

pub use rectpack::Rect;

pub struct Packer {
    context: Box<rectpack::Context>,
    nodes: Box<[rectpack::Node]>,
    width: i32,
    height: i32,
}

impl Packer {
    /// Create a new rectangle packer.
    ///
    /// # Panics
    /// Panics if width or height exceed i32::MAX.
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width < i32::MAX as usize && height < i32::MAX as usize);

        let mut nodes = vec![rectpack::Node::default(); width].into_boxed_slice();

        let width = width as i32;
        let height = height as i32;

        // Safety: `nodes` must not be deleted while context lives, and `context` must not be
        //         relocated.
        let context = unsafe {
            let mut context = uninit_box();
            stbrp_init_target(
                context.as_mut_ptr(),
                width,
                height,
                nodes.as_mut_ptr(),
                width, // Matches node count.
            );
            box_assume_init(context)
        };

        Self {
            context,
            nodes,
            width,
            height,
        }
    }

    /// Clear all previously packed rectangle state.
    pub fn clear(&mut self) {
        // Safety: `context` and `nodes` are always valid while packer exists, and width always
        //         matches node count.
        unsafe {
            stbrp_init_target(
                self.context.as_mut(),
                self.width,
                self.height,
                self.nodes.as_mut_ptr(),
                self.width,
            )
        }
    }

    /// Pack the provided rectangles into the rectangle given when the packer was created.
    ///
    /// Calling this function multiple times to incrementally pack a collection of rectangles may
    /// be less effective than packing the entire collection all at once.
    ///
    /// Returns true if all rectangles were successfully packed.
    pub fn pack(&mut self, rects: &mut [rectpack::Rect]) -> bool {
        let num_rects = rects.len().try_into().expect("too many rects to pack");
        // Safety: `context` and `nodes` are always valid while packer exists.
        let ret = unsafe { stbrp_pack_rects(self.context.as_mut(), rects.as_mut_ptr(), num_rects) };
        ret == 1
    }
}
