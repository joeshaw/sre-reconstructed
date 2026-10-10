// The String class of the SRCOMM library (segments 273e and 27eb), as
// byte vectors (the game's text is in the IBM PC character set), and C
// string helpers for the fixed-size name fields of the data files.

use crate::brtl::b_stricmp;

pub type Str = Vec<u8>;

// The C string in a fixed-size field: the bytes before the first NUL.
pub fn cstr(s: &[u8]) -> &[u8] {
    match s.iter().position(|&c| c == 0) {
        Some(n) => &s[..n],
        None => s,
    }
}

// An owned copy of a field's C string.
pub fn cs(s: &[u8]) -> Str {
    cstr(s).to_vec()
}

// strncpy(dst, src, n): copy at most n bytes, padding with NULs.
pub fn strncpy(dst: &mut [u8], src: &[u8], n: usize) {
    let src = cstr(src);
    for i in 0..n.min(dst.len()) {
        dst[i] = if i < src.len() { src[i] } else { 0 };
    }
}

// strcpy into a field (the source must fit, with its NUL).
pub fn strcpy(dst: &mut [u8], src: &[u8]) {
    let src = cstr(src);
    let n = src.len().min(dst.len().saturating_sub(1));
    dst[..n].copy_from_slice(&src[..n]);
    dst[n] = 0;
}

// Set a name field the way the game does: strncpy(dst, s, len - 1) and
// a NUL at the end.
pub fn set_name(dst: &mut [u8], src: &[u8]) {
    let n = dst.len() - 1;
    strncpy(dst, src, n);
    dst[n] = 0;
}

// 273e:066d: character at i, or 0 past the end.
pub fn at(s: &[u8], i: i16) -> u8 {
    if (i as u16 as usize) < s.len() {
        s[i as u16 as usize]
    } else {
        0
    }
}

// 273e:0692: substring (start, count); count < 0 means to the end.
pub fn substr(s: &[u8], mut start: i16, mut count: i16) -> Str {
    let len = s.len() as i16;
    if len < start {
        start = len;
    }
    if count < 0 {
        count = len;
    }
    let mut end = start.wrapping_add(count);
    if len < end {
        end = len;
    }
    let mut r = Str::new();
    let mut i = start;
    while i < end {
        r.push(s[i as usize]);
        i += 1;
    }
    r
}

// 273e:0738: erase count chars at start (-1 = to the end).
pub fn erase(s: &mut Str, mut start: i16, mut count: i16) {
    let len = s.len() as i16;
    if count == -1 {
        count = len;
    }
    if len < start {
        start = len;
    }
    if start as i32 + count as i32 > len as i32 {
        count = len - start;
    }
    if count > 0 {
        s.drain(start as usize..(start + count) as usize);
    }
}

// 273e:07cc: insert at position.
pub fn insert(s: &mut Str, mut pos: i16, o: &[u8]) {
    let len = s.len() as i16;
    if len < pos {
        pos = len;
    }
    let p = pos as usize;
    s.splice(p..p, o.iter().cloned());
}

// 27eb:053c: index of o at or after from, or -1.
pub fn find(s: &[u8], o: &[u8], from: i16) -> i16 {
    let last = (o.len() as i16).wrapping_add(from).wrapping_sub(1);
    if last < 0 {
        return 0;
    }
    let lc = at(o, last);
    let len = s.len() as i16;
    let mut i = last;
    while i < len {
        let p = s[i as usize..].iter().position(|&c| c == lc);
        match p {
            None => break,
            Some(off) => i += off as i16,
        }
        if at(s, i) != lc {
            i += 1;
            continue;
        }
        let mut j: i16 = 0;
        let mut k = i - last;
        while j < last {
            if at(o, j) != at(s, k) {
                break;
            }
            j += 1;
            k += 1;
        }
        if j >= last {
            return i - last;
        }
        i += 1;
    }
    -1
}

// 27eb:0492: index of c at or after from, or -1.
pub fn findch(s: &[u8], c: u8, from: i16) -> i16 {
    let n = s.len() as i16;
    let mut i = from;
    while i < n {
        if at(s, i) == c {
            return i;
        }
        i += 1;
    }
    -1
}

// 27eb:0425: case-insensitive comparison with a C string (stricmp == 0).
pub fn str_eq_i(a: &[u8], b: &[u8]) -> bool {
    b_stricmp(b, a) == 0
}

// 27eb:03e5: case-insensitive equality of two Strings.
pub fn str_eq_i2(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && b_stricmp(b, a) == 0
}

// 27eb:0049: decimal text of v, with thousands separators if commas.
pub fn num_str(mut v: i32, commas: bool) -> Str {
    let mut s = Str::new();
    let mut neg = false;
    if v < 0 {
        neg = true;
        v = v.wrapping_neg();
    }
    if v == 0 {
        s.push(b'0');
    } else {
        while v > 0 {
            s.push(b'0' + (v % 10) as u8);
            v /= 10;
            if s.len() % 4 == 3 && commas {
                s.push(b',');
            }
        }
        if commas && at(&s, s.len() as i16 - 1) == b',' {
            let n = s.len() as i16 - 1;
            erase(&mut s, n, 1);
        }
        if commas && s.len() == 5 {
            s[3] = at(&s, 4);
            let n = s.len() as i16 - 1;
            erase(&mut s, n, 1);
        }
    }
    if neg {
        s.push(b'-');
    }
    s.reverse();
    s
}

// 27eb:0003 / 27eb:0026: String(int) / String(long).
pub fn int_str(v: i32) -> Str {
    num_str(v, false)
}

// 27eb:0255: parse a number, ignoring commas; 0 if not all digits.
pub fn str_to_long(s: &[u8]) -> i32 {
    let s: Str = s.iter().cloned().filter(|&c| c != b',').collect();
    // strtoll(p, &end, 0), clamped to 32 bits.
    let at = |i: usize| if i < s.len() { s[i] } else { 0 };
    let mut i = 0;
    while matches!(at(i), b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c) {
        i += 1;
    }
    let mut neg = false;
    if at(i) == b'-' || at(i) == b'+' {
        neg = at(i) == b'-';
        i += 1;
    }
    let mut base = 10u32;
    let digits_start;
    if at(i) == b'0' && (at(i + 1) == b'x' || at(i + 1) == b'X') && at(i + 2).is_ascii_hexdigit() {
        base = 16;
        i += 2;
        digits_start = i;
    } else if at(i) == b'0' {
        base = 8;
        digits_start = i;
    } else {
        digits_start = i;
    }
    let mut w: i128 = 0;
    let mut any = false;
    loop {
        let c = at(i);
        let d = match c {
            b'0'..=b'9' => (c - b'0') as u32,
            b'a'..=b'z' => (c - b'a') as u32 + 10,
            b'A'..=b'Z' => (c - b'A') as u32 + 10,
            _ => 99,
        };
        if d >= base {
            break;
        }
        any = true;
        if w < (1i128 << 80) {
            w = w * base as i128 + d as i128;
        }
        i += 1;
    }
    let _ = digits_start;
    if !any {
        return 0;
    }
    if neg {
        w = -w;
    }
    // strtoll saturates at the 64-bit limits, then the game clamps.
    w = w.clamp(i64::MIN as i128, i64::MAX as i128);
    w = w.clamp(-0x80000000, 0x7fffffff);
    w as i32
}

// 27eb:02d4: round v to the given number of decimal digits of
// precision (1..9).
#[allow(dead_code)] // unused, as in the original
pub fn round_digits(mut v: i32, digits: i16) -> i32 {
    if v < 0 {
        return round_digits(v.wrapping_neg(), digits - 1).wrapping_neg();
    }
    if !(1..=9).contains(&digits) {
        return v;
    }
    let mut lim: i32 = 1;
    for _ in 0..digits {
        lim *= 10;
    }
    let mut n: i16 = 0;
    while v >= lim {
        let d = (v % 10) as i16;
        v /= 10;
        if v < lim && d >= 5 {
            v += 1;
        }
        n += 1;
    }
    while n > 0 {
        v = v.wrapping_mul(10);
        n -= 1;
    }
    v
}

// Concatenation helper: cat(&[a, b, c]).
pub fn cat(parts: &[&[u8]]) -> Str {
    let mut r = Str::new();
    for p in parts {
        r.extend_from_slice(p);
    }
    r
}
