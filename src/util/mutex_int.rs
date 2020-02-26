use spin::{Mutex, MutexGuard};

#[derive(Debug)]
pub struct MutexGuardInt<'a, T: ?Sized + 'a> {
    guard: MutexGuard<'a, T>
}

pub trait MutexIntExt {
    fn lock_int() -> MutexGuardInt {
        
    }
}

impl<T> MutexIntExt for Mutex<T> {

}