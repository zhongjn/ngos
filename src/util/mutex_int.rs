use core::ops::{Deref, DerefMut};
use spin::{Mutex, MutexGuard};

#[derive(Debug)]
pub struct MutexGuardInt<'a, T> {
    enable_interrupt_later: bool,
    guard: MutexGuard<'a, T>,
}

pub struct MutexInt<T> {
    interruptible: bool,
    inner: Mutex<T>,
}

impl<T> MutexInt<T> {
    pub const fn new(interruptible: bool, data: T) -> Self {
        Self {
            interruptible,
            inner: Mutex::new(data),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        if self.interruptible {
            assert!(
                !crate::kernel::is_interrupt_context(),
                "cannot obtain interruptible lock in interrupt context"
            );
            MutexGuardInt {
                guard: self.lock(),
                enable_interrupt_later: false,
            }
        } else {
            let int_en = x86_64::instructions::interrupts::are_enabled();
            if int_en {
                x86_64::instructions::interrupts::disable();
            }
            MutexGuardInt {
                guard: self.lock(),
                enable_interrupt_later: int_en,
            }
        }
    }
}

// pub trait MutexIntExt<T> {
//     fn lock_uninterruptible(&self) -> MutexGuardInt<T>;
//     fn lock_interruptible(&self) -> MutexGuard<T>;
// }

// impl<T> MutexIntExt<T> for Mutex<T> {
//     fn lock_interruptible(&self) -> MutexGuard<T> {
//         assert!(
//             !crate::kernel::is_interrupt_context(),
//             "cannot obtain interruptible lock in interrupt context"
//         );
//         MutexGuardInt {
//             guard: self.lock(),
//             enable_interrupt_later: false,
//         }
//     }

//     fn lock_uninterruptible(&self) -> MutexGuardInt<T> {
//         let int_en = x86_64::instructions::interrupts::are_enabled();
//         if int_en {
//             x86_64::instructions::interrupts::disable();
//         }
//         MutexGuardInt {
//             guard: self.lock(),
//             enable_interrupt_later: int_en,
//         }
//     }
// }

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
