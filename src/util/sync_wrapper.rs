use core::ops::Deref;

pub struct SyncWrapper<T> {
    val: T
}

impl<T> SyncWrapper<T> {
    pub fn new(val: T) -> Self {
        Self { val }
    }

    pub fn inner(&self) -> &T {
        &self.val
    }
}

impl<T> Deref for SyncWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

unsafe impl<T> Sync for SyncWrapper<T> {}

impl<T> Default for SyncWrapper<T> where T: Default {
    fn default() -> Self {
        Self { val: Default::default() }
    }
}