use x86_64::{PhysAddr, VirtAddr};

pub struct PhysAddrTranslator {
    physical_memory_offset: VirtAddr
}

impl PhysAddrTranslator {
    pub fn new(physical_memory_offset: VirtAddr) -> Self {
        Self { physical_memory_offset }
    }

    pub fn translate(&self, addr: PhysAddr) -> VirtAddr {
        self.physical_memory_offset + addr.as_u64()
    }
}