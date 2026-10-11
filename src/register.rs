// Registration keys (segment 68a3).

use crate::prelude::*;

const KEY_FILE: &[u8] = b"STATUS.SRE"; // DS:8cb0
const KEY_REGISTERED: &[u8] = b"REGISTERED"; // DS:8cb8
const KEY_BETA: &[u8] = b"BETA"; // DS:8cbc
const DEFAULT_BANK: &[u8] = b"First Intergalactic Bank of The Solar Realm"; // 8cdc

// 68a3:0000: decode a registration line into out (a C string, as the
// original writes it); true if its check letters fail.
fn decode_line(inp: &[u8], out: &mut [u8]) -> bool {
    let mut b = [0u8; 0x110];
    let src = cstr(inp);
    let n = src.len().min(0x100);
    b[..n].copy_from_slice(&src[..n]);
    let mut c1 = b[0] as i8;
    let mut c2 = b[1] as i8;
    let mut chk: i16 = 0;
    let mut o = 0usize;
    let mut i = 2usize;
    while i + 3 < b.len() && b[i + 2] != 0 {
        let mut v: i16 = (b[i] as i8 as i16 - b'0' as i16) * 10 + b[i + 1] as i8 as i16 - b'0' as i16;
        v += (c1 as i16 - 32) - (c2 as i16 - 32);
        while v < 0 {
            v += 100;
        }
        v %= 100;
        c1 = c2;
        c2 = (v + 32) as i8;
        if o < out.len() {
            out[o] = (v + 32) as u8;
        }
        o += 1;
        chk = (chk * 3 + v) % 676;
        i += 2;
    }
    if o < out.len() {
        out[o] = 0;
    }
    if b[i] as i8 as i16 != chk / 26 + b'A' as i16 || b[i + 1] as i8 as i16 != chk % 26 + b'A' as i16 {
        out[0] = 0;
        return true;
    }
    false
}

// 68a3:0124: read STATUS.SRE.
fn read_key_file(g: &mut Sre) {
    let f = ifstream(g, KEY_FILE, false);
    let mut kind = 0;
    g.system.regstate = RS_UNREGISTERED;
    strcpy(&mut g.system.bank, DEFAULT_BANK);
    let done = |g: &mut Sre| ifstream_done(g, f);
    if ifstream_fail(g, f) {
        return done(g);
    }
    let line = in_line(g, f);
    if at(&line, 0) == b'A' && at(&line, 1) == b'B' {
        kind = 2;
    }
    if ifstream_fail(g, f) || decode_line(&line, &mut g.system.regto) {
        return done(g);
    }
    if at(&line, 0) == b'H' && at(&line, 1) == b'G' {
        kind = 1;
    }
    let line = in_line(g, f);
    if ifstream_fail(g, f) || decode_line(&line, &mut g.system.bank) {
        strcpy(&mut g.system.bank, DEFAULT_BANK);
        return done(g);
    }
    g.regname = cs(&g.system.regto);
    let line = in_line(g, f);
    let mut kb = [0u8; 0x100];
    if ifstream_fail(g, f) || decode_line(&line, &mut kb) {
        strcpy(&mut g.system.bank, DEFAULT_BANK);
        return done(g);
    }
    let key = cs(&kb);
    if key != KEY_REGISTERED && key != KEY_BETA {
        strcpy(&mut g.system.bank, DEFAULT_BANK);
        return done(g);
    }
    let s = &g.system;
    if kind == 1 && s.bank[0] == b'M' && b_tolower(s.bank[1]) == b'y' && s.regto[0] == b'G' && s.regto[5] == b'P' {
        kb[3] = 0;
    }
    if kind == 2 && s.regto[0] == b'W' && b_tolower(s.regto[1]) == b'a' && s.regto[7] == b'J' && s.regto[13] == b'K' {
        kb[3] = 0;
    }
    let k = kb[0] as i8 as i32 - 0x4f;
    let iok = k >= 0 && kb.get(k as usize) == Some(&b'I');
    g.system.regstate = if iok { RS_REGISTERED } else { RS_UNREGISTERED };
    if key == KEY_BETA {
        g.system.regstate = RS_BETA;
    }
    done(g)
}

// The game is always registered.  The registered name and the bank's name
// come from STATUS.SRE when it has them, whatever its key says; otherwise
// the game is registered to the sysop.  With SRE_UNREGISTERED set,
// registration works as in the original: unregistered without a valid
// STATUS.SRE (the tests use this to compare with the original).
pub fn check_registration(g: &mut Sre) {
    read_key_file(g);
    if g.system.regstate != RS_UNREGISTERED || std::env::var_os("SRE_UNREGISTERED").is_some() {
        return;
    }
    let f = ifstream(g, KEY_FILE, false);
    let line = in_line(g, f);
    if ifstream_fail(g, f) || decode_line(&line, &mut g.system.regto) {
        let who: Str = if g.system.sysop[0] != 0 { cs(&g.system.sysop) } else { b"the Sysop".to_vec() };
        let n = who.len().min(63);
        strcpy(&mut g.system.regto, &who[..n]);
    }
    let line = in_line(g, f);
    if ifstream_fail(g, f) || decode_line(&line, &mut g.system.bank) {
        strcpy(&mut g.system.bank, DEFAULT_BANK);
    }
    ifstream_done(g, f);
    g.regname = cs(&g.system.regto);
    g.system.regstate = RS_REGISTERED;
}

// 68a3:039f: read a registration line, colouring as it is typed.
fn read_key_line(g: &mut Sre) -> Str {
    let mut buf = Str::new();
    loop {
        let mut c = in_char(g, KIN);
        if c.is_ascii_lowercase() {
            c = c - b'a' + b'A';
        }
        let n = buf.len() as i16;
        if c == 8 {
            if n > 0 {
                buf.pop();
                kout!(g; "\x08 \x08");
            }
        } else if (c.is_ascii_digit() && n >= 2) || c.is_ascii_uppercase() {
            if n < 2 {
                kout!(g; col(15));
            } else if ((n - 2) / 3) & 1 != 0 {
                kout!(g; col(11));
            } else if c.is_ascii_uppercase() {
                kout!(g; col(10));
            } else {
                kout!(g; col(14));
            }
            kout!(g; c);
            buf.push(c);
        }
        if c == b'\n' || c == b'\r' {
            kout!(g; b'\n');
            return buf;
        }
    }
}

// 68a3:0500: enter the registration lines (sysop).
pub fn register(g: &mut Sre) -> bool {
    let mut lines = [[0u8; 0x50]; 4];
    let f = ofstream(g, KEY_FILE, false, 0);
    kout!(g; col(14), "\nRegistration of the new SRE\n ");
    hline(g, col(12), 0x4e);
    for i in 1..=3i16 {
        kout!(g; col(7), "Registration Line #", col(11), i, cn(g), ":  ");
        let inp = read_key_line(g);
        let mut bad = decode_line(&inp, &mut lines[i as usize]);
        if !bad && i == 3 {
            bad = cstr(&lines[3]) != KEY_REGISTERED && cstr(&lines[3]) != KEY_BETA;
        }
        if bad {
            kout!(g; col(14), "\nInvalid registration line.\n");
            kout!(g; cn(g), "SRE is currently unregistered.\n");
            g.system.regstate = RS_UNREGISTERED;
            ofstream_done(g, f);
            return false;
        }
        out!(g, f; &inp, b'\n');
    }
    kout!(g; "\n ");
    hline(g, col(12), 0x4e);
    g.system.regstate = RS_REGISTERED;
    if cstr(&lines[3]) == KEY_BETA {
        g.system.regstate = RS_BETA;
    }
    kout!(g; cn(g), "This program is registered to:  ", col(14), cs(&lines[1]), b'\n');
    kout!(g; cn(g), "Bank name is:  \"", col(11), cs(&lines[2]), cn(g), "\"\n");
    kout!(g; "\nThank you for registering this program!\n");
    ofstream_done(g, f);
    true
}
