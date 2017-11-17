use memory::{Frame, FrameAllocator, PAGE_SIZE};
use memory::buddy::{Buddy};

pub struct Allocator {
    buddy: Buddy,
    start: usize,
    end: usize,
}

impl Allocator {
    pub fn new(kernel_start: usize, kernel_end: usize,
        multiboot_start: usize, multiboot_end: usize) -> Allocator
    {
        let mut alloc = Allocator{
            buddy: Buddy::new(),
            start: kernel_start,
            end: kernel_end,
        };
        // Mark kernel/multiboot memory as used
        let kernel_size = kernel_end - kernel_start;
        let kernel_pages = (kernel_size / PAGE_SIZE) + 1;
        alloc.buddy.mark_used(kernel_pages, 0); // Kernel start is always 0
        let multiboot_size = multiboot_end - multiboot_start;
        let multiboot_pages = (multiboot_size / PAGE_SIZE) + 1;
        let multiboot_offset = (multiboot_start - kernel_start) / PAGE_SIZE;
        alloc.buddy.mark_used(multiboot_pages, multiboot_offset);
        alloc
    }
}

impl FrameAllocator for Allocator {
    fn allocate(&mut self, num_pages: usize) -> Option<Frame> {
        let page_offset = self.buddy.allocate(num_pages) as usize;
        // Make sure that the end does not overflow
        let end = (page_offset + num_pages) * PAGE_SIZE;
        if page_offset >= 0  && end < self.end {
            Some(Frame{
                page_offset: page_offset,
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
