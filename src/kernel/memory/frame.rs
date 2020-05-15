use super::{ADDR_SPACE_MANAGER, PHYS_ADDR_TRANSLATOR};
use crate::util::{bit_set::BitSet, default_in_place::DefaultInPlace, mutex_int::MutexIntExt};
use bootloader::bootinfo::{FrameRange, MemoryMap, MemoryRegionType};
use core::intrinsics::size_of;
use heapless::consts::U16;
use heapless::Vec;
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, PageTableFlags, PhysFrame, Size4KiB,
    },
    PhysAddr,
};

const MAX_FRAME_COUNT_USIZE: usize = 1 << 26; // 64M frames
const MAX_FRAME_COUNT: u64 = MAX_FRAME_COUNT_USIZE as u64;
const FRAME_SIZE: u64 = 1 << 12; // 4K per frame

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct FrameNumber(u64);

impl FrameNumber {
    pub fn none() -> Self {
        FrameNumber(0)
    }

    pub fn is_none(self) -> bool {
        self.0 != 0
    }

    pub fn from_u64(idx: u64) -> Self {
        FrameNumber(idx)
    }

    pub fn into_u64(self) -> u64 {
        self.0
    }

    pub fn from_addr(addr: PhysAddr) -> Self {
        FrameNumber(addr.as_u64() >> 12)
    }

    pub fn into_addr(self) -> PhysAddr {
        PhysAddr::new(self.0 << 12)
    }

    pub fn from_frame(frame: PhysFrame<Size4KiB>) -> Self {
        FrameNumber::from_addr(frame.start_address())
    }

    pub fn into_frame(self) -> PhysFrame<Size4KiB> {
        PhysFrame::containing_address(self.into_addr())
    }
}

type FrameRangeVec = Vec<FrameRange, U16>;

struct BuddyStorage {
    order0_free: [u8; (MAX_FRAME_COUNT_USIZE + 7) / 8],
}

impl DefaultInPlace for BuddyStorage {
    unsafe fn default_in_place(s: *mut Self) {
        let arr = &mut (*s).order0_free;
        for byte in arr.iter_mut() {
            *byte = !0;
        }
    }
}

struct Buddy {
    free_head: FrameNumber,
    storage: &'static mut BuddyStorage,
}

impl Buddy {
    fn new(mgr: &mut FrameManager, page_table: &mut OffsetPageTable) -> Buddy {
        let pages = num::integer::div_ceil(size_of::<BuddyStorage>() as u64, FRAME_SIZE);
        let page_range = ADDR_SPACE_MANAGER.get().lock_int().kernel_alloc(pages);

        for i in 0..pages {
            let frame = mgr.alloc(0).expect("out of frames for buddy").into_frame();
            unsafe {
                page_table
                    .map_to(
                        page_range.start + i,
                        frame,
                        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                        &mut PagingFrameAllocator::new(mgr),
                    )
                    .expect("unexpected map error")
                    .flush();
            }
        }

        let storage_ptr: *mut BuddyStorage = page_range.start.start_address().as_mut_ptr();
        unsafe {
            DefaultInPlace::default_in_place(storage_ptr);
        };

        let mut buddy = Buddy {
            free_head: FrameNumber::none(),
            storage: unsafe { &mut *storage_ptr },
        };

        buddy.setup_free_links(&mgr.usable_range);
        buddy
    }

    fn setup_free_links(&mut self, usable_range: &FrameRangeVec) {
        assert!(!usable_range.is_empty(), "wtf?");
        self.free_head = FrameNumber::from_u64(usable_range.first().unwrap().start_frame_number);

        let mut idx_next = FrameNumber::none().into_u64();
        for idx in usable_range
            .iter()
            .flat_map(|range| range.start_frame_number..range.end_frame_number)
            .rev()
        {
            let phys_addr = FrameNumber::from_u64(idx).into_addr();
            let ptr_idx_ptr: *mut u64 =
                PHYS_ADDR_TRANSLATOR.get().translate(phys_addr).as_mut_ptr();
            unsafe {
                *ptr_idx_ptr = idx_next;
            }
            idx_next = idx;
        }
    }

    fn alloc(&mut self, order: u8) -> Option<FrameNumber> {
        assert!(order == 0, "not implemented yet");

        if self.free_head.is_none() {
            None
        } else {
            let idx = self.free_head.into_u64();
            let mut order0_free = BitSet::new(MAX_FRAME_COUNT, &mut self.storage.order0_free);
            assert!(order0_free.get(idx));
            order0_free.set(idx, false);

            let frame = self.free_head.into_frame();
            let next_idx_ptr: *mut u64 = PHYS_ADDR_TRANSLATOR
                .get()
                .translate(frame.start_address())
                .as_mut_ptr();
            unsafe {
                self.free_head = FrameNumber::from_u64(*next_idx_ptr);
                Some(FrameNumber::from_frame(frame))
            }
        }
    }

    fn dealloc(&mut self, order: u8, start: FrameNumber) {
        assert!(order == 0, "not implemented yet");

        let idx = start.into_u64();
        let mut order0_used = BitSet::new(MAX_FRAME_COUNT, &mut self.storage.order0_free);
        assert!(order0_used.get(idx));
        order0_used.set(idx, false);

        let next_idx_ptr: *mut u64 = PHYS_ADDR_TRANSLATOR
            .get()
            .translate(start.into_addr())
            .as_mut_ptr();
        unsafe {
            *next_idx_ptr = self.free_head.into_u64();
        }
        self.free_head = start;
        unimplemented!()
    }
}

pub struct FrameManager {
    usable_range: FrameRangeVec,
    buddy: Option<Buddy>,
}

trait FrameRangeExt {
    fn count(self) -> u64;
}

impl FrameRangeExt for FrameRange {
    fn count(self) -> u64 {
        self.end_frame_number - self.start_frame_number
    }
}

impl FrameManager {
    pub fn new(memory_map: &MemoryMap, page_table: &mut OffsetPageTable) -> FrameManager {
        let mut mgr = FrameManager {
            buddy: None,
            usable_range: Default::default(),
        };

        for range in memory_map
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .map(|r| r.range)
        {
            mgr.usable_range
                .push(range)
                .expect("too much usable region!");
        }

        mgr.buddy = Some(Buddy::new(&mut mgr, page_table));
        mgr
    }

    // fn frame_in_usable_range(&self, frame: PhysFrame) -> bool {
    //     let idx = FrameNumber::from_frame(frame).as_u64();
    //     for range in self.usable_range.iter() {
    //         if (range.start_frame_number..range.end_frame_number).contains(&idx) { return true; };
    //     }
    //     false
    // }

    pub fn alloc(&mut self, order: u8) -> Option<FrameNumber> {
        if let Some(buddy) = &mut self.buddy {
            buddy.alloc(order)
        } else {
            assert!(order == 0, "only support order 0 alloc before buddy setup");
            while let Some(range) = self.usable_range.last_mut() {
                if range.count() > 0 {
                    range.end_frame_number -= 1;
                    return Some(FrameNumber::from_u64(range.end_frame_number));
                }
                if range.count() == 0 {
                    self.usable_range.pop();
                }
            }
            None
        }
    }

    pub fn dealloc(&mut self, _order: u8, start_frame: FrameNumber) {
        if let Some(buddy) = &mut self.buddy {
            buddy.dealloc(0, start_frame);
        } else {
            panic!("dealloc before buddy setup is meaningless")
        }
    }
}

pub struct PagingFrameAllocator<'u> {
    manager: &'u mut FrameManager,
}

impl PagingFrameAllocator<'_> {
    pub fn new(manager: &mut FrameManager) -> PagingFrameAllocator {
        PagingFrameAllocator { manager }
    }
}

unsafe impl FrameAllocator<Size4KiB> for PagingFrameAllocator<'_> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        self.manager.alloc(0).map(|n| n.into_frame())
    }
}
