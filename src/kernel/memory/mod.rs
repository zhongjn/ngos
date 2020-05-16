use bootloader::bootinfo::MemoryMap;
use x86_64::{
    registers::control::*,
    structures::{
        idt::{InterruptStackFrame, PageFaultErrorCode},
        paging::*,
    },
    VirtAddr,
};

use addr_space::*;
use frame::*;
use phys_addr_trans::*;

use crate::util::{init_cell::InitCell, mutex_int::{MutexInt, MutexIntExt}};
use core::ops::DerefMut;

mod addr_space;
mod frame;
mod phys_addr_trans;
mod allocator;

static PHYS_ADDR_TRANSLATOR: InitCell<PhysAddrTranslator> = InitCell::new();
static ADDR_SPACE_MANAGER: InitCell<MutexInt<AddrSpaceManager>> = InitCell::new();
static OFFSET_PAGE_TABLE: InitCell<MutexInt<OffsetPageTable>> = InitCell::new();
static FRAME_MANAGER: InitCell<MutexInt<FrameManager>> = InitCell::new();

fn current_l4_page_table() -> &'static mut PageTable {
    unsafe {
        let (phys_frame, _flags) = Cr3::read();
        let virt_addr = PHYS_ADDR_TRANSLATOR.translate(phys_frame.start_address());
        let pt: *mut PageTable = virt_addr.as_mut_ptr();
        &mut *pt
    }
}

pub fn init(physical_memory_offset: u64, memory_map: &'static MemoryMap) {
    crate::call_stack!();

    for reg in memory_map.iter() {
        println!("{:?}", reg);
    }

    let physical_memory_offset = VirtAddr::new(physical_memory_offset);
    PHYS_ADDR_TRANSLATOR.init(PhysAddrTranslator::new(physical_memory_offset));

    ADDR_SPACE_MANAGER.init(Mutex::new(AddrSpaceManager::new()));

    for (i, entry) in current_l4_page_table().iter().enumerate() {
        if !entry.is_unused() {
            println!("{}: {:?}", i, entry);
        }
    }
    //panic!();
    OFFSET_PAGE_TABLE.init(Mutex::new(unsafe {
        OffsetPageTable::new(current_l4_page_table(), physical_memory_offset)
    }));

    FRAME_MANAGER.init(Mutex::new(FrameManager::new(
        memory_map,
        OFFSET_PAGE_TABLE.lock_uninterruptible().deref_mut(),
    )));

    // for _i in 0..10 {
    //     let f = FRAME_MANAGER.get().lock().alloc(0);
    //     println!("{:?}", f);
    // }

    // // panic!("done");
    // let pat = PHYS_ADDR_TRANSLATOR.get();
    // let ptr: *const _ = &pat;
    // let addr = OFFSET_PAGE_TABLE
    //     .lock_int()
    //     .translate_addr(VirtAddr::new(ptr as u64))
    //     .unwrap();
    // println!("STACK ADDR: {:?}", addr);
}

pub fn do_page_fault(
    addr: VirtAddr,
    _stack_frame: &mut InterruptStackFrame,
    err: PageFaultErrorCode,
) {
    if err.contains(PageFaultErrorCode::USER_MODE) {
        unimplemented!();
    } else {
        if !kernel_virtual_range().contains(&addr.as_u64()) {
            panic!("kernel mode page fault, address={:x}", addr.as_u64());
        }

        let mut page_table = OFFSET_PAGE_TABLE.lock_uninterruptible();
        let mut frame_manager = FRAME_MANAGER.lock_uninterruptible();
        let mut frame_allocator = PagingFrameAllocator::new(&mut *frame_manager);
        let page = Page::<Size4KiB>::containing_address(addr);
        let frame = frame_allocator
            .allocate_frame()
            .expect("out of physical memory");
        log::trace!("mapping page {:?} to frame {:?}", page, frame);
        unsafe {
            page_table
                .map_to(
                    page,
                    frame,
                    PageTableFlags::WRITABLE | PageTableFlags::PRESENT,
                    &mut frame_allocator,
                )
                .expect("failed to map virtual memory")
                .flush();
        }
    }
}
