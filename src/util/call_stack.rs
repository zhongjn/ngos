use heapless::Vec;
use heapless::consts::U128;
use core::cell::UnsafeCell;
use super::constant::Constant;
use lazy_static::*;

#[macro_export]
macro_rules! call_stack {
    () => {
        let _call = $crate::CallStackInfo::new(function!());
    }
}

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            core::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }}
}


lazy_static! {
    static ref STACK: Constant<UnsafeCell<Vec<&'static str, U128>>>
        = Default::default();
}

pub struct CallStackInfo {
    in_stack: bool,
    message: &'static str
}

impl CallStackInfo {
    pub fn new(message: &'static str) -> CallStackInfo {
        let stack = unsafe { &mut *STACK.get() };
        CallStackInfo { message, in_stack: stack.push(message).is_ok() }
    }

    pub fn print_all() {
        println!("[CALL STACK]");
        let stack = unsafe { &*STACK.get() };
        for (i, msg) in stack.iter().enumerate().rev() {
            println!("{}: {}", i, msg);
        }
    }
}

impl Drop for CallStackInfo {
    fn drop(&mut self) {
        if self.in_stack {
            let stack = unsafe { &mut *STACK.get() };
            stack.pop();
        }
    }
}