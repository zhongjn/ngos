mod text;

pub use text::*;

pub fn init() {
    // vga refresh
    crate::kernel::subscribe_timer(100_000_000, || {
        TEXT_WRITTER.lock().flush();
    });
}