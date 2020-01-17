use super::*;
use heapless::Vec;
use heapless::consts::U64;
use spin::Mutex;
use crate::util::init_cell::InitCell;

static mut REAL_TIME: u64 = 0;

struct TimerSubscription {
    last_trigger_time: u64,
    interval: u64,
    event_handler: fn(),
}

static TIMER_EVENT_HANDLERS: InitCell<Vec<TimerSubscription, U64>> = InitCell::new();

pub fn get_real_time() -> u64 {
    unsafe {
        let th: u32;
        let tl: u32;
        asm!("rdtsc" : "={eax}"(tl), "={edx}"(th) : : "edx eax" : "intel");
        REAL_TIME = ((th as u64) << 32) | (tl as u64);
        REAL_TIME
    }
}

pub fn subscribe_timer(interval: u64, event_handler: fn()) {
    unsafe {
        TIMER_EVENT_HANDLERS.get_mut().push(TimerSubscription {
            interval,
            event_handler,
            last_trigger_time: unsafe { REAL_TIME }
        });
    }
}

pub fn timer_event_handler() {
    let rt = get_real_time();
    unsafe {
        for sub in TIMER_EVENT_HANDLERS.get_mut().iter_mut() {
            if rt - sub.last_trigger_time >= sub.interval {
                let handler = sub.event_handler;
                handler();
                sub.last_trigger_time = rt;
            }
        }
    }
}

pub fn init() {
    TIMER_EVENT_HANDLERS.init(Vec::new());
}