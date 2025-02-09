use core::ops::Range;
use x86_64::structures::paging::page::PageRange;
use x86_64::structures::paging::Page;
use x86_64::VirtAddr;

const KERNEL_VIRTUAL_START: u64 = 20 << 39;
const KERNEL_VIRTUAL_LENGTH: u64 = 4 << 39; // 2T
const USER_VIRTUAL_START: u64 = 16 << 39;
const USER_VIRTUAL_LENGTH: u64 = 4 << 39; // 2T

pub const fn kernel_virtual_range() -> Range<u64> {
    KERNEL_VIRTUAL_START..KERNEL_VIRTUAL_START + KERNEL_VIRTUAL_LENGTH
}

pub const fn user_virtual_range() -> Range<u64> {
    USER_VIRTUAL_START..USER_VIRTUAL_START + USER_VIRTUAL_LENGTH
}

pub struct AddrSpaceManager {
    kernel_alloc: u64,
}

impl AddrSpaceManager {
    pub fn new() -> AddrSpaceManager {
        Self { kernel_alloc: 0 }
    }

    pub fn user() -> PageRange {
        let begin = VirtAddr::new(USER_VIRTUAL_START);
        let end = begin + USER_VIRTUAL_LENGTH;
        PageRange {
            start: Page::containing_address(begin),
            end: Page::containing_address(end),
        }
    }

    pub fn kernel_alloc(&mut self, pages: u64) -> PageRange {
        let kernel_start = Page::containing_address(VirtAddr::new(KERNEL_VIRTUAL_START));
        let cur_start = kernel_start + self.kernel_alloc;
        let cur_end = cur_start + pages;
        assert!(cur_end.start_address() <= kernel_start.start_address() + KERNEL_VIRTUAL_LENGTH);
        self.kernel_alloc += pages;
        PageRange {
            start: cur_start,
            end: cur_end,
        }
    }
}
