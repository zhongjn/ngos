use bootloader::bootinfo::{FrameRange, MemoryMap, MemoryRegion, MemoryRegionType};
use heapless::consts::U16;
use heapless::Vec;
use x86_64::PhysAddr;
use x86_64::structures::paging::Page;

use crate::util::bitset::BitSet;

use super::*;
use x86_64::structures::paging::page_table::FrameError::FrameNotPresent;
use core::cell::RefCell;
use core::pin::Pin;
use core::marker::PhantomData;

const MAX_FRAME_COUNT: u64 = 1 << 26;
const FRAME_SIZE: u64 = 1 << 12;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct FrameNumber(u64);

impl FrameNumber {
    pub fn none() -> Self {
        FrameNumber(0)
    }

    pub fn is_none(self) -> bool {
        self.0 != 0
    }

    pub fn new(idx: u64) -> Self {
        FrameNumber(idx)
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }

    pub fn from_addr(addr: PhysAddr) -> Self {
        FrameNumber(addr.as_u64() << 12)
    }

    pub fn addr(self) -> PhysAddr {
        PhysAddr::new(self.0 >> 12)
    }

    pub fn from_frame(frame: PhysFrame<Size4KiB>) -> Self {
        Self::from_addr(frame.start_address())
    }

    pub fn frame(self) -> PhysFrame<Size4KiB> {
        PhysFrame::containing_address(self.addr())
    }
}

type FrameRangeVec = Vec<FrameRange, U16>;

struct Buddy<'a> {
    phys_addr_trans: &'a PhysAddrTranslator,
    free_head: FrameNumber,
    bitset: &'static BitSet<{ MAX_FRAME_COUNT }>,
}


impl Buddy<'_> {
    const BUDDY_FRAMES: u64 = 16;

    fn new<'a>(mgr: &mut FrameManager<'a>, page_table: &mut OffsetPageTable) -> Buddy<'a> {
        for i in 0..Self::BUDDY_FRAMES {
            let frame = mgr.alloc().expect("out of frames for buddy");
            page_table.map_to(Page::containing_address(VirtAddr::new(0)),
                              frame,
                              PageTableFlags::PRESENT,
                              &mut PagingFrameAllocator::new(mgr))
                .expect("unexpected map error")
                .flush();
        }

        let mut buddy = Buddy {
            phys_addr_trans: mgr.phys_addr_trans,
            free_head: FrameNumber::none(),
            bitset: unsafe { core::mem::zeroed() },
        };

        buddy.setup_free_links(&mgr.usable_range);
        buddy
    }

    fn setup_free_links(&mut self, usable_range: &FrameRangeVec) {
        assert!(!usable_range.is_empty(), "wtf?");
        self.free_head = FrameNumber::new(usable_range.first().unwrap().start_frame_number);

        let mut idx_next = FrameNumber::none().as_u64();
        for idx in usable_range.iter()
            .flat_map(|range| range.start_frame_number..range.end_frame_number)
            .rev() {
            let phys_addr = FrameNumber::new(idx).addr();
            let ptr_idx_ptr: *mut u64 = self.phys_addr_trans.translate(phys_addr).as_mut_ptr();
            unsafe { *ptr_idx_ptr = idx_next; }
            idx_next = idx;
        }
    }

    fn alloc(&mut self, two_power_frames: u64) -> Option<FrameNumber> {
        assert!(two_power_frames == 0, "not implemented yet");

        if self.free_head.is_none() {
            None
        } else {
            let idx = self.free_head.as_u64();
            //assert!(self.bitset.get(idx));
            //self.bitset.set(idx, false);

            let frame = self.free_head.frame();
            let next_idx_ptr: *mut u64 = self.phys_addr_trans.translate(frame.start_address()).as_mut_ptr();
            unsafe {
                self.free_head = FrameNumber::new(*next_idx_ptr);
                Some(FrameNumber::from_frame(frame))
            }
        }
    }

    fn dealloc(&mut self, two_power_frames: u64, start: FrameNumber) {
        assert!(two_power_frames == 0, "not implemented yet");

        let idx = start.as_u64();
        //assert!(!self.bitset.get(idx));
        //self.bitset.set(idx, true);

        let next_idx_ptr: *mut u64 = self.phys_addr_trans.translate(start.addr()).as_mut_ptr();
        unsafe {
            *next_idx_ptr = self.free_head.as_u64();
        }
        self.free_head = start;
        unimplemented!()
    }
}

pub struct FrameManager<'a> {
    phys_addr_trans: &'a PhysAddrTranslator,
    usable_range: FrameRangeVec,
    buddy: Option<Buddy<'a>>,
    // bitset: Option<&'static BitSet<{ MAX_FRAME_COUNT }>>, // true if available
}

trait FrameRangeExt {
    fn count(self) -> u64;
}

impl FrameRangeExt for FrameRange {
    fn count(self) -> u64 {
        self.end_frame_number - self.start_frame_number
    }
}

impl FrameManager<'_> {
    pub fn new<'a>(memory_map: &MemoryMap, phys_addr_trans: &'a PhysAddrTranslator, page_table: &mut OffsetPageTable) -> FrameManager<'a> {
        //panic!("boom");
        let mut mgr = FrameManager {
            // free_head: FrameNumber::none(),
            buddy: None,
            usable_range: Default::default(),
            phys_addr_trans,
        };

        for range in memory_map.iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .map(|r| r.range) {
            mgr.usable_range.push(range).expect("too much usable region!");
            // mgr.bitset.set_all(range.start_frame_number, range.end_frame_number, true);
        }

        mgr.buddy = Some(Buddy::new(&mut mgr, page_table));
        mgr
    }

//    fn setup_buddy(&mut self) {
//
//    }

    fn alloc_for_bitset(&mut self) {
        let demand_frames = MAX_FRAME_COUNT / 8 / FRAME_SIZE;
    }

    fn setup_free_bitset(&mut self) {}

    fn frame_in_usable_range(&self, frame: PhysFrame) -> bool {
        let idx = FrameNumber::from_frame(frame).as_u64();
        for range in self.usable_range.iter() {
            if (range.start_frame_number..range.end_frame_number).contains(&idx) { return true; };
        }
        false
    }

    pub fn alloc(&mut self) -> Option<UnusedPhysFrame<Size4KiB>> {
        if let Some(buddy) = &mut self.buddy {
            unimplemented!()
        } else {
            while let Some(range) = self.usable_range.last_mut() {
                if range.count() > 0 {
                    range.end_frame_number -= 1;
                    return Some(unsafe {
                        UnusedPhysFrame::new(FrameNumber::new(range.end_frame_number).frame())
                    });
                }
                if range.count() == 0 {
                    self.usable_range.pop();
                }
            }
            None
        }
    }

    pub fn dealloc(&mut self, unused_frame: UnusedPhysFrame<Size4KiB>) {
        if let Some(buddy) = &mut self.buddy {
            unimplemented!()
        } else {
            panic!("dealloc before buddy setup is meaningless")
        }
    }
}

pub struct PagingFrameAllocator<'u, 'v: 'u> {
    manager: &'u mut FrameManager<'v>,
}

impl PagingFrameAllocator<'_, '_> {
    pub fn new<'u, 'v: 'u>(manager: &'u mut FrameManager<'v>) -> PagingFrameAllocator<'u, 'v> {
        PagingFrameAllocator { manager }
    }
}

unsafe impl FrameAllocator<Size4KiB> for PagingFrameAllocator<'_, '_> {
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame<Size4KiB>> {
        self.manager.alloc()
    }
}