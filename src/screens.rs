// Screens, prompts and text files (segments 43e0 and 7389).

use crate::brtl::b_delay;
use crate::devices::{kout_m, pause_prompt, set_mode};
use crate::files::FileIn;
use crate::sre::Sre;
use crate::sstring::{at, cat, substr, Str};
use crate::stream::*;
use crate::types::b_toupper;
use crate::kout;

pub struct ScreenState {
    pub version: Str,   // DS:baec "0.994b"
    pub copyright: Str, // DS:baf4
    pub flavordir: Str, // DS:d130 "flavors\<flavor>\"
    pub answer: u8,     // DS:c92c last yes/no answer
    pub sbcolor: i16,   // DS:a6e6 StatusBar.Background
    pub statusbars: i16, // DS:a6e8
}

impl Default for ScreenState {
    fn default() -> Self {
        ScreenState {
            version: b"0.994b".to_vec(),
            copyright: Str::new(),
            flavordir: Str::new(),
            answer: 0,
            sbcolor: 2,
            statusbars: 1,
        }
    }
}

// 43e0:0000
pub fn clear_screen(g: &mut Sre) {
    let m = kout_m(g).mode;
    set_mode(g, 1);
    if g.dev.ansi == 0 {
        kout!(g; 0x0cu8);
    } else {
        kout!(g; "\x1b[0m\x1b[2J");
    }
    set_mode(g, m);
}

// 43e0:0056
pub fn centered(g: &mut Sre, fg: i16, s: &[u8]) {
    let mut i: i16 = s.len() as i16 / 2 + 0x23;
    while i < 0x46 {
        kout!(g; b' ');
        i += 1;
    }
    kout!(g; col(fg));
    put_untranslated(g, KOUT, s);
    kout!(g; b'\n');
}

// 43e0:00da
pub fn title_screen(g: &mut Sre) {
    let savedlog = kout_m(g).logon;
    if kout_m(g).logopen != 0 {
        kout_m(g).logon = 0;
    }
    clear_screen(g);
    let sb = g.scr.sbcolor;
    let cn = g.dev.c_normal;
    if show_screen(g, None, b"Title", false) {
        let v = g.scr.version.clone();
        kout!(g; col(sb), 0xaeu8, col(sb + 8), 0xaeu8, col(14), "SRE:II ", col(15), v, col(sb + 8), 0xafu8,
              col(sb), 0xafu8, cn);
        kout!(g; "\nReleased on ", col(11), "Mar 07 1995", col(3), 0xf9u8, "17:40:37", cn, b'\n');
    } else {
        hline(g, col(sb), 0x46);
        centered(g, 15, b"Solar Realms Elite for DOS");
        centered(g, 14, b"Interstellar Imperialists");
        centered(g, 2, b"By");
        centered(g, 10, b"Amit Patel");
        centered(g, 2, b"at The Solar Realm");
        let v = cat(&[b"Version ", &g.scr.version, b": released on Mar 07 1995\xfa17:40:37"]);
        centered(g, 3, &v);
        hline(g, col(sb), 0x46);
    }
    let c = g.scr.copyright.clone();
    put_untranslated(g, KOUT, &c);
    pause_prompt(g);
    if kout_m(g).logopen != 0 {
        kout_m(g).logon = savedlog;
    }
}

// 43e0:042c: the sysop's status line; not part of the user's view.
pub fn status_bar(_g: &mut Sre, _who: &[u8], _what: &[u8], _c: u8) {}

// 43e0:08a4: show base.ANS (if ANSI and IBM), else base.MSG or base.TXT.
pub fn show_file_base(g: &mut Sre, base: &[u8], paged: bool) -> bool {
    let ans = cat(&[base, b".ANS"]);
    let msg = cat(&[base, b".MSG"]);
    let txt = cat(&[base, b".TXT"]);
    let savedpage = g.dev.pagelen;
    g.dev.pagelen = 0;
    let savedraw = g.ost(KOUT).raw;
    let savedmode = kout_m(g).mode;
    if !paged {
        g.ost(KOUT).raw = 1;
    }
    set_mode(g, 1);
    if g.dev.ansi != 0 && g.dev.ibm != 0 && display_file(g, &ans, 0) {
        g.dev.pagelen = savedpage;
        if savedraw == 0 {
            g.ost(KOUT).raw = 0;
        }
        set_mode(g, savedmode);
        return true;
    }
    g.dev.pagelen = savedpage;
    if savedraw == 0 {
        g.ost(KOUT).raw = 0;
    }
    set_mode(g, savedmode);
    display_file(g, &msg, 0) || display_file(g, &txt, 0)
}

// 43e0:0aa1: look for a screen in the flavor directory, then in dir
// (default "Screens/").
pub fn show_screen(g: &mut Sre, dir: Option<&[u8]>, name: &[u8], paged: bool) -> bool {
    let dir = dir.unwrap_or(b"Screens/");
    let f = cat(&[&g.scr.flavordir, name]);
    if show_file_base(g, &f, paged) {
        return true;
    }
    show_file_base(g, &cat(&[dir, name]), paged)
}

// 43e0:0b60: "question?  [y/N]"; def 0 = no, 1 = yes, 2 = must answer.
pub fn yes_no(g: &mut Sre, def: i16, question: &[u8]) -> bool {
    kout!(g; setw(0));
    let q = cat(&[question, b"?  "]);
    let (cbr, cbo, cn) = (g.c.bracket, g.c.bold, g.dev.c_normal);
    kout!(g; q, cbr, b'[', cbo);
    let intl = g.res_intlask();
    if intl {
        let f = g.ost(KOUT).newfg as i16 & 7;
        if def == 1 {
            kout!(g; b'1', col(f), "/0");
        } else {
            kout!(g; b'0', col(f), "/1");
        }
    } else if def == 2 {
        kout!(g; "y/n");
    } else if def == 1 {
        kout!(g; "Y/n");
    } else {
        kout!(g; "y/N");
    }
    kout!(g; cbr, b']', cn, "  ");
    let savedlog = kout_m(g).logon;
    if kout_m(g).logopen != 0 {
        kout_m(g).logon = 0;
    }
    loop {
        let (cn, ci) = (g.dev.c_normal, g.dev.c_input);
        kout!(g; cn, b' ');
        let mut spin = 0;
        while g.dev.effects != 0 && !key_ready(g, KIN) && spin < 20000 {
            kout!(g; col(11), "\x08<");
            b_delay(50);
            kout!(g; col(14), "\x08v");
            b_delay(50);
            kout!(g; col(10), "\x08>");
            b_delay(50);
            kout!(g; col(9), "\x08^");
            b_delay(50);
            kout!(g; "\x08 ");
            spin += 1;
        }
        kout!(g; 8u8, ci);
        let mut a = b_toupper(in_char(g, KIN));
        if a == b'1' {
            a = b'Y';
        }
        if a == b'0' {
            a = b'N';
        }
        if a == b'\r' {
            if def == 1 {
                a = b'Y';
            } else if def == 0 {
                a = b'N';
            }
        }
        g.scr.answer = a;
        if a == b'Y' || a == b'N' {
            break;
        }
    }
    if kout_m(g).logopen != 0 {
        kout_m(g).logon = savedlog;
    }
    let yes = g.scr.answer == b'Y';
    if g.res_intlask() {
        kout!(g; if yes { 1i16 } else { 0i16 });
    } else if yes {
        kout!(g; "Yes");
    } else {
        kout!(g; "No");
    }
    let cn = g.dev.c_normal;
    kout!(g; cn, "\n");
    yes
}

// 43e0:0f44: a line of n box characters in color c.
pub fn hline(g: &mut Sre, c: Color, n: i16) {
    kout!(g; c);
    let s = vec![0xc4u8; n.max(0) as usize];
    let cn = g.dev.c_normal;
    kout!(g; s, cn, b'\n');
}

// ---------------------------------------------------------------------
fn hex_digit(c: u8) -> i16 {
    if (c as i8) > 0x40 {
        (c as i8 as i16) - 55
    } else {
        (c as i8 as i16) - 48
    }
}

// 7389:063a: write a line with "^\BF" color codes.
pub fn put_coded(g: &mut Sre, s: &[u8], o: Sid) {
    let mut start: i16 = 0;
    let len = s.len() as i16;
    for i in 0..len {
        if at(s, i) != b'^' {
            continue;
        }
        if !(i + 3 < len) {
            continue;
        }
        if at(s, i + 1) != b'\\' {
            continue;
        }
        let bg = hex_digit(at(s, i + 2));
        let fg = hex_digit(at(s, i + 3));
        let part = substr(s, start, i - start);
        crate::out!(g, o; part, col2(fg, bg));
        start = i + 4;
    }
    let part = substr(s, start, -1);
    crate::out!(g, o; part);
}

// 7389:07c6: display a text file (each byte XORed with key).
pub fn display_file(g: &mut Sre, name: &[u8], key: u8) -> bool {
    let mut f = FileIn::open(name, true);
    if f.fail() {
        return false;
    }
    let mut line = Str::new();
    while !f.fail() {
        let c = f.getc() ^ key;
        if f.fail() {
            break;
        }
        line.push(c);
        if c == b'\n' && key_ready(g, KIN) {
            let k = in_char(g, KIN);
            if k == b' ' {
                kout!(g; col(4), "\nAborted\n");
                kout!(g; col(7));
                return true;
            }
        }
        if c == b'\n' {
            put_coded(g, &line, KOUT);
            line.clear();
        }
    }
    put_coded(g, &line, KOUT);
    kout!(g; col(7));
    true
}
