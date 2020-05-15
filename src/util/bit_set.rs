use core::ops::Range;

pub struct BitSet<'a> {
    n_bits: u64,
    data: &'a mut [u8],
}

impl BitSet<'_> {
    pub fn new(n_bits: u64, data: &mut [u8]) -> BitSet<'_> {
        assert!(n_bits <= data.len() as u64 * 8);
        BitSet { n_bits, data }
    }

    pub fn set(&mut self, idx: u64, value: bool) {
        assert!(idx < self.n_bits);
        let word_mask = 1 << (idx % 8) as u8;
        let word = unsafe { self.data.get_unchecked_mut(idx as usize / 8) };
        if value {
            *word |= word_mask;
        } else {
            *word &= !word_mask;
        }
    }

    pub fn set_all(&mut self, range: Range<u64>, value: bool) {
        // TODO efficient impl
        for idx in range {
            self.set(idx, value);
        }
    }

    pub fn get(&self, idx: u64) -> bool {
        assert!(idx < self.n_bits);
        let word_mask = 1 << (idx % 8) as u8;
        unsafe { (*self.data.get_unchecked(idx as usize / 8) & word_mask) != 0 }
    }
}

// use crate::util::default_in_place::DefaultInPlace;

// const WORD_SIZE: u64 = 32;

// const fn word_count(length: u64) -> u64 {
//     (length + WORD_SIZE - 1) / WORD_SIZE
// }

// pub struct BitSet<const LENGTH: u64> {
//     array: [u32; word_count(LENGTH) as usize],
// }

// impl<const LENGTH: u64> BitSet<{ LENGTH }> {
//     pub fn new() -> Self {
//         unsafe { core::mem::zeroed() }
//     }

//     pub fn length(&self) -> u64 {
//         LENGTH
//     }

//     pub fn get(&self, idx: u64) -> bool {
//         assert!(idx < LENGTH);
//         let word_idx = idx / WORD_SIZE;
//         let word_mask = 1 << ((idx % WORD_SIZE) as u32);
//         let word = self.array[word_idx as usize];
//         (word & word_mask) != 0
//     }

//     pub fn set(&mut self, idx: u64, value: bool) {
//         assert!(idx < LENGTH);
//         let word_idx = idx / WORD_SIZE;
//         let word_mask = 1 << ((idx % WORD_SIZE) as u32);
//         self.array[word_idx as usize] |= word_mask;
//     }

//     pub fn set_all(&mut self, idx_from: u64, idx_to: u64, value: bool) {
//         // TODO: more efficient impl
//         for idx in idx_from..idx_to {
//             self.set(idx, value);
//         }
//     }

//     pub fn find_first(&self, idx_from: u64, idx_to: u64, value: bool) -> Option<u64> {
//         // TODO: more efficient impl
//         for idx in idx_from..idx_to {
//             if self.get(idx) == value {
//                 return Some(idx);
//             }
//         }
//         None
//     }
// }

// impl<const LENGTH: u64> Default for BitSet<{ LENGTH }> {
//     fn default() -> Self {
//         BitSet::new()
//     }
// }

// impl<const LENGTH: u64> DefaultInPlace for BitSet<{ LENGTH }> {
//     unsafe fn default_in_place(s: *mut Self) {
//         let arr = &mut (&mut *s).array;
//         for i in 0..arr.len() {
//             arr[i] = 0;
//         }
//     }
// }

// #[test_case]
// fn bitset_test1() {
//     println!("bitset test");
//     type BS = BitSet<128>;
//     let mut bitset = BS::new();
//     bitset.set(0, true);
//     bitset.set(6, true);
//     bitset.set(125, true);
//     assert_eq!(bitset.get(0), true);
//     assert_eq!(bitset.get(1), false);
//     assert_eq!(bitset.get(6), true);
//     assert_eq!(bitset.get(125), true);
//     assert_eq!(bitset.get(124), false);
// }
