use super::gdt;
use crate::{kernel::time::timer_event_handler, util::mutex_int::MutexIntExt};
use core::sync::atomic::{AtomicBool, Ordering};
use lazy_static::*;
use pic8259_simple::ChainedPics;
use spin;
use x86_64::structures::idt::*;

static IS_INTERRUPT_CONTEXT: AtomicBool = AtomicBool::new(false);

pub fn is_interrupt_context() -> bool {
    IS_INTERRUPT_CONTEXT.load(Ordering::Relaxed)
}

struct InterruptContextHandle {
    _private: (),
}

impl InterruptContextHandle {
    fn new() -> Self {
        IS_INTERRUPT_CONTEXT.store(true, Ordering::Relaxed);
        Self { _private: () }
    }
}

impl Drop for InterruptContextHandle {
    fn drop(&mut self) {
        IS_INTERRUPT_CONTEXT.store(false, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
    fn as_usize(self) -> usize {
        self as usize
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = make_idt_static();
}

fn make_idt_static() -> InterruptDescriptorTable {
    let mut idt = InterruptDescriptorTable::new();
    unsafe {
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.page_fault
            .set_handler_fn(page_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);

        idt
    }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    let _int = InterruptContextHandle::new();
    println!("BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    let _int = InterruptContextHandle::new();
    panic!("DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    err: PageFaultErrorCode,
) {
    let _int = InterruptContextHandle::new();
    use x86_64::registers::control::Cr2;
    super::memory::do_page_fault(Cr2::read(), stack_frame, err);

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", err);
    println!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    let _int = InterruptContextHandle::new();
    unsafe {
        PICS.lock_uninterruptible()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
        timer_event_handler();
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    let _int = InterruptContextHandle::new();
    use pc_keyboard::{layouts, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    fn make_keyboard_static() -> Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> {
        Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1))
    }
    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            make_keyboard_static();
    }

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    {
        let mut keyboard = KEYBOARD.lock();
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                println!("key pressed {:?}", key);
            }
        }
    }

    unsafe {
        PICS.lock_uninterruptible()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub fn init() {
    crate::call_stack!();
    IDT.load();
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}
