use crate::util::bitset::BitSet;
use super::*;
use x86_64::structures::paging::page_table::PageTableEntry;
use core::ops::Range;

const MAX_FRAME_COUNT: u64 = 1 << 26;

fn frame_number(addr: PhysAddr) -> u64 {
    addr.as_u64() << 12
}

struct FrameRange {
    start: u64,
    count: u64,
}

fn huge_page_frame_range(level: i32, addr: PhysAddr) -> FrameRange {
    FrameRange {
        start: frame_number(addr),
        count: match level {
            2 => 1 << 9, // 2^9 pages
            3 => 1 << 18, // 2^18 pages
            _ => panic!("this should never happen")
        },
    }
}

struct FrameManager {
    bitset: BitSet<{ MAX_FRAME_COUNT }>
}

impl FrameManager {
    fn traverse_mark_entry(&mut self, level: i32,
                           page_table: &PageTable) {
        for entry in page_table.iter() {
            let flags = entry.flags();
            // is page present?
            if flags.contains(PageTableFlags::PRESENT) {
                // huge page, special treatment
                if flags.contains(PageTableFlags::HUGE_PAGE) {
                    let range = huge_page_frame_range(level, entry.addr());
                    self.bitset.set_all(range.start, range.start + range.count, true);
                } else {
                    if level == 1 {
                        self.bitset.set(frame_number(entry.addr()), true);
                    } else {
                        // go to next level
                        self.traverse_mark_entry(
                            level - 1,
                            unsafe { *physical_to_virtual(entry.addr()).as_ptr() })
                    }
                }
            }
        }
    }

    pub fn from_physical_memory(l4_table: &PageTable) -> FrameManager {
        let mut mgr = FrameManager { bitset: Default::default() };
        mgr.traverse_mark_entry(4, l4_table);
        mgr
    }

    // TODO: accelerate frame management
    pub fn alloc(&mut self) -> Option<UnusedPhysFrame> {
        unimplemented!()
    }

    pub fn dealloc(&mut self, frame: PhysFrame) {
        unimplemented!()
    }
}