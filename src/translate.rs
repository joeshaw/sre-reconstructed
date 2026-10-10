// Output translation (segment 155b): "Output.TranslationFile" lets a sysop
// replace words in all game text, and a few colour/bracket resources.

use crate::files::FileIn;
use crate::resource::{as_atom, node_length, node_nth, parse_list, Resource};
use crate::sre::Sre;
use crate::sstring::{at, erase, find, insert, Str};
use crate::stream::*;
use crate::out;

pub struct TransState {
    table: Vec<Vec<(Str, Str)>>, // DS:c96a, by first byte
    pub tr: Str,                 // DS:c962
}

impl Default for TransState {
    fn default() -> Self {
        TransState { table: vec![Vec::new(); 256], tr: Str::new() }
    }
}

// 155b:00ce
fn apply(g: &Sre, s: &mut Str) {
    let mut pos: i16 = 0;
    while pos < s.len() as i16 {
        let ch = at(s, pos);
        for (from, to) in g.trans.table[ch as usize].iter() {
            let n = from.len();
            // strncmp(s + pos, from, n)
            let p = pos as usize;
            let matches = (0..n).all(|k| s.get(p + k).cloned().unwrap_or(0) == from[k]);
            if matches {
                erase(s, pos, n as i16);
                insert(s, pos, to);
                pos -= 1;
                pos += to.len() as i16;
                break;
            }
        }
        pos += 1;
    }
    let atname = b"@A";
    loop {
        let i = find(s, atname, 0);
        if i < 0 {
            break;
        }
        erase(s, i, 2);
        insert(s, i, b"Amit Patel");
    }
}

// 155b:027c: translate a string.
pub fn translate(g: &mut Sre, s: &[u8]) -> Str {
    if at(s, 0) == b'\\' {
        g.trans.tr = s[1..].to_vec();
        return g.trans.tr.clone();
    }
    if at(s, 0) == 0 || at(s, 1) == 0 || at(s, 2) == 0 {
        g.trans.tr = s.to_vec();
        return s.to_vec();
    }
    let mut t = s.to_vec();
    apply(g, &mut t);
    g.trans.tr = t.clone();
    t
}

// 155b:00b2: length of the translated string.
pub fn display_len(g: &mut Sre, s: &[u8]) -> i16 {
    translate(g, s);
    g.trans.tr.len() as i16
}

// 155b:0313: read the translation file and the Output.* resources.
pub fn load_translation(g: &mut Sre) {
    let mut tf = Resource::num(b"Output.TranslationFile", 0);
    let name = tf.str(&mut g.resdb);
    if !name.is_empty() {
        let mut f = FileIn::open(&name, false);
        for t in g.trans.table.iter_mut() {
            t.clear();
        }
        let mut lineno: i16 = 0;
        while !f.fail() {
            let mut line = Str::new();
            loop {
                let c = f.getc();
                if f.fail() {
                    break;
                }
                if c == b'\n' {
                    break;
                }
                line.push(c);
            }
            if at(&line, 0) == b';' || line.is_empty() {
                continue;
            }
            lineno += 1;
            let l = parse_list(&line);
            if node_length(&l) != 2 {
                out!(g, KLOCAL; "Error in translation file:  line ", lineno, " does not have two items on it.\n");
                continue;
            }
            let a = node_nth(&l, 0);
            let b = node_nth(&l, 1);
            let (a, b) = match (as_atom(&a), as_atom(&b)) {
                (Some(a), Some(b)) => (a.str().to_vec(), b.str().to_vec()),
                _ => {
                    out!(g, KLOCAL; "Error in translation file:  line ", lineno, " needs to have two strings.\n");
                    continue;
                }
            };
            let ch = at(&a, 0);
            g.trans.table[ch as usize].push((a, b));
        }
        g.translate_hook = true;
    }
    let mut lb = Resource::text(b"Output.LeftBracket", Some(b"["));
    let mut rb = Resource::text(b"Output.RightBracket", Some(b"]"));
    let l = lb.str(&mut g.resdb);
    let r = rb.str(&mut g.resdb);
    if !l.is_empty() {
        g.menuchars.itemopen = l[0];
    }
    if !r.is_empty() {
        g.menuchars.itemclose = r[0];
    }
    let mut bc = Resource::num(b"Output.Bracket.Color", g.c.bracket.fg as i32);
    let mut bo = Resource::num(b"Output.Bold.Color", g.c.bold.fg as i32);
    let mut nc = Resource::num(b"Output.Number.Color", g.c.number.fg as i32);
    let mut ec = Resource::num(b"Output.Empire.Color", g.c.empire.fg as i32);
    g.c.bracket.fg = bc.long(&mut g.resdb) as i16;
    g.c.bold.fg = bo.long(&mut g.resdb) as i16;
    g.c.number.fg = nc.long(&mut g.resdb) as i16;
    g.c.empire.fg = ec.long(&mut g.resdb) as i16;
}
