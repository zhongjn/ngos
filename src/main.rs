#![no_std]
#![no_main]
#![feature(asm)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![feature(const_fn)]
#![feature(const_generics)]
#![feature(core_intrinsics)]

mod vga;
mod serial;
mod kernel;
mod util;

use vga::*;
use core::panic::PanicInfo;
use bootloader::BootInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("KERNEL PANIC! {}", info);
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
