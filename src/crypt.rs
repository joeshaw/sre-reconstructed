// Data file integrity checks and encryption (segment 7420, 14f6:000f..01eb).

use crate::brtl::b_rand;
use crate::game::Checksum;
use std::sync::OnceLock;

// 7420:0000
fn crc_table() -> &'static [u32; 256] {
    static TAB: OnceLock<[u32; 256]> = OnceLock::new();
    TAB.get_or_init(|| {
        let mut t = [0u32; 256];
        for (i, e) in t.iter_mut().enumerate() {
            let mut v = i as u32;
            for _ in 0..8 {
                v = if v & 1 != 0 { (v >> 1) ^ 0xedb88320 } else { v >> 1 };
            }
            *e = v;
        }
        t
    })
}

// 7420:009a
pub fn crc32(data: &[u8]) -> i32 {
    let tab = crc_table();
    let mut crc: u32 = 0xffffffff;
    for &b in data {
        crc = ((crc >> 8) & 0x00ffffff) ^ tab[((crc ^ b as u32) & 0xff) as usize];
    }
    crc as i32
}

// 7420:0130: a 16-bit running sum of sums (the routine computes more,
// but only this survives into the result).
pub fn sum16(data: &[u8]) -> i32 {
    let (mut s1, mut s2) = (0u16, 0u16);
    for &b in data {
        s1 = s1.wrapping_add(b as u16);
        s2 = s2.wrapping_add(s1);
    }
    s2 as i32
}

fn rol3(k: u8) -> u8 {
    k.rotate_left(3)
}

// 7420:0196: encrypt in place; returns the final key.
pub fn encrypt(p: &mut [u8], salt: &[u8; 8]) -> [u8; 8] {
    let mut k = *salt;
    let mut f: u8 = 0;
    for i in 0..p.len() {
        let j = i & 7;
        let orig = p[i];
        p[i] ^= k[j];
        k[j] = rol3(k[j]) ^ f ^ orig;
        f = f.wrapping_add(7);
    }
    k
}

// 7420:028c: decrypt in place; true if the final key matches.
pub fn decrypt(p: &mut [u8], salt: &[u8; 8], key: &[u8; 8]) -> bool {
    let mut k = *salt;
    let mut f: u8 = 0;
    for i in 0..p.len() {
        let j = i & 7;
        p[i] ^= k[j];
        k[j] = rol3(k[j]) ^ f ^ p[i];
        f = f.wrapping_add(7);
    }
    k == *key
}

// 14f6:005c: checksum and encrypt (8 calls of rand()).
pub fn make_checksum(data: &mut [u8]) -> Checksum {
    let mut c = Checksum::default();
    let sum = sum16(data);
    let crc = crc32(data);
    let mut salt = [0u8; 8];
    for s in salt.iter_mut() {
        *s = (((b_rand() as i32) << 8) / 0x8000) as u8;
    }
    let key = encrypt(data, &salt);
    c.sum = sum;
    c.crc = crc;
    c.salt = salt;
    c.key = key;
    c
}

// 14f6:0147: decrypt and verify both sums.
pub fn check_checksum(data: &mut [u8], c: &Checksum) -> bool {
    if !decrypt(data, &c.salt, &c.key) {
        return false;
    }
    c.sum == sum16(data) && c.crc == crc32(data)
}

// 14f6:01cb: decrypt only.
pub fn unscramble(data: &mut [u8], c: &Checksum) -> bool {
    decrypt(data, &c.salt, &c.key)
}
