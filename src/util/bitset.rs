use crate::*;
use core::ops::Range;

const WORD_SIZE: u64 = 32;

const fn word_count(length: u64) -> u64 {
    (length + WORD_SIZE - 1) / WORD_SIZE
}

pub struct BitSet<const LENGTH: u64> {
    array: [u32; word_count(LENGTH) as usize]
}

impl<const LENGTH: u64> BitSet<{LENGTH}> {
    pub fn new() -> Self {
        unsafe { core::mem::zeroed() }
    }

    pub fn get(&self, idx: u64) -> bool {
        assert!(idx < LENGTH);
        let word_idx = idx / WORD_SIZE;
        let word_mask = 1 << ((idx % WORD_SIZE) as u32);
        let word = self.array[word_idx as usize];
        (word & word_mask) != 0
    }

    pub fn set(&mut self, idx: u64, value: bool) {
        assert!(idx < LENGTH);
        let word_idx = idx / WORD_SIZE;
        let word_mask = 1 << ((idx % WORD_SIZE) as u32);
        self.array[word_idx as usize] |= word_mask;
    }

    pub fn set_all(&mut self, idx_from: u64, idx_to: u64, value: bool) {
        // TODO: more efficient impl
        for idx in idx_from..idx_to {
            self.set(idx, value);
        }
    }
}

impl<const LENGTH: u64> Default for BitSet<{LENGTH}> {
    fn default() -> Self {
        BitSet::new()
    }
}

#[test_case]
fn bitset_test1() {
    println!("bitset test");
    type BS = BitSet<128>;
    let mut bitset = BS::new();
    bitset.set(0, true);
    bitset.set(6, true);
    bitset.set(125, true);
    assert_eq!(bitset.get(0), true);
    assert_eq!(bitset.get(1), false);
    assert_eq!(bitset.get(6), true);
    assert_eq!(bitset.get(125), true);
    assert_eq!(bitset.get(124), false);
}