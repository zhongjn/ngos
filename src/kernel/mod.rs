mod int;
mod gdt;
mod time;
mod memory;
#[allow(dead_code)]
mod misc;

use time::get_real_time;
pub use time::subscribe_timer;
use bootloader::BootInfo;

pub fn init(boot_info: &'static BootInfo) {
    //    let _info = CallStackInfo::new("kernel::init");
    call_stack!();

    println!("not crashed");
    gdt::init();
    int::init();
    time::init();
    memory::init(boot_info.physical_memory_offset, &boot_info.memory_map);

    //test.borrow();
    //unsafe { *(0xdeadbeef as *mut u8) = 1; }
    //println!("not crashed");

}

pub fn start() {
    println!("running...");
    loop {}
}