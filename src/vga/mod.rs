#[macro_use]
mod text;

pub use text::*;

pub fn init() {
    // vga refresh
    crate::kernel::subscribe_timer(10_000_000, || {
        TEXT_WRITTER.lock().flush();
    });
}