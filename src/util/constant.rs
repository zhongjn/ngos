use core::ops::Deref;

pub struct Constant<T> {
    val: T
}

impl<T> Constant<T> {
    pub fn new(val: T) -> Self {
        Self { val }
    }

    pub fn inner(&self) -> &T {
        &self.val
    }
}

impl<T> Deref for Constant<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

unsafe impl<T> Sync for Constant<T> {}

impl<T> Default for Constant<T> where T: Default {
    fn default() -> Self {
        Self { val: Default::default() }
    }
}

impl<T> From<T> for Constant<T> {
    fn from(t: T) -> Self {
        Self::new(t)
    }
}