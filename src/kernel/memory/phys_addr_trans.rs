use x86_64::{PhysAddr, VirtAddr};

pub struct AddressTranslator {
    physical_memory_offset: VirtAddr
}

impl AddressTranslator {
    pub fn new(physical_memory_offset: VirtAddr) -> Self {
        Self { physical_memory_offset }
    }

    pub fn physical_to_virtual(&self, addr: PhysAddr) -> VirtAddr {
        self.physical_memory_offset + addr.as_u64()
    }
}