// #![no_std]
// #![no_main]
// #![feature(custom_test_frameworks)]
// #![test_runner(ngos::test_runner)]
// #![reexport_test_harness_main = "test_main"]

// #[macro_use]
// extern crate ngos;

// use bootloader::BootInfo;
// use core::panic::PanicInfo;


// #[cfg(test)]
// #[panic_handler]
// fn panic(info: &PanicInfo) -> ! {
//     ngos::test_panic_handler(info)
// }

// #[cfg(not(test))]
// #[panic_handler]
// fn panic(info: &PanicInfo) -> ! {
//     use ngos::util::call_stack::CallStackInfo;
//     println!("KERNEL PANIC! {}", info);
//     serial_println!("KERNEL PANIC! {}", info);
//     {
//         let mut writer = ngos::vga::TEXT_WRITER.lock();
//         CallStackInfo::print_all(&mut *writer);
//         writer.flush();
//     }
//     {
//         let mut writer = ngos::serial::SERIAL1.lock();
//         CallStackInfo::print_all(&mut *writer);
//     }

//     loop {}
// }

// #[cfg(not(test))]
// #[no_mangle]
// pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
//     ngos::kernel::init(boot_info);
//     ngos::vga::init();

//     #[cfg(test)]
//     test_main();

//     ngos::kernel::start();

//     print!("terminated");
//     loop {}
// }
