pub use self::paging::{PhysicalAddress, test_paging};
pub use self::alloc::Allocator;

mod alloc;
mod buddy;
mod paging;

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Frame {
    number: usize,
    num_pages: usize,  // Number of pages allocated to this frame
}

impl Frame {
    pub fn from_address(&self, address: usize, num_pages: usize) -> Frame {
        Frame{
            number: address / PAGE_SIZE,
            num_pages: num_pages,
        }
    }

    pub fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }
}

pub trait FrameAllocator {
    fn allocate(&mut self, num_pages: usize) -> Option<Frame>;
    fn deallocate(&mut self, frame: Frame);
}
