use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

use memory::FrameAllocator;
use memory::paging::{Page, ENTRY_COUNT};
use memory::paging::entry::{Entry, EntryFlags};

pub struct Table<L>
where
    L: TableLevel,
{
    entries: [Entry; ENTRY_COUNT],
    level: PhantomData<L>,
}

impl<L> Index<usize> for Table<L>
where
    L: TableLevel,
{
    type Output = Entry;

    fn index(&self, index: usize) -> &Entry {
        &self.entries[index]
    }
}

impl<L> IndexMut<usize> for Table<L>
where
    L: TableLevel,
{
    fn index_mut(&mut self, index: usize) -> &mut Entry {
        &mut self.entries[index]
    }
}

impl<L> Table<L>
where
    L: TableLevel,
{
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }
}

impl<L> Table<L>
where
    L: HierarchicalLevel,
{

    pub fn next_table<'a>(&'a self, index: usize) -> Option<&'a Table<L::NextLevel>>
    {
        self.next_table_address(index)
            .map(|addr| unsafe { &*(addr as *const _) })
    }

    pub fn next_table_mut<'a>(&'a self, index: usize) -> Option<&'a mut Table<L::NextLevel>>
    {
        self.next_table_address(index)
            .map(|addr| unsafe { &mut *(addr as *mut _) })
    }

    pub fn next_table_or_create<'a, A>(&'a mut self, index: usize, allocator: &mut A) -> &'a mut Table<L::NextLevel>
    where
        A: FrameAllocator,
    {
        if self.next_table(index).is_none() {
            // Disable huge pages for now
            assert!(!self.entries[index].flags().contains(EntryFlags::HUGE_PAGE),
                    "huge pages is disabled in the mapper");
            let frame = allocator.allocate(1).expect("no more physical memory frames are available");
            println!("creating next table index: {}, frame: {:?}", index, frame);
            self.entries[index].set(frame, EntryFlags::PRESENT | EntryFlags::WRITABLE);
            self.next_table_mut(index).unwrap().zero();
        }
        self.next_table_mut(index).unwrap()
    }

    fn next_table_address(&self, index: usize) -> Option<usize> {
        let entry_flags = self[index].flags();
        if entry_flags.contains(EntryFlags::PRESENT) && !entry_flags.contains(EntryFlags::HUGE_PAGE) {
            let table_address = self as *const _ as usize;
            // Since the P4 table recursively references itself... the last 9 bit (3 octal) page
            // address (before the 12 bit index offset) points at the NEXT table
            // We shift the table address over and shift the index into this 9 bit space so it
            // becomes the pointer to the next table
            Some((table_address << 9) | index << 12)
        } else {
            None
        }
    }
}

pub const P4: *mut Table<Level4> = 0o177777_777_777_777_777_0000 as *mut _;

pub trait TableLevel {}

pub enum Level4 {}
pub enum Level3 {}
pub enum Level2 {}
pub enum Level1 {}

impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}

pub trait HierarchicalLevel: TableLevel {
    type NextLevel: TableLevel;
}

impl HierarchicalLevel for Level4 {
    type NextLevel = Level3;
}
impl HierarchicalLevel for Level3 {
    type NextLevel = Level2;
}
impl HierarchicalLevel for Level2 {
    type NextLevel = Level1;
}
