// Research developments: Text/Research.LST compiled into
// Data/Research.DAT (segment 6db3).

use crate::layout::record;
use crate::prelude::*;

const RESEARCH_DAT: &[u8] = b"Data/Research.DAT"; // DS:a514
const RESEARCH_LST: &[u8] = b"Text/Research.LST"; // DS:a518

record! {
    // A record of Data/Research.DAT (0xff bytes).
    pub struct Development {
        pub index: i16,         // 00
        pub cost: i32,          // 02 research points per planet
        pub typ: i16,           // 06 planet type
        pub kind: i16,          // 08 1 = permanent, 2 = temporary, 0 = menu
        pub amount: i32,        // 0a
        pub text: [u8; 0xf1],   // 0e
    }
}
const _: () = assert!(<Development as Le>::SIZE == 0xff);

fn mtime(name: &[u8]) -> i32 {
    use std::os::unix::fs::MetadataExt;
    match std::fs::metadata(host_path(name)) {
        Ok(m) => m.mtime() as i32,
        Err(_) => 0,
    }
}

// 6db3:063c
pub fn research_load(g: &mut Sre, d: &mut Development, index: i16) {
    d.text = [0; 0xf1];
    let mut f = DataFile::new(g, RESEARCH_DAT, 0xff, 0, 1);
    let mut b = d.to_bytes();
    let ok = index >= 0 && f.read(index as i32, &mut b, 1);
    *d = Development::from_bytes(&b);
    if !ok {
        d.index = index;
        d.cost = 0;
        d.typ = 0;
        d.kind = 0;
        d.amount = 0;
    }
}

// 6db3:0000: recompile the list if it is newer than the data file.
pub fn research_init(g: &mut Sre) -> i16 {
    let lst = mtime(RESEARCH_LST);
    let dat = mtime(RESEARCH_DAT);
    if lst as u32 <= dat as u32 {
        return 0;
    }
    out!(g, KLOCAL; col(3), "Recompiling ", RESEARCH_LST, " to ", RESEARCH_DAT, "...", cn(g));
    let inp = ifstream(g, RESEARCH_LST, false);
    host_remove(RESEARCH_DAT);
    let mut outf = DataFile::new(g, RESEARCH_DAT, 0xff, 0, 1);
    let mut n: i32 = 0;
    let mut line: i32 = 0;
    while !ifstream_fail(g, inp) {
        let s = read_line(g, inp);
        line += 1;
        let list = parse_list(&s);
        if node_length(&list) == 0 {
            continue;
        }
        let mut err: Option<&str> = None;
        let mut items: Vec<NodeP> = vec![None; 5];
        if node_length(&list) != 5 {
            err = Some(":  5 items needed.\n");
        } else {
            for (i, it) in items.iter_mut().enumerate() {
                *it = node_nth(&list, i as i16);
            }
            let t = |k: usize| items[k].as_deref().map(|n| n.typ());
            if t(0) != Some(AT_SYMBOL) {
                err = Some(":  item 1 should be a name.\n");
            } else if t(1) != Some(AT_SYMBOL) {
                err = Some(":  item 2 should be a name.\n");
            } else if t(2) != Some(AT_NUMBER) {
                err = Some(":  item 3 should be a number.\n");
            } else if t(3) != Some(AT_NUMBER) {
                err = Some(":  item 4 should be a number.\n");
            } else if t(4) != Some(AT_STRING) {
                err = Some(":  item 5 should be a string.\n");
            }
        }
        if let Some(e) = err {
            out!(g, KLOCAL; "Error on line ", line, e);
            continue;
        }
        let it = |k: usize| items[k].as_deref().unwrap();
        let planet = it(0).str().to_vec();
        let duration = it(1).str().to_vec();
        let text = it(4).str().to_vec();
        let mut kind: i16 = 0;
        if str_eq_i(b"permanent", &duration) {
            kind = 1;
        }
        if str_eq_i(b"temporary", &duration) {
            kind = 2;
        }
        if str_eq_i(b"menu", &duration) {
            kind = 3;
        }
        if kind == 0 {
            out!(g, KLOCAL; "Error on line ", line, ":  duration must be 'temporary' or 'permanent'.\n");
            continue;
        }
        let mut t: i16 = 0;
        while t < 10 {
            if !str_eq_i(planet_name(t as i8), &planet) {
                t += 1;
                continue;
            }
            let mut d = Development::default();
            research_load(g, &mut d, -1);
            d.index = n as i16;
            d.typ = t;
            d.cost = it(3).num();
            d.amount = it(2).num();
            d.kind = if kind == 1 {
                1
            } else if kind == 2 {
                2
            } else {
                0
            };
            strncpy(&mut d.text, &text, 0xef);
            d.text[0xef] = 0;
            let b = d.to_bytes();
            outf.write(g, n, &b, 1);
            n += 1;
            break;
        }
        if t == 10 {
            out!(g, KLOCAL; "Error on line ", line, ":  planet type not found.\n");
        }
    }
    ifstream_done(g, inp);
    kout!(g; "\n\n");
    n as i16
}

// 6db3:05da
pub fn research_count(g: &mut Sre) -> i16 {
    if g.m.researchcount != -1 {
        return g.m.researchcount;
    }
    let mut f = DataFile::new(g, RESEARCH_DAT, 0xff, 0, 1);
    g.m.researchcount = f.numrecs() as i16;
    g.m.researchcount
}

impl Development {
    // 6db3:073b
    pub fn permanent(&self) -> i32 {
        if self.kind == 1 {
            self.amount
        } else {
            0
        }
    }

    // 6db3:075d
    pub fn temporary(&self) -> i32 {
        if self.kind == 2 {
            self.amount
        } else {
            0
        }
    }

    // 6db3:077f: the text, with "^\BF" colour codes.
    pub fn show(&self, g: &mut Sre) {
        let hex = |c: u8| -> i16 {
            if (c as i8) > b'@' as i8 {
                c as i8 as i16 - 55
            } else {
                c as i8 as i16 - 48
            }
        };
        let text = cstr(&self.text).to_vec();
        let at = |i: usize| self.text.get(i).cloned().unwrap_or(0);
        let len = text.len();
        let mut i = 0;
        while i < len {
            if text[i] == b'^' && at(i + 1) == b'\\' {
                let bg = hex(b_toupper(at(i + 2)));
                let fg = hex(at(i + 3));
                if (0..=15).contains(&bg) && (0..=15).contains(&fg) {
                    i += 3;
                    if bg == 0 {
                        kout!(g; col2(fg, -1));
                    } else {
                        kout!(g; col2(fg, bg));
                    }
                } else {
                    kout!(g; text[i]);
                }
            } else {
                kout!(g; text[i]);
            }
            i += 1;
        }
        kout!(g; b'\n');
    }
}
