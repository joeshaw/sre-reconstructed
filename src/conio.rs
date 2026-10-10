// Borland conio on the sysop's local screen (the C library's text-mode
// functions, and the window streams of segment 1fd3 built on them).
//
// The host terminal stands in for the PC's screen: positioning, clearing
// and colours become ANSI sequences, and the terminal's cursor and
// editing keys become the PC's extended key codes.  The local screen is
// taken to be 80x25, all of it the initial window (the original kept
// its top and bottom lines for the sysop's status bars).

use crate::devices::{term_flush, term_getc, term_peek, term_write, term_write_glyphs};
use crate::sre::Sre;
use crate::sstring::Str;
use crate::stream::*;
use crate::out;

#[derive(Clone, Copy)]
pub struct TextInfo {
    pub winleft: u8,
    pub wintop: u8,
    pub winright: u8,
    pub winbottom: u8,
    pub attribute: u8,
    pub screenwidth: u8,
    pub screenheight: u8,
}

pub struct ConState {
    pub text: TextInfo,
    pub curx: i16, // in the window
    pub cury: i16,
}

impl Default for ConState {
    fn default() -> Self {
        ConState {
            text: TextInfo { winleft: 1, wintop: 1, winright: 80, winbottom: 25, attribute: 7, screenwidth: 80, screenheight: 25 },
            curx: 1,
            cury: 1,
        }
    }
}

// Test scripts (SRE_TESTMARK set) get conio output bracketed by DCS ...
// ST, to tell it from the local stream's.
fn cwrite(g: &mut Sre, s: &[u8], glyphs: bool) {
    let marked = g.dev.term.testmark;
    if marked {
        term_write(g, b"\x1bP");
    }
    if glyphs {
        term_write_glyphs(g, s);
    } else {
        term_write(g, s);
    }
    if marked {
        term_write(g, b"\x1b\\");
    }
}

fn put(g: &mut Sre, s: &[u8]) {
    cwrite(g, s, false);
}

// Move the terminal's cursor to (x, y) of the window.
fn place(g: &mut Sre) {
    let t = g.con.text;
    let seq = format!("\x1b[{};{}H", t.wintop as i32 + g.con.cury as i32 - 1, t.winleft as i32 + g.con.curx as i32 - 1);
    put(g, seq.as_bytes());
}

pub fn gettextinfo(g: &Sre) -> TextInfo {
    g.con.text
}

pub fn window(g: &mut Sre, left: i16, top: i16, right: i16, bottom: i16) {
    let t = g.con.text;
    if left < 1 || top < 1 || right > t.screenwidth as i16 || bottom > t.screenheight as i16 || left > right || top > bottom {
        return;
    }
    g.con.text.winleft = left as u8;
    g.con.text.wintop = top as u8;
    g.con.text.winright = right as u8;
    g.con.text.winbottom = bottom as u8;
    gotoxy(g, 1, 1);
}

pub fn gotoxy(g: &mut Sre, x: i16, y: i16) {
    let t = g.con.text;
    if x < 1 || y < 1 || x > t.winright as i16 - t.winleft as i16 + 1 || y > t.winbottom as i16 - t.wintop as i16 + 1 {
        return;
    }
    g.con.curx = x;
    g.con.cury = y;
    place(g);
}

pub fn clreol(g: &mut Sre) {
    // The windows used all reach the right edge of the screen.
    put(g, b"\x1b[K");
}

pub fn clrscr(g: &mut Sre) {
    let t = g.con.text;
    let mut y: i16 = 1;
    while y <= t.winbottom as i16 - t.wintop as i16 + 1 {
        g.con.curx = 1;
        g.con.cury = y;
        place(g);
        clreol(g);
        y += 1;
    }
    gotoxy(g, 1, 1);
}

fn line_op(g: &mut Sre, op: &[u8]) {
    let t = g.con.text;
    let seq = format!("\x1b[{};{}r", t.wintop, t.winbottom);
    put(g, seq.as_bytes());
    place(g);
    put(g, op);
    put(g, b"\x1b[r");
    g.con.curx = 1;
    place(g);
}

pub fn insline(g: &mut Sre) {
    line_op(g, b"\x1b[L");
}

pub fn delline(g: &mut Sre) {
    line_op(g, b"\x1b[M");
}

pub fn textattr(g: &mut Sre, attr: i16) {
    const ANSI: &[u8] = b"04261537";
    let fg = attr & 15;
    let bg = (attr >> 4) & 7;
    let mut seq: Vec<u8> = b"\x1b[0;".to_vec();
    if fg & 8 != 0 {
        seq.extend_from_slice(b"1;");
    }
    seq.push(b'3');
    seq.push(ANSI[(fg & 7) as usize]);
    seq.extend_from_slice(b";4");
    seq.push(ANSI[bg as usize]);
    seq.push(b'm');
    put(g, &seq);
    g.con.text.attribute = attr as u8;
}

pub fn cputs(g: &mut Sre, s: &[u8]) {
    cwrite(g, s, true);
    for &c in s {
        if c == b'\r' {
            g.con.curx = 1;
        } else if c == b'\n' {
            g.con.cury += 1;
        } else if c == 8 {
            g.con.curx = if g.con.curx > 1 { g.con.curx - 1 } else { 1 };
        } else if c != 7 {
            g.con.curx += 1;
        }
    }
}

pub fn setcursortype(g: &mut Sre, typ: i16) {
    put(g, if typ != 0 { b"\x1b[?25h" } else { b"\x1b[?25l" });
}

// The next byte of local input without taking it: -1 at the end, -2 if
// none has arrived.  NULs are test-script markers (see devices.rs).
fn peek(g: &mut Sre) -> i32 {
    loop {
        let c = term_peek(g);
        if c == 0 && g.dev.term.testmark {
            term_getc(g);
            continue;
        }
        return c;
    }
}

pub fn kbhit(g: &mut Sre) -> bool {
    let c = peek(g);
    if c == -1 {
        g.exit(1); // the end of the host's input
    }
    c >= 0
}

// A terminal escape sequence for a cursor or editing key, as its scan code.
fn escape_key(g: &mut Sre) -> i16 {
    let mut c = term_getc(g);
    let mut n = 0;
    if c == b'[' as i32 {
        loop {
            c = term_getc(g);
            if (b'0' as i32..=b'9' as i32).contains(&c) {
                n = n * 10 + (c - b'0' as i32);
            } else if c != b';' as i32 {
                break;
            }
        }
    } else {
        c = term_getc(g);
    }
    match c as u8 {
        b'A' => 0x48,
        b'B' => 0x50,
        b'C' => 0x4d,
        b'D' => 0x4b,
        b'H' => 0x47,
        b'F' => 0x4f,
        b'~' => match n {
            1 | 7 => 0x47,
            4 | 8 => 0x4f,
            2 => 0x52,
            3 => 0x53,
            5 => 0x49,
            6 => 0x51,
            _ => 0,
        },
        _ => 0,
    }
}

pub fn bioskey0(g: &mut Sre) -> i16 {
    loop {
        term_flush(g);
        while !kbhit(g) {
            crate::brtl::b_sleep(15);
        }
        let mut c = term_getc(g);
        if c == 0x1b {
            let n = peek(g);
            if n == b'[' as i32 || n == b'O' as i32 {
                let scan = escape_key(g);
                if scan != 0 {
                    return scan << 8;
                }
                continue;
            }
        }
        if c == 0x7f {
            c = 8;
        }
        return c as i16;
    }
}

// ---------------------------------------------------------------------
// 1fd3:0471: set the window and remember it.
pub fn set_window(g: &mut Sre, left: i16, top: i16, right: i16, bottom: i16) {
    window(g, left, top, right, bottom);
}

// 1fd3:04a2: the whole screen as the window.
pub fn full_window(g: &mut Sre) {
    let t = g.con.text;
    set_window(g, 1, 1, t.screenwidth as i16, t.screenheight as i16);
}

// 1fd3:0342: colours take effect at once.
pub fn window_set_color(g: &mut Sre, mut bg: i16, fg: i16) {
    if bg == -1 {
        bg = g.ost(KWIN).curbg as i16;
    }
    textattr(g, (bg << 4) | fg);
    let o = g.ost(KWIN);
    o.curfg = fg as i8;
    o.newfg = fg as i8;
    o.curbg = bg as i8;
    o.newbg = bg as i8;
}

// 1fd3:03ab
pub fn win_at(g: &mut Sre, x: i16, y: i16) {
    gotoxy(g, x, y);
}

// 1fd3:05f1
pub fn win_clear(g: &mut Sre) {
    gotoxy(g, 1, 1);
    clrscr(g);
}

// 1fd3:0630
pub fn win_clear_eol(g: &mut Sre) {
    clreol(g);
}

fn pending(g: &mut Sre) -> &mut i16 {
    match &mut g.ist(KWININ).kind {
        IKind::Window { pending } => pending,
        _ => panic!("not kwinin"),
    }
}

// 1fd3:0268: keys from the local keyboard; an extended key reads as 0
// and then its scan code.
pub fn window_get_key(g: &mut Sre) -> u8 {
    let p = *pending(g);
    if p != 0 {
        *pending(g) = 0;
        return p as u8;
    }
    if g.ost(KWIN).raw == 0 {
        out!(g, KWIN; b"\xdb\x08");
    }
    *pending(g) = 0; // 1fd3:0235
    let mut k = bioskey0(g);
    if k & 0xff != 0 {
        k &= 0xff;
    }
    if g.ost(KWIN).raw == 0 {
        out!(g, KWIN; " \x08");
    }
    if (k >> 8) == 0 {
        return k as u8;
    }
    if ((k >> 8) & 0xff) == 1 {
        return (k & 0xff) as u8;
    }
    *pending(g) = (k >> 8) & 0xff;
    0
}

// 1fd3:090d
pub fn window_get_char(g: &mut Sre) -> u8 {
    in_char(g, KWININ)
}

// ---------------------------------------------------------------------
// Scrolling pick lists (segments 2858 and 28b5): a column of strings in
// the current window, chosen with the cursor keys.
pub struct PickList {
    pub current: i16,   // +04
    pub normal: i16,    // +08 attribute of the items
    pub highlight: i16, // +0a attribute of the current item
    pub items: Vec<Str>, // +10
    pub capacity: i16,  // +06
    pub top: i16,       // +14 first item shown
    pub height: i16,    // +16
    pub width: i16,     // +18
}

impl PickList {
    // 2858:000f, 28b5:000e
    pub fn new(g: &Sre, normal: i16, highlight: i16) -> PickList {
        let t = gettextinfo(g);
        PickList {
            current: 0,
            normal,
            highlight,
            items: Vec::new(),
            capacity: 2,
            top: 0,
            width: t.winright as i16 - t.winleft as i16 + 1,
            height: t.winbottom as i16 - t.wintop as i16 + 1,
        }
    }

    pub fn count(&self) -> i16 {
        self.items.len() as i16
    }

    // 2858:0189
    pub fn add(&mut self, s: &[u8]) {
        if self.count() == self.capacity {
            self.capacity *= 2;
        }
        self.items.push(s.to_vec());
    }

    // 2858:02cb (1-based)
    pub fn item(&self, n: i16) -> Option<&Str> {
        if n < 1 || self.capacity < n {
            return None;
        }
        self.items.get((n - 1) as usize)
    }

    // 2858:0306
    fn draw_item(&self, g: &mut Sre, index: i16, row: i16, attr: i16) {
        if index < 0 || self.capacity <= index {
            return;
        }
        gotoxy(g, 1, row + 1);
        textattr(g, attr);
        if let Some(s) = self.items.get(index as usize) {
            cputs(g, s);
        }
        clreol(g);
    }

    // 28b5:00ee
    fn draw_all(&mut self, g: &mut Sre) {
        for i in 0..self.height {
            let n = self.normal;
            self.show(g, i + self.top, n);
        }
    }

    // 28b5:011e: show an item, scrolling to it.
    fn show(&mut self, g: &mut Sre, index: i16, attr: i16) {
        let row = index - self.top;
        if row == -1 {
            gotoxy(g, 1, 1);
            insline(g);
            self.draw_item(g, index + index - self.top, 0, self.normal);
            self.top -= 1;
        }
        if self.height == row {
            gotoxy(g, 1, 1);
            delline(g);
            self.draw_item(g, index, self.height - 1, self.normal);
            self.top += 1;
        }
        if row < -1 || self.height < row {
            if row < -1 {
                self.top = index;
            }
            if self.height < row {
                self.top = index - self.height + 1;
            }
            for i in 0..self.height {
                self.draw_item(g, i + self.top, i, self.normal);
            }
        }
        self.draw_item(g, index, index - self.top, attr);
        if self.top == index && self.top > 0 && self.normal != attr {
            gotoxy(g, self.width - 1, 1);
            textattr(g, attr);
            cputs(g, b"\x18");
            gotoxy(g, self.width - 1, 1);
        }
        if self.top + self.height - 1 == index && self.top + self.height < self.count() && self.normal != attr {
            gotoxy(g, self.width - 1, self.height);
            textattr(g, attr);
            cputs(g, b"\x19");
            gotoxy(g, self.width - 1, self.height);
        }
    }

    // 2858:03dd: 1-based choice, 0 for none
    pub fn run(&mut self, g: &mut Sre) -> i16 {
        if self.current < 0 {
            self.current = 0;
        }
        self.draw_all(g);
        if self.count() == 0 {
            return 0;
        }
        setcursortype(g, 0);
        let (c, h) = (self.current, self.highlight);
        self.show(g, c, h);
        loop {
            let prev = self.current;
            let mut key: i16;
            loop {
                key = 0;
                if kbhit(g) {
                    key = window_get_char(g) as i16;
                    if key == 0 {
                        key = (window_get_char(g) as i16) << 8;
                    }
                }
                if key == 0 {
                    crate::brtl::b_sleep(15);
                }
                if key != 0 {
                    break;
                }
            }
            if key == b'\r' as i16 {
                break;
            }
            if key == 0x1b {
                self.current = -1;
                break;
            }
            match key as u16 {
                0x4800 => self.current -= 1,
                0x4900 => self.current = 0,
                0x5000 => self.current += 1,
                0x5100 => self.current = self.count() - 1,
                _ => {}
            }
            if self.current < 0 {
                self.current = 0;
            }
            if self.current >= self.count() {
                self.current = self.count() - 1;
            }
            if self.current != prev {
                let n = self.normal;
                self.show(g, prev, n);
                let (c, h) = (self.current, self.highlight);
                self.show(g, c, h);
            }
        }
        setcursortype(g, 2);
        if self.current >= 0 && (self.current as usize) < self.items.len() {
            return self.current + 1;
        }
        0
    }
}
