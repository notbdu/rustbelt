use memory::Frame;
use memory::paging::ENTRY_COUNT;

const FLAG_MASK: usize = 0x000FFFFF_FFFFF000;

/**
The bit layout of each page table entry is as follows:
    Bit(s)  Name                        Meaning
    0       present                     the page is currently in memory
    1       writable                    it's allowed to write to this page
    2       user accessible             if not set, only kernel mode code can access this page
    3       write through caching       writes go directlyto memory
    4       disable cache               no cache is used for this page
    5       accessed                    the CPU sets this bit when this page is used
    6       dirty                       the CPU sets this bit when a write to this page occurs
    7       huge page/null              must be 0 in P1 and P4, creates a 1GiB page in P3, creates a 2MiB page in P2
    8       global                      page isn't flushed from caches on address space switch (PGE bit of CR4 register must be set)
    9-11    available                   can be used freely by the OS
    12-51   physical address            the page aligned 52bit physical address of the frame or the   next page table
    52-62   available                   can be used freely by the OS
    63      no execute                  forbid executing code on this page (the NXE bit in the EFER register must be set)
**/
pub struct Entry(u64);

impl Entry {
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    pub fn frame_pointer(&self) -> Option<Frame> {
        if self.flags().contains(EntryFlags::PRESENT) {
            Some(Frame{
                num_pages: 1,
                // Bits 12-51 represent the physical address
                // of the frame or next page table
                number: self.0 as usize & FLAG_MASK,
            })
        } else {
            None
        }
    }

    pub fn set(&mut self, frame: Frame, flags: EntryFlags) {
        // Assert that the frame address has no flag bits set.
        // Flag bits are bits > 51 and < 12 since the physical address only sits in bits 12-51.
        // When memory is 4096 byte aligned, the first 12 bits are never set. For example:
        // 4096     -> 0x00000000_00001000
        // 4096 * 2 -> 0x00000000_00002000
        // 4096 * 3 -> 0x00000000_00003000
        assert!(frame.start_address() & !FLAG_MASK == 0);
        // Set both physical address && flag bits
        self.0 = (frame.start_address() as u64) | flags.bits();
    }
}

bitflags! {
    pub struct EntryFlags: u64 {
        const PRESENT =         1 << 0;
        const WRITABLE =        1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH =   1 << 3;
        const NO_CACHE =        1 << 4;
        const ACCESSED =        1 << 5;
        const DIRTY =           1 << 6;
        const HUGE_PAGE =       1 << 7;
        const GLOBAL =          1 << 8;
        const NO_EXECUTE =      1 << 63;
    }
}
