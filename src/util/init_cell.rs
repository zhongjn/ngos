use core::ops::Deref;
use core::cell::UnsafeCell;

struct InitInternal<T> {
    has_value: bool,
    value: T,
}

pub struct InitCell<T> {
    internal: UnsafeCell<Option<T>>
}

unsafe impl<T> Sync for InitCell<T> {}

impl<T> InitCell<T> {
    pub const fn new() -> Self {
        InitCell {
            internal: UnsafeCell::new(None)
        }
    }

    pub fn init(&'static self, value: T) {
        unsafe {
            let internal: &mut Option<T> = &mut *self.internal.get();
            assert!(internal.is_none());
            internal.replace(value);
        }
    }

    pub fn get(&self) -> &T {
        unsafe {
            let internal = &*self.internal.get();
            assert!(internal.is_some());
            internal.as_ref().unwrap()
        }
    }

    pub unsafe fn get_mut(&self) -> &mut T {
        let internal = &mut *self.internal.get();
        assert!(internal.is_some());
        internal.as_mut().unwrap()
    }
}

impl<T> Deref for InitCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}
