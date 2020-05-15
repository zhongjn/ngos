use crate::util::{constant::Constant, init_cell::InitCell};
use x86_64::structures::gdt::*;
use x86_64::structures::tss::*;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

static _TSS: InitCell<Constant<TaskStateSegment>> = InitCell::new();
static _GDT: InitCell<Constant<(GlobalDescriptorTable, Selectors)>> = InitCell::new();

// lazy_static! {
//     static ref TSS: TaskStateSegment = make_tss_static();
// }
// lazy_static! {
//     static ref GDT: (GlobalDescriptorTable, Selectors) = make_gdt_static();
// }

fn make_tss_static() -> TaskStateSegment {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
        const STACK_SIZE: usize = 4096;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
        let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
        let stack_end = stack_start + STACK_SIZE;
        stack_end
    };
    tss
}

fn make_gdt_static(tss: &'static TaskStateSegment) -> (GlobalDescriptorTable, Selectors) {
    let mut gdt = GlobalDescriptorTable::new();
    let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let tss_selector = gdt.add_entry(Descriptor::tss_segment(tss));
    (
        gdt,
        Selectors {
            code_selector,
            tss_selector,
        },
    )
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;

    _TSS.init(Constant::from(make_tss_static()));
    _GDT.init(Constant::from(make_gdt_static(_TSS.get())));
    _GDT.0.load();
    unsafe {
        set_cs(_GDT.1.code_selector);
        load_tss(_GDT.1.tss_selector);
    }

    // GDT.0.load();
    // unsafe {
    //     set_cs(GDT.1.code_selector);
    //     load_tss(GDT.1.tss_selector);
    // }
}
