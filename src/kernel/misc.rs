use crate::kernel::time::get_real_time;

pub fn halt_loop() {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn sleep(time: u64) {
    let before = get_real_time();
    loop {
        x86_64::instructions::hlt();
        let current = get_real_time();
        if current - before >= time {
            break;
        }
    }
}

pub struct BenchmarkHandle {
    start_time: u64
}

impl BenchmarkHandle {
    pub fn time(&self) -> u64 {
        return get_real_time() - self.start_time;
    }
    pub fn print(&self) {
        println!("time={}", self.time());
    }
}

pub fn benchmark() -> BenchmarkHandle {
    BenchmarkHandle { start_time: get_real_time() }
}