use bootloader::bootinfo::MemoryMap;
use spin::Mutex;
use x86_64::{VirtAddr, registers::control::*, structures::paging::*};

use frame::*;
use phys_addr_trans::*;
use addr_space::*;

use crate::util::{init_cell::InitCell, mutex_int::MutexIntExt};
use core::ops::DerefMut;

mod frame;
mod phys_addr_trans;
mod addr_space;

static ADDR_SPACE_MANAGER: InitCell<Mutex<AddrSpaceManager>> = InitCell::new();
static PHYS_ADDR_TRANSLATOR: InitCell<PhysAddrTranslator> = InitCell::new();
static OFFSET_PAGE_TABLE: InitCell<Mutex<OffsetPageTable>> = InitCell::new();
static FRAME_MANAGER: InitCell<Mutex<FrameManager>> = InitCell::new();

fn current_l4_page_table() -> &'static mut PageTable {
    unsafe {
        let (phys_frame, _flags) = Cr3::read();
        let virt_addr = PHYS_ADDR_TRANSLATOR.translate(phys_frame.start_address());
        let pt: *mut PageTable = virt_addr.as_mut_ptr();
        &mut *pt
    }
}

// fn test_create_map() {
//     use x86_64::structures::paging::PageTableFlags as Flags;
//     let page = Page::<Size4KiB>::containing_address(VirtAddr::new(0xdeadbeef));
//     let frame = unsafe {
//         UnusedPhysFrame::new(
//             PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(0xb8000)))
//     };
//     let flags = Flags::PRESENT | Flags::WRITABLE;
// }

pub fn init(physical_memory_offset: u64, memory_map: &'static MemoryMap) {
    //let _info = CallStackInfo::new("kernel::memory::init");
    call_stack!();

    let physical_memory_offset = VirtAddr::new(physical_memory_offset);
    PHYS_ADDR_TRANSLATOR.init(PhysAddrTranslator::new(physical_memory_offset));

    ADDR_SPACE_MANAGER.init(Mutex::new(AddrSpaceManager::new()));

//    for reg in memory_map.iter() {
//        println!("{:?}", reg);
//    }
//
    for (i, entry) in current_l4_page_table().iter().enumerate() {
        if !entry.is_unused() {
            println!("{}: {:?}", i, entry);
        }
    }
    //panic!();
    OFFSET_PAGE_TABLE.init(Mutex::new(unsafe {
        OffsetPageTable::new(
            current_l4_page_table(),
            physical_memory_offset)
    }));

    FRAME_MANAGER.init(Mutex::new(FrameManager::new(
        memory_map,
        OFFSET_PAGE_TABLE.lock_int().deref_mut())));

    for _i in 0..10 {
        let f = FRAME_MANAGER.get().lock().alloc(0);
        println!("{:?}", f);
    }

    // panic!("done");
    let pat = PHYS_ADDR_TRANSLATOR.get();
    let ptr: *const _ = &pat;
    let addr = OFFSET_PAGE_TABLE.lock_int().translate_addr(VirtAddr::new(ptr as u64)).unwrap();
    println!("STACK ADDR: {:?}", addr);
    //return;

//    panic!("wtf");
//    let pat = PHYS_ADDR_TRANSLATOR.get();
//    panic!("pat");
//    let fm = FrameManager::new(memory_map, pat);
//    panic!("f");
//    FRAME_MANAGER.init(fm);

//    println!("hello");
    //let f = unsafe { FRAME_MANAGER.get_mut() }.alloc();
    //println!("{:?}", f);

//    let p = OFFSET_PAGE_TABLE.get().translate_addr(VirtAddr::new(0xdeafbeef));
//    // test_create_map();
//    println!("trans: {:?}", p);
//    let pg = current_l4_page_table();
//    for (i, entry) in pg.iter()
//        .filter(|e| { !e.is_unused() })
//        .enumerate() {
//        println!("L4 {}: {:?}", i, entry);
//    }
}