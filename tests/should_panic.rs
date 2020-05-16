#![no_std]
#![no_main]

use core::panic::PanicInfo;
use ngos::{exit_qemu, serial_print, serial_println, QemuExitCode};
use bootloader::BootInfo;

#[no_mangle]
pub extern "C" fn _start(_boot_info: &'static BootInfo) -> ! {
    should_fail();
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

fn should_fail() {
    serial_print!("should_fail... ");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}
