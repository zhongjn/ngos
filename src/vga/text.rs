const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const VGA_ADDR: *mut u8 = 0xb8000 as *mut u8;

use lazy_static::*;
use spin::Mutex;
use core::fmt::{Arguments, Error, Write};
use core::cell::{Cell, RefCell};

lazy_static! {
    pub static ref TEXT_WRITTER: Mutex<TextWriter> = Mutex::new(TextWriter::default());
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        TEXT_WRITTER.lock().write_fmt(args).unwrap();
    });
}

struct VGATextAdapter {
    data: [[u8; BUFFER_WIDTH]; BUFFER_HEIGHT],
    underline: (usize, usize),
}

impl Default for VGATextAdapter {
    fn default() -> Self {
        VGATextAdapter {
            data: unsafe { core::mem::zeroed() },
            underline: (0, 0),
        }
    }
}

impl VGATextAdapter {
    pub fn refresh_vga(&self) {
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let offset: isize = (row * BUFFER_WIDTH + col) as isize * 2;
                let blink_flag: u8 = ((row, col) != self.underline) as u8;
                unsafe {
                    *VGA_ADDR.offset(offset) = self.data[row][col];
                    *VGA_ADDR.offset(offset + 1) = 0xf;
                }
            }
        }
    }
}

#[doc(hidden)]
pub struct TextWriter {
    text_buf: VGATextAdapter,
    row: usize,
    col: usize,
}

impl Default for TextWriter {
    fn default() -> Self {
        TextWriter {
            text_buf: VGATextAdapter::default(),
            row: 0,
            col: 0,
        }
    }
}

impl TextWriter {
    pub fn flush(&mut self) {
        self.text_buf.underline = (self.row, self.col);
        self.text_buf.refresh_vga();
    }

    fn scroll_down(&mut self) {
        for row in 0..BUFFER_HEIGHT - 1 {
            for col in 0..BUFFER_WIDTH {
                self.text_buf.data[row][col] = self.text_buf.data[row + 1][col];
            }
        }
        for col in 0..BUFFER_WIDTH {
            self.text_buf.data[BUFFER_HEIGHT - 1][col] = 0;
        }
        self.row -= 1;
    }

    pub fn write_ascii(&mut self, ch: u8) {
        match ch {
            10 => self.new_line(), // LF
            32 => self.write_ascii(0), // space
            _ => {
                if self.col >= BUFFER_WIDTH {
                    self.new_line();
                }
                self.text_buf.data[self.row][self.col] = ch;
                self.col += 1;
            }
        }
    }

    fn new_line(&mut self) {
        self.col = 0;
        self.row += 1;
        if self.row >= BUFFER_HEIGHT {
            self.scroll_down();
        }
    }
}

impl Write for TextWriter {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for &ch in s.as_bytes() {
            self.write_ascii(ch);
        }
        // self.flush();
        Ok(())
    }
}

