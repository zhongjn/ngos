use core::ops::{Deref, DerefMut};
use spin::{Mutex, MutexGuard};

#[derive(Debug)]
pub struct MutexGuardInt<'a, T> {
    enable_interrupt_later: bool,
    guard: MutexGuard<'a, T>,
}

pub struct MutexInt<T> {
    allow_interrupt_context: bool,
    inner: Mutex<T>,
}

impl<T> MutexInt<T> {
    pub const fn new(allow_interrupt_context: bool, data: T) -> Self {
        Self {
            allow_interrupt_context,
            inner: Mutex::new(data),
        }
    }

    pub fn lock(&self) -> MutexGuardInt<T> {
        if !self.allow_interrupt_context {
            assert!(
                !crate::kernel::is_interrupt_context(),
                "does not allow interrupt context"
            );
            MutexGuardInt {
                guard: self.inner.lock(),
                enable_interrupt_later: false,
            }
        } else {
            let int_en = x86_64::instructions::interrupts::are_enabled();
            if int_en {
                x86_64::instructions::interrupts::disable();
            }
            MutexGuardInt {
                guard: self.inner.lock(),
                enable_interrupt_later: int_en,
            }
        }
    }
}

impl<T> Drop for MutexGuardInt<'_, T> {
    fn drop(&mut self) {
        if self.enable_interrupt_later {
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
