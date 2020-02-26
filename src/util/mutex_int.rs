use spin::{Mutex, MutexGuard};
use core::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct MutexGuardInt<'a, T> {
    guard: MutexGuard<'a, T>
}

pub trait MutexIntExt<T> {
    fn lock_int(&self) -> MutexGuardInt<T>;
}

impl<T> MutexIntExt<T> for Mutex<T> {
    fn lock_int(&self) -> MutexGuardInt<T> {
        x86_64::instructions::interrupts::disable();
        MutexGuardInt { guard: self.lock() }
    }
}

impl<T> Drop for MutexGuardInt<'_, T> {
    fn drop(&mut self) {
        x86_64::instructions::interrupts::enable();
    }
}

impl<T> Deref for MutexGuardInt<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

impl<T> DerefMut for MutexGuardInt<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.deref_mut()
    }
}