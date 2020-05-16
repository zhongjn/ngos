#[macro_use]
mod text;

pub use text::*;

pub fn init_non_core() {
    crate::call_stack!();
    // vga refresh
    crate::kernel::subscribe_timer(10_000_000, || {
        TEXT_WRITER.lock().flush();
    });
}