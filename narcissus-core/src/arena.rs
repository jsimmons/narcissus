use std::{alloc::Layout, cell::Cell, mem::MaybeUninit, ptr::NonNull};

use crate::oom;

#[derive(Debug)]
pub struct AllocError;

impl std::fmt::Display for AllocError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for AllocError {}

#[inline(always)]
unsafe fn layout_from_size_align(size: usize, align: usize) -> Layout {
    if cfg!(debug_assertions) {
        Layout::from_size_align(size, align).unwrap()
    } else {
        Layout::from_size_align_unchecked(size, align)
    }
}

/// Wrapper around a pointer to a page footer.
///
/// Allows us to easily borrow the least significant bit of the page pointer to keep track of
/// whether a given page was allocated on the heap, with the global allocator. Or if it is the stack
/// page in a HybridArena.
#[derive(Clone, Copy)]
struct PagePointer(*mut PageFooter);

impl PagePointer {
    #[inline(always)]
    fn empty() -> PagePointer {
        PagePointer(&EMPTY_PAGE as *const PageFooterSync as *mut PageFooter)
    }

    #[inline(always)]
    fn new_stack(page: *mut PageFooter) -> PagePointer {
        PagePointer(((page as usize) | 0x1) as *mut PageFooter)
    }

    #[inline(always)]
    fn new_heap(page: *mut PageFooter) -> PagePointer {
        PagePointer(page)
    }

    #[inline(always)]
    fn is_empty(self) -> bool {
        self.0 == &EMPTY_PAGE as *const PageFooterSync as *mut PageFooter
    }

    #[inline(always)]
    fn is_stack(self) -> bool {
        self.0 as usize & 0x1 == 1
    }

    #[inline(always)]
    fn as_ptr(self) -> *mut PageFooter {
        (self.0 as usize & !0x1) as *mut PageFooter
    }

    #[inline(always)]
    unsafe fn as_ref<'a>(&self) -> &'a PageFooter {
        &*self.as_ptr()
    }
}

#[repr(C)]
#[repr(align(16))]
struct PageFooter {
    /// Pointer to the start of this page.
    base: NonNull<u8>,
    /// Pointer to the current bump allocation cursor. Must be within the range `base..=&self`.
    bump: Cell<NonNull<u8>>,
    /// Page size in bytes.
    size: usize,
    /// Pointer to the next page.
    next: Cell<PagePointer>,
}

const PAGE_FOOTER_SIZE: usize = std::mem::size_of::<PageFooter>();
const PAGE_MIN_SIZE: usize = 64; // 64 bytes (32 bytes for footer)
const PAGE_MAX_SIZE: usize = 256 * 1024 * 1024; // 256 MiB

impl PageFooter {
    /// Fast path allocation from this page
    #[inline(always)]
    fn try_alloc_layout(&self, layout: Layout) -> Option<NonNull<u8>> {
        unsafe {
            let base = self.base.as_ptr();
            let bump = self.bump.get().as_ptr();

            // Check structure invariants.
            debug_assert!(base <= bump);
            debug_assert!(bump as *const u8 <= self as *const _ as *const u8);

            // Guard against underflow.
            if (bump as usize) < layout.size() {
                return None;
            }

            // Cannot wrap due to guard above.
            let bump = bump.wrapping_sub(layout.size());
            let remainder = bump as usize & (layout.align() - 1);
            // Cannot have a remainder greater than the magnitude of the value, so this cannot wrap.
            let bump = bump.wrapping_sub(remainder);

            if bump >= base {
                // Cannot be null because `base` cannot be null (derived from `NonNull<u8>`).
                let bump = NonNull::new_unchecked(bump);
                self.bump.set(bump);
                Some(bump)
            } else {
                None
            }
        }
    }

    /// Reset the bump pointer for this page, freeing it up to be allocated again.
    ///
    /// # Safety
    ///
    /// This must only be called on pages which have no outstanding references to allocations, as it
    /// allows subsequent operations to allocate the same addresses.
    unsafe fn reset(&self) {
        self.bump.set(NonNull::new_unchecked(
            self.base.as_ptr().add(self.size - PAGE_FOOTER_SIZE),
        ));
    }
}

/// Special type for the empty page because static requires Sync.
/// Safe because the empty page is immutable.
#[repr(transparent)]
struct PageFooterSync(PageFooter);
unsafe impl Sync for PageFooterSync {}

static EMPTY_PAGE: PageFooterSync = PageFooterSync(unsafe {
    PageFooter {
        base: NonNull::new_unchecked(&EMPTY_PAGE as *const PageFooterSync as *mut u8),
        bump: Cell::new(NonNull::new_unchecked(
            &EMPTY_PAGE as *const PageFooterSync as *mut u8,
        )),
        size: 0,
        next: Cell::new(PagePointer(
            &EMPTY_PAGE as *const PageFooterSync as *mut PageFooter,
        )),
    }
});

/// Create a new page, large enough for the given layout, and prepend it to the linked list of
/// pages.
///
/// Returns the new page.
///
/// # Safety
///
/// `page` must refer to a valid page footer, or the empty page.
#[cold]
unsafe fn prepend_new_page(page: PagePointer, layout: Layout) -> Option<PagePointer> {
    let page_size = page.as_ref().size;
    // Double each allocated page to amortize allocation cost.
    let new_page_size = page_size * 2;
    // Clamp between `PAGE_MIN_SIZE` and `PAGE_MAX_SIZE` to handle the case where the existing
    // page is the empty page, and to avoid overly large allocated blocks.
    let new_page_size = new_page_size.max(PAGE_MIN_SIZE).min(PAGE_MAX_SIZE);
    // Ensure that after all that, the given page is large enough to hold the thing we're trying
    // to allocate.
    let new_page_size = new_page_size.max(layout.size() + layout.align() + PAGE_FOOTER_SIZE);
    let size_without_footer = new_page_size - PAGE_FOOTER_SIZE;
    debug_assert_ne!(size_without_footer, 0);

    let layout = layout_from_size_align(new_page_size, std::mem::align_of::<PageFooter>());
    let base_ptr = std::alloc::alloc(layout);
    let base = NonNull::new(base_ptr)?;
    let bump = NonNull::new_unchecked(base_ptr.add(size_without_footer));
    let footer = bump.as_ptr() as *mut PageFooter;
    debug_assert_ne!(base, bump);
    debug_assert!(base < bump);

    std::ptr::write(
        footer,
        PageFooter {
            base,
            bump: Cell::new(bump),
            size: new_page_size,
            next: Cell::new(page),
        },
    );

    Some(PagePointer::new_heap(footer))
}

/// Deallocate the given page if it was allocated with the global allocator, and all the heap pages
/// linked to it.
///
/// # Safety
///
/// Must not be called on any pages that hold live allocations, or pages which link to pages that
/// hold live allocations.
#[cold]
unsafe fn deallocate_page_list(mut page: PagePointer) {
    // Walk the linked list of pages and deallocate each one that originates from the heap.
    // Only the second last page can be a stack page, and it links to the empty page.
    while !page.is_empty() && !page.is_stack() {
        let p = page;
        page = page.as_ref().next.get();

        let layout = layout_from_size_align(p.as_ref().size, std::mem::align_of::<PageFooter>());
        std::alloc::dealloc(p.as_ref().base.as_ptr(), layout);
    }
}

/// An allocation arena.
///
/// Bump allocates within pages allocated from the global heap allocator.
///
/// Objects that are allocated within the arena will never have their `Drop` function called.
#[repr(C)]
pub struct Arena {
    page_list_head: Cell<PagePointer>,
}

/// An allocation arena with an allocation region that lives on the stack.
///
/// Bump allocates from the stack page until it's exhausted, then behaves like a regular `Arena`.
///
/// Objects that are allocated within the arena will never have their `Drop` function called.
#[repr(C)]
pub struct HybridArena<const STACK_CAP: usize> {
    data: MaybeUninit<[u8; STACK_CAP]>,
    footer: Cell<PageFooter>,
    page_list_head: Cell<PagePointer>,
}

impl Arena {
    pub fn new() -> Arena {
        Arena {
            page_list_head: Cell::new(PagePointer::empty()),
        }
    }

    /// Reset the arena.
    ///
    /// Releases all pages to the global allocator, except for the most recently allocated one,
    /// which has its bump pointer reset.
    ///
    /// Does not call destructors on any objects allocated by the pool.
    pub fn reset(&mut self) {
        // We don't want to write to the static empty page, so abandon here if we haven't allocated
        // any pages.
        if self.page_list_head.get().is_empty() {
            return;
        }

        unsafe {
            let page = self.page_list_head.get().as_ref();
            // Clear the current page.
            page.reset();
            // Truncate the linked list by appending the empty page, then free the rest.
            let page_after_head = page.next.replace(PagePointer::empty());
            deallocate_page_list(page_after_head)
        }
    }

    #[inline(always)]
    pub fn alloc<T>(&self, value: T) -> &mut T {
        // Safety: We allocate memory for `T` and then write a `T` into that location.
        unsafe {
            let layout = Layout::new::<T>();
            let ptr = self.alloc_layout(layout);
            let ptr = ptr.as_ptr() as *mut T;
            std::ptr::write(ptr, value);
            &mut *ptr
        }
    }

    #[inline(always)]
    pub fn alloc_with<T, F>(&self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        // Safety: We allocate memory for `T` and then write a `T` into that location.
        unsafe {
            let layout = Layout::new::<T>();
            let ptr = self.alloc_layout(layout);
            let ptr = ptr.as_ptr() as *mut T;
            std::ptr::write(ptr, f());
            &mut *ptr
        }
    }

    #[inline(always)]
    pub fn try_alloc_with<T, F>(&self, f: F) -> Result<&mut T, AllocError>
    where
        F: FnOnce() -> T,
    {
        // Safety: We allocate memory for `T` and then write a `T` into that location.
        unsafe {
            let layout = Layout::new::<T>();
            let ptr = match self.try_alloc_layout(layout) {
                Ok(ptr) => ptr,
                Err(e) => return Err(e),
            };
            let ptr = ptr.as_ptr() as *mut T;
            std::ptr::write(ptr, f());
            Ok(&mut *ptr)
        }
    }

    #[inline(always)]
    pub fn alloc_layout(&self, layout: Layout) -> NonNull<u8> {
        match self.try_alloc_layout(layout) {
            Ok(ptr) => ptr,
            Err(_) => oom(),
        }
    }

    #[inline(always)]
    pub fn try_alloc_layout(&self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        if let Some(ptr) = unsafe { self.page_list_head.get().as_ref() }.try_alloc_layout(layout) {
            Ok(ptr)
        } else {
            self.try_alloc_layout_slow(layout)
        }
    }

    #[inline(never)]
    #[cold]
    fn try_alloc_layout_slow(&self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        unsafe {
            // Current page head is either a valid page, or the empty page.
            self.page_list_head
                .set(prepend_new_page(self.page_list_head.get(), layout).ok_or(AllocError)?);

            // Can not fail as new pages are created with enough space for the requested
            // allocation.
            Ok(self
                .page_list_head
                .get()
                .as_ref()
                .try_alloc_layout(layout)
                .unwrap_unchecked())
        }
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_slice_copy<T>(&self, src: &[T]) -> &mut [T]
    where
        T: Copy,
    {
        let layout = Layout::for_value(src);
        let len = src.len();
        let src = src.as_ptr();
        let dst = self.alloc_layout(layout).cast::<T>().as_ptr();

        // Safety: We allocate dst with the same size as src before copying into it.
        unsafe {
            std::ptr::copy_nonoverlapping(src, dst, len);
            std::slice::from_raw_parts_mut(dst, len)
        }
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_slice_clone<T>(&self, src: &[T]) -> &mut [T]
    where
        T: Clone,
    {
        let layout = Layout::for_value(src);
        let dst = self.alloc_layout(layout).cast::<T>().as_ptr();

        // Safety: We allocate dst with the same size as src before copying into it.
        unsafe {
            for (i, value) in src.iter().cloned().enumerate() {
                std::ptr::write(dst.add(i), value);
            }
            std::slice::from_raw_parts_mut(dst, src.len())
        }
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_str(&self, src: &str) -> &mut str {
        let str = self.alloc_slice_copy(src.as_bytes());
        // Safety: We've just copied this string from a valid `&str`, so it must be valid too.
        unsafe { std::str::from_utf8_unchecked_mut(str) }
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        unsafe { deallocate_page_list(self.page_list_head.get()) }
    }
}

impl<const STACK_CAP: usize> HybridArena<STACK_CAP> {
    pub fn new() -> Self {
        // Ideally we'd pad `STACK_CAP` out to the alignment, avoiding wasting any space, but we
        // can't do maffs with constants just yet, so abort instead.
        debug_assert!(STACK_CAP % std::mem::align_of::<PageFooter>() == 0);
        Self {
            data: MaybeUninit::uninit(),
            footer: Cell::new(PageFooter {
                base: NonNull::dangling(),
                bump: Cell::new(NonNull::dangling()),
                size: STACK_CAP,
                next: Cell::new(PagePointer::empty()),
            }),
            page_list_head: Cell::new(PagePointer::empty()),
        }
    }

    /// Reset the arena.
    ///
    /// Releases all pages to the global allocator, except for the most recently allocated one,
    /// which has its bump pointer reset.
    ///
    /// Does not call destructors on any objects allocated by the pool.
    pub fn reset(&mut self) {
        // We don't want to write to the static empty page, so abandon here if we haven't allocated
        // any pages.
        if self.page_list_head.get().is_empty() {
            return;
        }

        unsafe {
            let page = self.page_list_head.get().as_ref();
            // Clear the current page.
            page.reset();
            // Truncate the linked list by appending the empty page, then free the rest.
            let page_after_head = page.next.replace(PagePointer::empty());
            deallocate_page_list(page_after_head)
        }
    }

    #[inline(always)]
    pub fn alloc<T>(&self, value: T) -> &mut T {
        // Safety: We allocate memory for `T` and then write a `T` into that location.
        unsafe {
            let layout = Layout::new::<T>();
            let ptr = self.alloc_layout(layout);
            let ptr = ptr.as_ptr() as *mut T;
            std::ptr::write(ptr, value);
            &mut *ptr
        }
    }

    #[inline(always)]
    pub fn alloc_with<T, F>(&self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        // Safety: We allocate memory for `T` and then write a `T` into that location.
        unsafe {
            let layout = Layout::new::<T>();
            let ptr = self.alloc_layout(layout);
            let ptr = ptr.as_ptr() as *mut T;
            std::ptr::write(ptr, f());
            &mut *ptr
        }
    }

    #[inline(always)]
    pub fn try_alloc_with<T, F>(&self, f: F) -> Result<&mut T, AllocError>
    where
        F: FnOnce() -> T,
    {
        // Safety: We allocate memory for `T` and then write a `T` into that location.
        unsafe {
            let layout = Layout::new::<T>();
            let ptr = match self.try_alloc_layout(layout) {
                Ok(ptr) => ptr,
                Err(e) => return Err(e),
            };
            let ptr = ptr.as_ptr() as *mut T;
            std::ptr::write(ptr, f());
            Ok(&mut *ptr)
        }
    }

    #[inline(always)]
    pub fn alloc_layout(&self, layout: Layout) -> NonNull<u8> {
        match self.try_alloc_layout(layout) {
            Ok(ptr) => ptr,
            Err(_) => oom(),
        }
    }

    #[inline(always)]
    pub fn try_alloc_layout(&self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        // When the arena is in its initial state, the head points to an empty page. In this case we
        // need to "allocate" the stack page and set the page head.
        //
        // We also need to ensure that if we're allocating into a hybrid array, that no moves have
        // happened in the meantime.
        //
        // That is we need to avoid failure in the following situation.
        //
        // ```
        // let arena = HybridArena::<4>::new();
        // let x = arena.alloc(1);
        //
        // fn take_arena(arena: HybridArena<4>) -> HybridArena<4> {
        //     let y = arena.alloc(2);
        //     arena
        // }
        //
        // let arena = take_arena(arena);
        // let z = arena.alloc(3);
        // ```
        //
        // Allocating in an arena that links to a stack page that isn't the same address as our
        // current self's page address, is a memory safety failure.
        //
        // It's safe to reset the page in this case, becuase it's only possible to move the arena
        // while there are no references pinning it in place.
        let page = self.page_list_head.get();
        if page.is_empty() || (page.is_stack() && page.as_ptr() != self.footer.as_ptr()) {
            unsafe { self.setup_hybrid_page() }
        }

        if let Some(ptr) = unsafe { self.page_list_head.get().as_ref() }.try_alloc_layout(layout) {
            Ok(ptr)
        } else {
            self.try_alloc_layout_slow(layout)
        }
    }

    /// When a hybrid array is in its default state, or when it has been moved, it's necessary to
    /// fix-up the page footer and page list head.
    ///
    /// # Safety
    ///
    /// Must not be called when there are outstanding allocations, as it will reset the hybrid page.
    #[inline(never)]
    #[cold]
    unsafe fn setup_hybrid_page(&self) {
        let base = self.data.as_ptr() as *mut u8;
        let bump = base.add(STACK_CAP);
        self.footer.set(PageFooter {
            base: NonNull::new_unchecked(base),
            bump: Cell::new(NonNull::new_unchecked(bump)),
            size: STACK_CAP + PAGE_FOOTER_SIZE,
            next: Cell::new(PagePointer::empty()),
        });
        debug_assert_eq!(base as usize, self as *const _ as usize);
        debug_assert_eq!(bump as usize, self.footer.as_ptr() as usize);
        self.page_list_head
            .set(PagePointer::new_stack(self.footer.as_ptr()));
    }

    #[inline(never)]
    #[cold]
    fn try_alloc_layout_slow(&self, layout: Layout) -> Result<NonNull<u8>, AllocError> {
        unsafe {
            // Current page head is either a valid page, or the empty page.
            self.page_list_head
                .set(prepend_new_page(self.page_list_head.get(), layout).ok_or(AllocError)?);

            // Can not fail as new pages are created with enough space for the requested
            // allocation.
            Ok(self
                .page_list_head
                .get()
                .as_ref()
                .try_alloc_layout(layout)
                .unwrap_unchecked())
        }
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_slice_copy<T>(&self, src: &[T]) -> &mut [T]
    where
        T: Copy,
    {
        let layout = Layout::for_value(src);
        let len = src.len();
        let src = src.as_ptr();
        let dst = self.alloc_layout(layout).cast::<T>().as_ptr();

        // Safety: We allocate dst with the same size as src before copying into it.
        unsafe {
            std::ptr::copy_nonoverlapping(src, dst, len);
            std::slice::from_raw_parts_mut(dst, len)
        }
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_slice_clone<T>(&self, src: &[T]) -> &mut [T]
    where
        T: Clone,
    {
        let layout = Layout::for_value(src);
        let dst = self.alloc_layout(layout).cast::<T>().as_ptr();

        // Safety: We allocate dst with the same size as src before copying into it.
        unsafe {
            for (i, value) in src.iter().cloned().enumerate() {
                std::ptr::write(dst.add(i), value);
            }
            std::slice::from_raw_parts_mut(dst, src.len())
        }
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc_str(&self, src: &str) -> &mut str {
        let str = self.alloc_slice_copy(src.as_bytes());
        // Safety: We've just copied this string from a valid `&str`, so it must be valid too.
        unsafe { std::str::from_utf8_unchecked_mut(str) }
    }
}

impl<const STACK_CAP: usize> Default for HybridArena<STACK_CAP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const STACK_CAP: usize> Drop for HybridArena<STACK_CAP> {
    fn drop(&mut self) {
        unsafe { deallocate_page_list(self.page_list_head.get()) }
    }
}

#[cfg(test)]
mod tests {
    use super::{Arena, HybridArena};
    #[test]
    fn arena() {
        let mut arena = Arena::new();
        let x = arena.alloc(100);
        let y = arena.alloc(100);
        assert_eq!(*x, *y);
        assert_ne!(x as *const i32, y as *const i32);
        for i in 0..1_000_000 {
            _ = arena.alloc(i);
        }
        arena.reset();
        for i in 0..1_000_000 {
            _ = arena.alloc(i);
        }
        arena.reset();
        for i in 0..1_000_000 {
            _ = arena.alloc(i);
        }
    }

    #[test]
    fn hybrid_arena() {
        let mut arena = HybridArena::<32>::new();
        let x = arena.alloc(100);
        let y = arena.alloc(100);
        assert_eq!(*x, *y);
        assert_ne!(x as *const i32, y as *const i32);
        for i in 0..1_000_000 {
            _ = arena.alloc(i);
        }
        arena.reset();
        for i in 0..1_000_000 {
            _ = arena.alloc(i);
        }
        arena.reset();
        for i in 0..1_000_000 {
            _ = arena.alloc(i);
        }

        // move it move it
        let mut arena = HybridArena::<16>::new();
        let x = arena.alloc(1);
        assert_eq!(*x, 1);
        arena.reset();

        fn take_arena(arena: HybridArena<16>) -> HybridArena<16> {
            let y = arena.alloc(2);
            assert_eq!(*y, 2);
            arena
        }

        let arena = take_arena(arena);
        let z = arena.alloc(3);
        assert_eq!(*z, 3);
    }
}
