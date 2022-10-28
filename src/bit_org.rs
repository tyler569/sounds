use std::{collections::VecDeque, io::Read};

pub struct BitOrg {
    pending: VecDeque<bool>,
    unread: VecDeque<u8>,
}

impl BitOrg {
    pub fn new() -> Self {
        Self {
            pending: VecDeque::new(),
            unread: VecDeque::new(),
        }
    }

    // fn to_u8(bits: &[bool]) -> u8 {
    //     let mut res = 0;

    //     bits.iter().rev().fold(0, |a, &b| (a << 1) + b as u8)
    // }

    fn reorg(&mut self) {
        while self.pending.len() >= 8 {
            let mut next = 0;
            for i in 0..8 {
                next <<= 1;
                next += self.pending.pop_front().unwrap() as u8;
            }
            self.unread.push_back(next);
        }
    }

    pub fn push_bits(&mut self, bit_count: u32, bits: u64) {
        for i in (0..bit_count).rev() {
            self.pending.push_back(bits & (1 << i) != 0)
        }

        self.reorg();
    }

    pub fn pop_one(&mut self) -> Option<u8> {
        self.unread.pop_front()
    }
}

impl Read for BitOrg {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut index = 0;
        loop {
            match self.pop_one() {
                Some(v) => buf[index] = v,
                None => break,
            }
            index += 1;
        }

        Ok(index)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_bitorg_usage() {
        let mut bitorg = BitOrg::new();
        bitorg.push_bits(8, 0xaa);
        assert_eq!(bitorg.pop_one(), Some(0xaa));

        bitorg.push_bits(16, 0xaabb);
        assert_eq!(bitorg.pop_one(), Some(0xaa));
        assert_eq!(bitorg.pop_one(), Some(0xbb));

        bitorg.push_bits(4, 0xd);
        assert_eq!(bitorg.pop_one(), None);
        bitorg.push_bits(4, 0xa);
        assert_eq!(bitorg.pop_one(), Some(0xda));
    }

    fn fold_bytes(acc: (u32, u64), &v: &u8) -> (u32, u64) {
        (acc.0 + 1, (acc.1 << 8) + v as u64)
    }

    #[test]
    fn test_reading_bitorg() {
        const DATA: &[u8] = b"Hello World";

        let mut bitorg = BitOrg::new();
        DATA
            .chunks(4)
            .map(|v| v.iter().fold((0, 0), fold_bytes))
            .for_each(|v| bitorg.push_bits(v.0 * 8, v.1));
        
        let mut buffer = [0u8; 64];
        let n = bitorg.read(&mut buffer).unwrap();

        assert_eq!(DATA, &buffer[..n]);
    }
}