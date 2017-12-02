use memory::{Frame, FrameAllocator, PAGE_SIZE};
use memory::buddy::{Buddy};

pub struct Allocator {
    buddy: Buddy,
}

impl Allocator {
    pub fn new(kernel_end: usize,
        multiboot_start: usize, multiboot_end: usize) -> Allocator
    {
        let mut alloc = Allocator{
            buddy: Buddy::new(),
        };
        // Mark kernel/multiboot memory as used
        let kernel_pages = kernel_end / PAGE_SIZE;
        alloc.buddy.mark_used(kernel_pages, 0);
        let multiboot_size = multiboot_end - multiboot_start;
        let multiboot_offset = multiboot_start / PAGE_SIZE;
        alloc.buddy.mark_used(1, multiboot_offset);
        alloc
    }
}

impl FrameAllocator for Allocator {
    fn allocate(&mut self, num_pages: usize) -> Option<Frame> {
        let frame_number = self.buddy.allocate(num_pages);
        if frame_number >= 0 {
            Some(Frame{
                number: frame_number as usize,
                num_pages: num_pages,
            })
        } else {
            None
        }
    }

    fn deallocate(&mut self, _frame: Frame) {
        unimplemented!()
    }
}
