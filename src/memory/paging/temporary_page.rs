use super::{ActivePageTable, Page, VirtualAddress};
use memory::{Frame, FrameAllocator};

struct TinyAllocator([Option<Frame>; 3]);

impl FrameAllocator for TinyAllocator {
    pub fn new<A>(allocator: A) -> TinyAllocator {
        // Allocate some 1 page frames
        let mut f = || allocator.allocate(1);
        let frames = [f(), f(), f()];
        TinyAllocator(frames)
    }

    fn allocate(&mut self, num_pages: usize) -> Option<Frame> {
        // Just going to assume the following are 1 page sized frames
        // Return the first unused frame
        for frame_option in &mut self.0 {
            if frame_option.is_some() {
                return frame_option.take();
            }
        }
    }

    fn deallocate(&mut self, frame: Frame) {
        for frame_option in &mut self.0 {
            if frame_option.is_none() {
                *frame_option = Some(frame);
                return;
            }
        }
    }
}

pub struct TemporaryPage {
        page: Page,
        allocator: TinyAllocator,
}

impl TemporaryPage {
    pub fn new<A>(page: Page, allocator: &mut A) -> TemporaryPage
        where A: FrameAllocator
    {
        TemporaryPage{
            page: page,
            allocator: TinyAllocator::new(allocator),
        }
    }
    /// Maps the temporary page to the given frame in the active table.
    /// Returns the start address of the temporary page.
    pub fn map(&mut self,
               frame: Frame,
               active_table: &mut ActivePageTable)
        -> VirtualAddress
    {
        use super::entry::WRITABLE;

        assert!(active_table.translate_page(self.page).is_none(),
                "temporary page is already mapped");
        active_table.map_to(self.page, frame, WRITABLE, self.allocator);
        self.page.start_address()
    }

    /// Unmaps the temporary page in the active table.
    pub fn unmap(&mut self, active_table: &mut ActivePageTable) {
        active_table.unmap(self.page, self.allocator)
    }

    pub fn map_table_frame(&mut self,
                           frame: Frame,
                           active_table: &mut ActivePageTable)
        -> &mut Table<Level1> {
        unsafe { &mut *(self.map(frame, active_table) as *mut Table<Level1> }
    }
}
