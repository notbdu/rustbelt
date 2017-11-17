pub use self::alloc::Allocator;

mod alloc;
mod buddy;

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Frame {
    page_offset: usize,
    num_pages: usize,  // Number of pages allocated to this frame
}

pub trait FrameAllocator {
    fn allocate(&mut self, num_pages: usize) -> Option<Frame>;
    fn deallocate(&mut self, frame: Frame);
}
