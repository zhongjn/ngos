use crate::util::{init_cell::InitCell, mutex_int::MutexInt};
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};
use linked_list_allocator::Heap;

struct HeapWrapper {
    inner: InitCell<MutexInt<Heap>>,
}

unsafe impl GlobalAlloc for HeapWrapper {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.inner
            .lock()
            .allocate_first_fit(layout)
            .ok()
            .map_or(0 as *mut u8, |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner
            .lock()
            .deallocate(NonNull::new_unchecked(ptr), layout)
    }
}

#[global_allocator]
static KERNEL_HEAP: HeapWrapper = HeapWrapper {
    inner: InitCell::new(),
};
const KERNEL_HEAP_SIZE: u64 = 1 << 36; // 16G

pub fn init() {
    crate::call_stack!();
    let pages = KERNEL_HEAP_SIZE >> 12;
    let page_range = super::ADDR_SPACE_MANAGER.lock().kernel_alloc(pages);

    let heap = unsafe {
        Heap::new(
            page_range.start.start_address().as_u64() as usize,
            KERNEL_HEAP_SIZE as usize,
        )
    };
    KERNEL_HEAP.inner.init(MutexInt::new(false, heap));
}

#[cfg(test)]
use alloc::{boxed::Box, vec::Vec};

#[test_case]
fn few_box() {
    let b1 = Box::new(1);
    let mut b2 = Box::new(2);
    *b2.as_mut() = 3;
    assert_eq!(*b1, 1);
    assert_eq!(*b2, 3);
}

#[test_case]
fn reuse_box() {
    let b0 = Box::new(100000);
    for i in 0..10000 {
        let b = Box::new(i);
        assert_eq!(*b, i);
    }
    assert_eq!(*b0, 100000);
}

#[test_case]
fn large_vec() {
    let mut v = Vec::new();
    for i in 0..10000 {
        v.push(i);
    }
    assert_eq!(v.len(), 10000);
    for i in 0..10000 {
        assert_eq!(*v.get(i).unwrap(), i);
    }
}
