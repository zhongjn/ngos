use core::{cell::UnsafeCell, mem::MaybeUninit, ops::Deref};

struct InitCellInner<T> {
    init: bool,
    value: MaybeUninit<T>,
}

pub struct InitCell<T> {
    inner: UnsafeCell<InitCellInner<T>>,
}

unsafe impl<T: Sync> Sync for InitCell<T> {}

impl<T> InitCell<T> {
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(InitCellInner {
                init: false,
                value: MaybeUninit::uninit(),
            }),
        }
    }

    pub fn init(&'static self, value: T) {
        unsafe {
            let inner = &mut *self.inner.get();
            assert!(!inner.init);
            *inner.value.as_mut_ptr() = value;
            inner.init = true;
        }
    }

    pub unsafe fn init_in_place(&'static self, f: impl FnOnce(*mut T)) {
        let inner = &mut *self.inner.get();
        assert!(!inner.init);
        f(inner.value.as_mut_ptr());
        inner.init = true;
    }

    pub fn get(&self) -> &T {
        unsafe {
            let inner = &mut *self.inner.get();
            assert!(inner.init);
            &*inner.value.as_ptr()
        }
    }
}

impl<T> Default for InitCell<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for InitCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}
