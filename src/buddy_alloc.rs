// Set size as constants, trying to dynamically size the tree at runtime would require
// creating an unsafe pointer to an arbitrary memory address and setting the manually
// setting the memory.
const LEVELS: usize = 8;  // Can allocate 2 ^ LEVELS blocks of memory
const SIZE: usize = (1 << LEVELS + 1) - 1;

#[repr(u8)]
#[derive(PartialEq, Copy, Clone)]
enum Node {
    Unused,
    Used,
    Split,
    Full,
}

pub struct BuddyAllocator {
    levels: usize,
    tree: [Node; SIZE],
}

impl BuddyAllocator {
    pub fn new() -> BuddyAllocator {
        BuddyAllocator{
            levels: LEVELS,
            tree: [Node::Unused; SIZE],
        }
    }

    // Takes a size (# of blocks requested) and returns an index offset
    pub fn allocate(&mut self, s: usize) -> isize {
        // Get the number of blocks requested
        let requested_blocks;
        if s == 0 {
            requested_blocks = 1;
        } else {
            requested_blocks = s.next_power_of_two();
        }
        let requested_level = self.log_base_2(requested_blocks);
        if requested_level > self.levels {
            return -1;
        }

        // start at index 0 and move in
        let mut index = 0;
        let mut current_level = self.levels;
        'forward: loop {
            let has_buddy = index & 1 == 1;
            if current_level != requested_level {
                match self.tree[index] {
                    Node::Used | Node::Full => {
                        // Check the buddy node if we haven't already
                        if has_buddy {
                            index += 1;
                        }
                        continue 'forward;
                    }
                    Node::Unused => {
                        // Split the node and descend
                        self.tree[index] = Node::Split;
                        index = index * 2 + 1;
                        current_level -= 1;
                        continue 'forward;
                    }
                    Node::Split => {
                        // Just descend
                        index = index * 2 + 1;
                        current_level -= 1;
                        continue 'forward;
                    }
                }
            } else {
                // Requested level and current level match up
                if self.tree[index] == Node::Unused {
                    self.tree[index] = Node::Used;
                    // Recursively check if parents are full and mark them as such
                    self.update_parents((index + 1) / 2 - 1);
                    break 'forward;
                }
            }
            // Check buddy node if we haven't already
            if has_buddy {
                index += 1;
                continue 'forward;
            }
            // Backtrack if we reach a level match AND we've checked both nodes
            'backward: loop {
                index = (index + 1) / 2 - 1;
                current_level += 1;
                let has_buddy_inner = index & 1 == 1;
                if has_buddy_inner {
                    index += 1;
                    break 'backward;
                }
            }
        }

        return index as isize;
    }

    pub fn free(&mut self, index_offset: usize) {
        if index_offset > self.tree.len() - 1 {
            panic!("offset {} is > length of tree {}", index_offset, self.tree.len());
        }
        // Recursively free and combine nodes
        self.free_and_combine(index_offset);

        // Recursively update parents
        self.update_parents((index_offset + 1) / 2 - 1);
    }

    fn free_and_combine(&mut self, index: usize) {
        self.tree[index] = Node::Unused;
        // We are already at the top of the tree, we're done
        if index == 0 {
            return;
        }
        let other_node: usize;
        let has_right_buddy = (index & 1) == 1;
        if has_right_buddy {
            other_node = index + 1;
        } else {
            other_node = index - 1;
        }
        // Recursively combine nodes
        if self.tree[other_node] == Node::Unused {
            self.free_and_combine((index + 1) / 2 - 1);
        }
        return;
    }

    // Propagate changes up to parent nodes
    fn update_parents(&mut self, index: usize) {
        // Check both child nodes to see if they are both either FULL or USED
        let left_child = index * 2 + 1;
        let right_child = index * 2 + 2;
        let left_child_used_or_full = self.tree[left_child] == Node::Full || self.tree[left_child] == Node::Used;
        let right_child_used_or_full = self.tree[right_child] == Node::Full || self.tree[right_child] == Node::Used;
        if left_child_used_or_full && right_child_used_or_full {
            // Both children USED or FULL
            self.tree[index] = Node::Full;
        } else if self.tree[left_child] == Node::Unused && self.tree[right_child] == Node::Unused {
            // Both children are UNUSED
            self.tree[index] = Node::Unused;
        } else {
            // Default to split node if neither FULL or UNUSED
            self.tree[index] = Node::Split;
        }
        // We're at the top of the tree, we're done
        if index == 0 {
            return;
        }
        self.update_parents((index + 1) / 2 - 1);
    }

    // Finds the position of the most signifcant bit
    fn log_base_2(&self, requested_blocks: usize) -> usize {
        let mut exp = 0;
        let mut find_msb_bit = requested_blocks;
        find_msb_bit >>= 1;
        while (find_msb_bit > 0) {
            find_msb_bit >>= 1;
            exp += 1;
        }
        return exp;
    }
}
