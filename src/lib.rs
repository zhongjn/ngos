
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(core_intrinsics)]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

#[macro_use]
pub mod vga;
#[allow(dead_code)]
#[macro_use]
pub mod util;
pub mod serial;
#[allow(dead_code)]
pub mod kernel;
pub mod logger;

use core::panic::PanicInfo;
#[cfg(test)]
use bootloader::BootInfo;

pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("running {} tests...", tests.len());
    for (i, test) in tests.iter().enumerate() {
        serial_println!("test no.{}", i);
        test();
        serial_println!("test no.{} - OK", i);
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    use crate::util::call_stack::CallStackInfo;
    println!("KERNEL PANIC! {}", info);
    serial_println!("KERNEL PANIC! {}", info);
    {
        let mut writer = vga::TEXT_WRITER.lock();
        CallStackInfo::print_all(&mut *writer);
        writer.flush();
    }
    {
        let mut writer = serial::SERIAL1.lock();
        CallStackInfo::print_all(&mut *writer);
    }

    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    kernel::init(boot_info);
    vga::init_non_core();

    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}