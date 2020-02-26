#![no_std]
#![no_main]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(custom_test_frameworks)]
#![feature(asm)]
#![feature(abi_x86_interrupt)]
#![feature(const_fn)]
#![feature(const_generics)]
#![feature(core_intrinsics)]

#[macro_use]
mod vga;

#[allow(dead_code)]
#[macro_use]
mod util;

mod serial;
mod kernel;


use vga::*;
use core::panic::PanicInfo;
use bootloader::BootInfo;
use crate::util::call_stack::CallStackInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("KERNEL PANIC! {}", info);
    CallStackInfo::print_all();
    TEXT_WRITTER.lock().flush();
    loop {}
}

//noinspection RsUnresolvedReference
#[cfg(test)]
fn test() {
    test_main();
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("running {} tests...", tests.len());
    for (i, test) in tests.iter().enumerate() {
        println!("test no.{}", i);
        test();
        println!("test no.{} - OK", i);
    }
}

#[test_case]
fn trivial_assertion() {
    println!("trivial assertion... ");
    assert_eq!(1, 1);
}

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    #[cfg(test)] test();

    kernel::init(boot_info);
    vga::init();

    kernel::start();

    print!("terminated");
    loop {}
}
