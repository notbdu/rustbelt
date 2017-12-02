use core::ptr::Unique;
use core::ops::{Deref, DerefMut};

pub use self::entry::EntryFlags;
use self::table::{Table, Level4, P4};
use self::temporary_page::{TemporaryPage};
use self::mapper::{Mapper};
use memory::{PAGE_SIZE, Frame, FrameAllocator};

mod table;
mod entry;
mod temporary_page;
mod mapper;

const ENTRY_COUNT: usize = 512;

// Each physical address should be page aligned to not have any 0-11 bits set.
// x86 physical addresses should be smaller than 2^52. This means that physical addresses
// should ONLY have bits 12-51 set.
pub type PhysicalAddress = usize;
// Each virtual address should also be page aligned.
// Bits 0-11 are offset bits while bits 12-47 are page table index bits.
// x86 virtual addresses should be smaller than 2^48. Only bits 0-47 are set w/ the remaining
// bits 48-63 are sign extension bits (copies of the MSB).
pub type VirtualAddress = usize;

/**
The bit layout of a virtual address is as follows:
    Bit(s)  Name                Meaning
    0-11    Offset bits         Offset in bytes of the final page table
    12-20   P1 index            Entry index on the P1 page table
    21-29   P2 index            Entry index on the P2 page table
    30-38   P3 index            Entry index on the P3 page table
    39-47   P4 index            Entry index on the P4 page table

Our P4 table is recursively mapped so table access adheres to the following invariant:
    Table   Address                             Indexes
    P4      0o177777_777_777_777_777_0000       –
    P3      0o177777_777_777_777_XXX_0000       XXX is the P4 index
    P2      0o177777_777_777_XXX_YYY_0000       like above, and YYY is the P3 index
    P1      0o177777_777_XXX_YYY_ZZZ_0000       like above, and ZZZ is the P2 index
Where bits 0o177777 (48-63) are the sign extension bits.
**/
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    number: usize,
}

impl Page {
    pub fn from_address(address: VirtualAddress) -> Page {
        // Addresses on x86 are just 48 bits long, the remaining bits are just sign extension
        // (copies of the msb). Address space is split in two halves. A higher half w/ sign
        // extension and a lower half w/o.
		assert!(address < 0x0000_8000_0000_0000 ||
			address >= 0xffff_8000_0000_0000,
			"invalid address: 0x{:x}", address);
        // NOTE: When we divide by `PAGE_SIZE`, the index methods below do not require a 12 bit
        // shift since this division basically shifts the bits over by 12 bits when page size is
        // 4096. See below:
        // 0o177777_777_777_777_XXX_0000 / 4096 -> 0o0000177777_777_777_777_XXX
        Page { number: address / PAGE_SIZE }
    }

	pub fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }

    fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }

    fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }

    fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }

    fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }
}

pub struct ActivePageTable {
    mapper: Mapper,
}

impl Deref for ActivePageTable {
    type Target = Mapper;
    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    type Target = Mapper;
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

impl ActivePageTable {
    unsafe fn new() -> ActivePageTable {
        ActivePageTable{
            mapper: Mapper::new(),
        }
    }

    pub fn with<F>(&mut self,
                   inactive_table: &mut InactivePageTable,
                   temporary_page: &mut TemporaryPage,
                   f: F)
        where F: FnOnce(&mut Mapper)
    {
		use x86_64::instructions::tlb;
		use x86_64::registers::control_regs;

        let active_table_backup = Frame::from_address(unsafe { control_regs::cr3() } as usize, 1);
        let p4_table = temporary_page.map_table_frame(active_table_backup.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
		// overwrite recursive mapping to point to the inactive page table
		self.p4_mut()[511].set(inactive_table.p4_frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);

		// flush translation lookaside buffer cache to clear old translations
		tlb::flush_all();

		// re-execute f with new context
		f(self);
    }
}

pub struct InactivePageTable {
    p4_frame: Frame,
}

impl InactivePageTable {
    // Creates valid `InactivePageTable`s that are zero'ed and recursively mapped
    pub fn new(frame: Frame,
               active_table: &mut ActivePageTable,
               temporary_page: &mut TemporaryPage)
        -> InactivePageTable
    {
        {
            // Place this block in an inner scope to ensure that the `table` variable is dropped
            // as soon as it goes out of scope. This is required since `table` exclusively borrows
            // `temporary_page` as long as its alive.
            let table = temporary_page.map_table_frame(frame.clone(), active_table);
            // zero out the table
            table.zero();
            // set up recursive mapping
            table[511].set(frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
        }
        temporary_page.unmap(active_table);

        InactivePageTable { p4_frame: frame }
    }
}

pub fn test_paging<A>(allocator: &mut A)
where
    A: FrameAllocator,
{
    let mut page_table = unsafe { ActivePageTable::new() };
    let addr: usize = 42 * 512 * 512 * 4096; // 42th P3 entry
    let page = Page::from_address(addr);
    let frame = allocator.allocate(1).expect("no more physical memory");
    page_table.map_to(page, frame, EntryFlags::empty(), allocator);
    println!("Some = {:?}", page_table.translate(addr));
	println!("{:#x}", unsafe {
		*(Page::from_address(addr).start_address() as *const u64)
	});
    page_table.unmap(page, allocator);
    println!("Some = {:?}", page_table.translate(addr));
	println!("{:#x}", unsafe {
		*(Page::from_address(addr).start_address() as *const u64)
	});
}
