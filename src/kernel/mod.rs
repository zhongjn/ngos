mod int;
mod gdt;
mod time;
mod memory;
mod misc;

// use time::get_real_time;
use bootloader::BootInfo;
pub use time::subscribe_timer;

pub fn init(boot_info: &'static BootInfo) {
    crate::call_stack!();

    println!("not crashed");
    gdt::init();
    int::init();
    time::init();
    memory::init(boot_info.physical_memory_offset, &boot_info.memory_map);
}

pub fn start() {
    println!("running...");
    loop {}
}