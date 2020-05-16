use crate::util::init_cell::InitCell;
use crate::util::mutex_int::MutexIntExt;
use heapless::consts::U64;
use heapless::Vec;
use spin::Mutex;

static mut REAL_TIME: u64 = 0;

#[derive(Debug)]
struct TimerSubscription {
    last_trigger_time: u64,
    interval: u64,
    event_handler: fn(),
}

static TIMER_EVENT_HANDLERS: InitCell<Mutex<Vec<TimerSubscription, U64>>> = InitCell::new();

pub fn get_real_time() -> u64 {
    unsafe {
        let th: u32;
        let tl: u32;
        llvm_asm!("rdtsc" : "={eax}"(tl), "={edx}"(th) : : "edx eax" : "intel");
        REAL_TIME = ((th as u64) << 32) | (tl as u64);
        REAL_TIME
    }
}

pub fn subscribe_timer(interval: u64, event_handler: fn()) {
    TIMER_EVENT_HANDLERS
        .lock_interruptible()
        .push(TimerSubscription {
            interval,
            event_handler,
            last_trigger_time: unsafe { REAL_TIME },
        })
        .expect("too much timer subs!");
}

pub fn timer_event_handler() {
    let rt = get_real_time();
    for sub in TIMER_EVENT_HANDLERS.lock_uninterruptible().iter_mut() {
        if rt - sub.last_trigger_time >= sub.interval {
            let handler = sub.event_handler;
            handler();
            sub.last_trigger_time = rt;
        }
    }
}

pub fn init() {
    TIMER_EVENT_HANDLERS.init(Mutex::default());
}
