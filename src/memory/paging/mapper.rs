use core::ptr::Unique;

pub use self::entry::EntryFlags;
use self::table::{Table, Level4, P4};
use memory::{PAGE_SIZE, Frame, FrameAllocator};

pub struct Mapper {
    p4: Unique<Table<Level4>>,
}

impl Mapper {
    pub unsafe fn new() -> Mapper {
        Mapper{
            p4: Unique::new_unchecked(table::P4),
        }
    }

    pub fn translate(&self, address: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = address % PAGE_SIZE;
        self.translate_page(Page::from_address(address))
            .map(|frame| frame.number * PAGE_SIZE + offset)
    }

    pub fn map<A>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A)
    where
        A: FrameAllocator,
    {
        let frame = allocator.allocate(1).expect("no more physical memory frames are available");
        self.map_to(page, frame, flags, allocator)
    }

    pub fn identity_map<A>(&mut self, frame: Frame, flags: EntryFlags, allocator: &mut A)
    where
        A: FrameAllocator,
    {
        let page = Page::from_address(frame.start_address());
        self.map_to(page, frame, flags, allocator)
    }

    pub fn map_to<A>(&mut self, page: Page, frame: Frame,
                     flags: EntryFlags, allocator: &mut A)
    where
        A: FrameAllocator,
    {
        let p3 = self.p4_mut().next_table_or_create(page.p4_index(), allocator);
        let p2 = p3.next_table_or_create(page.p3_index(), allocator);
        let p1 = p2.next_table_or_create(page.p2_index(), allocator);

        // Make sure that the p1 table entry is unused
        assert!(p1[page.p1_index()].is_unused());
        println!("after is unused");
        // Flip the PRESENT flag and map the p1 table entry to the physical frame
        println!("mapping p1 index {} to frame {:?}", page.p1_index(), frame);
        p1[page.p1_index()].set(frame, flags | EntryFlags::PRESENT);
    }

    pub fn unmap<A>(&mut self, page: Page, allocator: &mut A)
    where
        A: FrameAllocator,
    {
        let p1 = self.p4_mut().next_table_mut(page.p4_index())
            .and_then(|p3| p3.next_table_mut(page.p3_index()))
            .and_then(|p2| p2.next_table_mut(page.p2_index()))
            // Only error expected at this point
            .expect("huge pages disabled");
        // Free frame pointer
        // allocator.free(p1[page.p1_index()].frame_pointer().expect("tried to unmap an unused
        // page"))
        p1[page.p2_index()].set_unused();

		// Flush the tlb cache
		use x86_64::instructions::tlb;
		use x86_64::VirtualAddress;
		tlb::flush(VirtualAddress(page.start_address()));
    }

    fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.as_ref() }
    }

    fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.as_mut() }
    }

    fn translate_page(&self, page: Page) -> Option<Frame> {
        let p3 = self.p4().next_table(page.p4_index());

        let huge_page = || {
            None
        };

        p3.and_then(|p3| p3.next_table(page.p3_index()))
            .and_then(|p2| p2.next_table(page.p2_index()))
            .and_then(|p1| p1[page.p1_index()].frame_pointer())
            .or_else(huge_page)
    }
}
