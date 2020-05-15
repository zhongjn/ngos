use core::ops::{Deref, DerefMut};
use spin::{Mutex, MutexGuard};

#[derive(Debug)]
pub struct MutexGuardInt<'a, T> {
    interrupt_enabled_before: bool,
    guard: MutexGuard<'a, T>,
}

pub trait MutexIntExt<T> {
    fn lock_int(&self) -> MutexGuardInt<T>;
}

impl<T> MutexIntExt<T> for Mutex<T> {
    fn lock_int(&self) -> MutexGuardInt<T> {
        let int_en = x86_64::instructions::interrupts::are_enabled();
        if int_en {
            x86_64::instructions::interrupts::disable();
        }
        MutexGuardInt {
            interrupt_enabled_before: int_en,
            guard: self.lock(),
        }
    }
}

impl<T> Drop for MutexGuardInt<'_, T> {
    fn drop(&mut self) {
        if self.interrupt_enabled_before {
            x86_64::instructions::interrupts::enable();
        }
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
