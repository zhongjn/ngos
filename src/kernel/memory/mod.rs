use super::*;
use x86_64::structures::paging::*;
use x86_64::{VirtAddr, PhysAddr};
use x86_64::registers::control::*;
use core::ops::Deref;
use core::ops::DerefMut;
use spin::Mutex;
use core::cell::{UnsafeCell, Cell};
use crate::util::init_cell::InitCell;
use crate::util::bitset::BitSet;

mod frame_manager;

const KERNEL_VIRTUAL_START: u64 = 1 << 40;
const KERNEL_VIRTUAL_LENGTH: u64 = 1 << 32;


static PHYSICAL_MEMORY_OFFSET: InitCell<VirtAddr> = InitCell::new();
static OFFSET_PAGE_TABLE: InitCell<OffsetPageTable> = InitCell::new();

pub fn kmalloc(size: usize) -> *mut () {
    unimplemented!();
}

pub fn kfree(addr: *mut ()) {
    unimplemented!();
}

pub fn physical_to_virtual(addr: PhysAddr) -> VirtAddr {
    *PHYSICAL_MEMORY_OFFSET + addr.as_u64()
}

fn current_l4_page_table() -> &'static mut PageTable {
    unsafe {
        let (phys_frame, _flags) = Cr3::read();
        let virt_addr = physical_to_virtual(phys_frame.start_address());
        let pt: *mut PageTable = virt_addr.as_mut_ptr();
        &mut *pt
    }
}


struct DummyAllocator {}

unsafe impl FrameAllocator<Size4KiB> for DummyAllocator {
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame<Size4KiB>> {
        None
    }
}

fn test_create_map() {
    use x86_64::structures::paging::PageTableFlags as Flags;
    let page = Page::<Size4KiB>::containing_address(VirtAddr::new(0xdeadbeef));
    let frame = unsafe {
        UnusedPhysFrame::new(
            PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(0xb8000)))
    };
    let flags = Flags::PRESENT | Flags::WRITABLE;
    let mut allocator = DummyAllocator {};

    let result = unsafe { OFFSET_PAGE_TABLE.get_mut() }.map_to(page, frame, flags, &mut allocator);
    result.expect("map failed").flush();
}

pub fn init(physical_memory_offset: u64) {
    PHYSICAL_MEMORY_OFFSET.init(VirtAddr::new(physical_memory_offset));
    unsafe {
        OFFSET_PAGE_TABLE.init(OffsetPageTable::new(
            current_l4_page_table(),
            *PHYSICAL_MEMORY_OFFSET.get()));
    }


    let p = OFFSET_PAGE_TABLE.get().translate_addr(VirtAddr::new(0xdeafbeff));
    // test_create_map();
    println!("trans: {:?}", p);
    let pg = current_l4_page_table();
    for (i, entry) in pg.iter()
        .filter(|e| { !e.is_unused() })
        .enumerate() {
        println!("L4 {}: {:?}", i, entry);
    }

    println!("offset {}", physical_memory_offset);
//    let pg = current_l4_page_table();
//    for (i, entry) in pg.iter().take(20).enumerate() {
//        println!("L4 entry {}: {:?}", i, entry);
//        //sleep(1000000);
//    }
}