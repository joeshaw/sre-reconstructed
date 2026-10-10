// Messages between empires: the line editor, sending, reading and replying
// (segment 58b6).

use crate::prelude::*;
use std::any::Any;

pub const MESSAGE_CLASS: &[u8] = b"Message"; // DS:6335
const MESSAGE_FILE: &[u8] = b"Data/Message"; // DS:663c
pub static R_MESSAGES: GRes = GRes::num(b"Messages", 1); // 62e8
static R_ANONYMOUS: GRes = GRes::num(b"Messages.Anonymous", 1); // 62f4

pub fn r_messages(g: &mut Sre) -> bool {
    g.grb(&R_MESSAGES)
}

pub const MAXLINES: i16 = 99;
pub const LINELEN: usize = 0x50;
pub const QUOTE: u8 = 0xff;
pub const MF_ANON: u8 = 1;
pub const MF_PUBLIC: u8 = 2;

const HEADER: usize = 0x85;
const DATASIZE: usize = 0x1f75;
// The lines are one buffer, as in the original, where a line's text can
// run on into the next; some room past the end stands in for whatever
// followed the object in memory.
const LINEBUF: usize = MAXLINES as usize * LINELEN + 0x200;

// Class "Message" (vtable 676a).
#[derive(Clone)]
pub struct Message {
    pub w0: i16,           // 00
    pub w2: i16,           // 02
    pub from: i16,         // 04 sending empire, -1 = sysop/non-player
    pub created: i32,      // 06
    pub expires: i32,      // 0a
    pub name: [u8; 0x20],  // 0e sender as shown
    pub to: [u8; 0x20],    // 2e recipient letters
    pub list: [i16; 25],   // 4e recipients (public: those who read it)
    pub count: i16,        // 80
    pub flags: u8,         // 82 1 anonymous, 2 public
    pub nlines: i16,       // 83
    pub lines: Vec<u8>,    // 85
}

pub fn new_message() -> Box<dyn Streamable> {
    Box::new(Message::new())
}

// The C string at an offset of the line buffer.
fn cstr_at(b: &[u8], off: usize) -> Str {
    cstr(&b[off.min(b.len())..]).to_vec()
}

fn lo(i: i16) -> usize {
    i as usize * LINELEN
}

impl Message {
    // 58b6:047e
    pub fn new() -> Message {
        let mut m = Message {
            w0: 0,
            w2: 0,
            from: 0,
            created: 0,
            expires: 0,
            name: [0; 0x20],
            to: [0; 0x20],
            list: [0; 25],
            count: 0,
            flags: 0,
            nlines: 0,
            lines: vec![0; LINEBUF],
        };
        m.clear();
        m
    }

    // 58b6:0387
    pub fn clear(&mut self) {
        self.w0 = -1;
        self.w2 = -1;
        self.from = -1;
        self.count = 0;
        self.created = 0;
        self.expires = 0;
        self.name = [0; 0x20];
        self.to = [0; 0x20];
        self.list = [-1; 25];
        for b in self.lines[..MAXLINES as usize * LINELEN].iter_mut() {
            *b = b' ';
        }
        self.flags &= !MF_ANON;
        self.flags &= !MF_PUBLIC;
        self.nlines = 0;
    }

    fn ch(&self, line: i16, col: usize) -> u8 {
        self.lines[lo(line) + col]
    }

    fn set(&mut self, line: i16, col: usize, c: u8) {
        let o = lo(line) + col;
        if o < self.lines.len() {
            self.lines[o] = c;
        }
    }

    // 58b6:0205: for public messages the list holds those who have read it.
    pub fn receives(&self, id: i16) -> bool {
        let n = self.count.max(0) as usize;
        if self.flags & MF_PUBLIC != 0 {
            return !self.list[..n.min(25)].contains(&id);
        }
        self.list[..n.min(25)].contains(&id)
    }

    // 58b6:029a
    pub fn mark_read(&mut self, id: i16) {
        if self.flags & MF_PUBLIC != 0 {
            for i in 0..self.count.max(0) as usize {
                if self.list[i] == id {
                    return;
                }
            }
            if self.count < 25 {
                self.list[self.count as usize] = id;
                self.count += 1;
            } else {
                self.count = 0;
                self.flags &= !MF_PUBLIC;
            }
            return;
        }
        let mut i: i16 = 0;
        while i < self.count {
            if self.list[i as usize] == id {
                self.count -= 1;
                self.list[i as usize] = self.list[self.count as usize];
                i -= 1;
            }
            i += 1;
        }
    }

    // 58b6:04f6: sent to the empire, rather than seen by its spies.
    pub fn addressed_to(&self, id: i16) -> bool {
        if self.flags & MF_PUBLIC != 0 {
            return true;
        }
        let mut i = 0;
        while i < self.to.len() && self.to[i] != 0 {
            if self.to[i] as i8 as i16 == id + b'A' as i16 {
                return true;
            }
            i += 1;
        }
        false
    }

    // 58b6:054b
    pub fn header(&self, g: &mut Sre, n: i16, c1: i16, c2: i16) {
        kout!(g; col(c1), b"\xda");
        for _ in 0..66 {
            kout!(g; 0xc4u8);
        }
        if n < 1000 {
            kout!(g; 0xc4u8);
        }
        if n < 100 {
            kout!(g; 0xc4u8);
        }
        if n < 10 {
            kout!(g; 0xc4u8);
        }
        if n <= 0 {
            kout!(g; 0xc4u8);
        } else {
            kout!(g; col(c2), n);
        }
        kout!(g; col(c1), b"\xc4\xc4\n");
        kout!(g; col(c1), b"\xb3  ", cn(g), "From:    ");
        if self.from != -1 {
            kout!(g; cemp(g), cs(&self.name), b'\n');
        } else if self.name[0] != 0 {
            kout!(g; cn(g), cs(&self.name), b'\n');
        } else {
            kout!(g; col(14), "SYSOP\n");
        }
        kout!(g; col(c1), b"\xb3  ", cn(g), "To:      ", cemp(g), cs(&self.to), b'\n');
        if self.flags & MF_ANON != 0 || self.flags & MF_PUBLIC != 0 || self.from == -1 {
            kout!(g; col(c1), b"\xb3  ", cn(g), "Flags:   ", col(c1));
            if self.flags & MF_ANON != 0 {
                kout!(g; "(anonymous) ");
            }
            if self.flags & MF_PUBLIC != 0 {
                kout!(g; "(public) ");
            }
            if self.from == -1 {
                kout!(g; if self.name[0] != 0 { "(non-player) " } else { "(sysop) " });
            }
            kout!(g; b'\n');
        }
    }

    // 58b6:0868: the details only the sysop sees.
    pub fn show_info(&self, g: &mut Sre, c: i16) {
        if self.from != -1 {
            kout!(g; col(c), b"\xb3  ", cn(g), "Real sender:  ", cemp(g), (self.from as u8).wrapping_add(b'A'), b'\n');
        }
        let t = time_str(g, self.created);
        kout!(g; col(c), b"\xb3  ", cn(g), "Date:         ", t, b'\n');
    }

    // 58b6:0961
    pub fn show_lines(&self, g: &mut Sre, c1: i16, _c2: i16, c3: i16, c4: i16) {
        for i in 0..self.nlines.max(0) {
            kout!(g; col(c1), 0xb3u8);
            let mut s;
            if self.ch(i, 0) == QUOTE {
                kout!(g; col(c3), self.ch(i, 1), b'>', col(c4));
                s = lo(i) + 2;
            } else {
                kout!(g; "  ", cn(g));
                s = lo(i);
            }
            while s < self.lines.len() && self.lines[s] != 0 {
                kout!(g; self.lines[s]);
                s += 1;
            }
            kout!(g; b'\n');
        }
    }

    // 58b6:0aa4
    fn line_number(&self, g: &mut Sre, c: i16) {
        if self.nlines == MAXLINES {
            kout!(g; col(14), "END");
        } else {
            kout!(g; col(c), (self.nlines + 1) / 10, (self.nlines + 1) % 10, b' ');
        }
    }

    // 58b6:0b54: the line editor.  With a quote character the old text is
    // kept as quoted lines.
    pub fn edit(&mut self, g: &mut Sre, quote: u8, c: i16) {
        let cmd = col(14);
        let mut column: i16 = 0;
        kout!(g; col(c), "   [---+----|----+----|----+----|----+----|----+----|----+----|----+----]\n");
        if quote != 0 {
            let mut n: i16 = 0;
            let mut i: i16 = 0;
            while i < self.nlines && n < 20 {
                if self.ch(i, 0) == QUOTE {
                    i += 1;
                    continue;
                }
                let tmp = cstr_at(&self.lines, lo(i));
                self.set(n, 0, QUOTE);
                self.set(n, 1, quote);
                self.set(n, 2, 0);
                let mut o = lo(n) + 2;
                for &b in tmp.iter() {
                    if o < self.lines.len() {
                        self.lines[o] = b;
                    }
                    o += 1;
                }
                if o < self.lines.len() {
                    self.lines[o] = 0;
                }
                n += 1;
                i += 1;
            }
            self.nlines = n;
            self.show_lines(g, 3, 0xb, 9, 3);
        } else {
            self.nlines = 0;
        }
        self.line_number(g, 10);
        loop {
            kout!(g; col(15));
            let k = in_char(g, KIN);
            if (k == b'\r' || k == b'\n') && self.nlines < MAXLINES {
                kout!(g; b'\n');
                self.set(self.nlines, column as usize, 0);
                self.nlines += 1;
                column = 0;
                self.line_number(g, 10);
                continue;
            }
            if k == 8 && column > 0 {
                kout!(g; "\x08 \x08");
                column -= 1;
                self.set(self.nlines, column as usize, 0);
                continue;
            }
            if k == 8 && self.nlines > 0 && self.ch(self.nlines - 1, 0) != QUOTE {
                self.nlines -= 1;
                column = cstr_at(&self.lines, lo(self.nlines)).len() as i16;
                kout!(g; "\x08\x08\x08");
                self.line_number(g, 0xc);
                let t = cstr_at(&self.lines, lo(self.nlines));
                kout!(g; col(15), t);
                continue;
            }
            if k == b'/' && column == 0 {
                kout!(g; col(15), b'/', col(7), b'-');
                kout!(g; col(14), "Command ", cbr(g), b'[', cbo(g));
                kout!(g; b'S', col(3), "=save ", cbo(g));
                kout!(g; b'A', col(3), "=abort ", cbo(g));
                kout!(g; b'L', col(3), "=list ", cbo(g));
                kout!(g; b'C', col(3), "=clear");
                kout!(g; cbr(g), b']', cn(g), b' ', col(15));
                let mut a = in_char(g, KIN);
                for _ in 0..0x2a {
                    kout!(g; "\x08 \x08");
                }
                a = b_toupper(a);
                if a == b'S' {
                    kout!(g; cmd, "\x08\x08\x08Save message.\n");
                    return;
                }
                if a == b'A' {
                    kout!(g; cmd, "Are you sure?  ");
                    let mut y;
                    loop {
                        y = b_toupper(in_char(g, KIN));
                        if y == b'Y' || y == b'N' {
                            break;
                        }
                    }
                    for _ in 0..0xf {
                        kout!(g; "\x08 \x08");
                    }
                    if y == b'Y' {
                        kout!(g; cmd, "\x08\x08\x08Aborting...\n");
                        self.nlines = 0;
                        return;
                    }
                }
                if a == b'C' {
                    kout!(g; cmd, "\x08\x08\x08Cleared message; start over.\n");
                    self.nlines = 0;
                    column = 0;
                    self.line_number(g, 10);
                }
                if a == b'L' {
                    kout!(g; cmd, "\x08\x08\x08List:\n", cn(g));
                    self.show_lines(g, 3, 0xb, 9, 3);
                    self.line_number(g, 10);
                }
                continue;
            }
            let printable = (0x20..=0xaf).contains(&k);
            if printable && column < 0x45 + (ctype_signed(k) & CT_SP) as i16 && self.nlines < MAXLINES {
                kout!(g; k);
                self.set(self.nlines, column as usize, k);
                column += 1;
                continue;
            }
            if !printable || self.nlines >= 0x62 {
                continue;
            }
            // Wrap the last word onto a new line.
            let line = lo(self.nlines);
            let mut brk: i16 = column - 1;
            while ctype_signed(self.lines[line + brk as usize]) & CT_SP == 0 && brk > 0x32 {
                brk -= 1;
            }
            if brk > 0x32 {
                brk += 1;
            } else {
                brk = column;
            }
            for _ in brk..column {
                kout!(g; "\x08 \x08");
            }
            kout!(g; b'\n');
            let mut m: usize = 0;
            for i in brk..column {
                let b = self.lines[line + i as usize];
                self.set(self.nlines + 1, m, b);
                m += 1;
            }
            self.lines[line + brk as usize] = 0;
            self.nlines += 1;
            self.set(self.nlines, m, k);
            m += 1;
            self.set(self.nlines, m, 0);
            self.line_number(g, 10);
            let t = cstr_at(&self.lines, lo(self.nlines));
            kout!(g; col(15), t);
            column = m as i16;
        }
    }

    // 58b6:13ce
    pub fn add_recipient(&mut self, id: i16) {
        for i in 0..self.count.max(0) as usize {
            if self.list[i] == id {
                return;
            }
        }
        if (self.count as usize) < 25 {
            self.list[self.count as usize] = id;
        }
        self.count += 1;
    }

    // 58b6:1425
    pub fn remove_recipient(&mut self, id: i16) {
        for i in 0..self.count.max(0) as usize {
            if self.list[i] == id {
                self.count -= 1;
                self.list[i] = self.list[self.count as usize];
                return;
            }
        }
    }

    // 58b6:1487: sort the recipients and spell them out.
    pub fn sort_recipients(&mut self) {
        let n = self.count.max(0) as usize;
        for i in 0..n.saturating_sub(1) {
            for j in i + 1..n {
                if self.list[i] > self.list[j] {
                    self.list.swap(i, j);
                }
            }
        }
        for i in 0..n {
            self.to[i] = (self.list[i] as u8).wrapping_add(b'A');
        }
        if n < self.to.len() {
            self.to[n] = 0;
        }
    }

    // 58b6:156a
    pub fn address(&mut self, from: i16, to: i16) {
        self.flags &= !MF_ANON;
        self.flags &= !MF_PUBLIC;
        if to == -1 {
            self.count = 0;
        } else {
            self.list[0] = to;
            self.count = 1;
        }
        self.sort_recipients();
        self.from = from;
    }

    // 58b6:15db: turn a received message into a reply; returns the
    // empire replied to (-1 if anonymous), or -2 if no one was chosen.
    pub fn prepare_reply(&mut self, g: &mut Sre, me: i16) -> i16 {
        let anon = self.flags & MF_ANON != 0;
        let mut to = self.from;
        if anon {
            to = -1;
        }
        self.flags &= !MF_ANON;
        let all = if self.flags & MF_PUBLIC != 0 {
            yes_no(g, 0, b"Publicly post your reply")
        } else if self.to[1] == 0 && self.to[0] as i8 as i16 - b'A' as i16 == me {
            false
        } else {
            yes_no(g, 0, b"Send a copy to all original recipients")
        };
        let mut chosen: i32 = 0;
        if !anon {
            mask_set(&mut chosen, self.from);
        }
        if all {
            if self.flags & MF_PUBLIC == 0 {
                let mut i = 0;
                while i < self.to.len() && self.to[i] != 0 {
                    mask_set(&mut chosen, self.to[i] as i8 as i16 - b'A' as i16);
                    i += 1;
                }
                if self.count != 1 {
                    mask_clear(&mut chosen, me);
                }
            }
        } else {
            self.flags &= !MF_PUBLIC;
            self.list[0] = self.from;
        }
        self.count = 0;
        if self.flags & MF_PUBLIC == 0 {
            let m = empire_mask(g);
            let c = choose_empires(g, m, &mut chosen, b"*", list_empires, Some(b"Destinations: ")) as u8;
            if anon && yes_no(g, 1, b"Include the original (anonymous) author") {
                mask_set(&mut chosen, self.from);
            }
            if c == 0 {
                return -2;
            }
            if c == b'*' {
                self.flags |= MF_PUBLIC;
            } else {
                for i in 0..25 {
                    if mask_test(chosen, i) {
                        self.add_recipient(i);
                    }
                }
                self.sort_recipients();
            }
        }
        self.from = me;
        to
    }

    // 58b6:181e
    pub fn choose_recipients(&mut self, g: &mut Sre, from: i16) -> bool {
        self.clear();
        let mut chosen: i32 = 0;
        let m = empire_mask(g);
        let c = choose_empires(g, m, &mut chosen, b"*", list_empires, Some(b"Choose Empires [* for a public message] > ")) as u8;
        if c == 0 {
            return false;
        }
        if c == b'*' {
            kout!(g; cin(g), "(Public Message)", cn(g), b'\n');
            self.count = 0;
            self.flags |= MF_PUBLIC;
            strcpy(&mut self.to, b"All");
        } else {
            self.count = 0;
            for i in 0..25 {
                if mask_test(chosen, i) {
                    self.add_recipient(i);
                }
            }
            self.sort_recipients();
            if self.count == 0 {
                return false;
            }
            self.flags &= !MF_PUBLIC;
        }
        self.from = from;
        true
    }

    // 58b6:195c: write the text and sign the message.
    pub fn compose(&mut self, g: &mut Sre, sender: &[u8], quote: u8) -> bool {
        kout!(g; cn(g), "You have ", cnum(g), MAXLINES, cn(g), " lines for the message.  ", cbr(g), b'[', cbo(g), "/S",
              cbr(g), b']', cn(g), " to save!\n");
        self.edit(g, quote, 3);
        if self.nlines == 0 {
            return false;
        }
        kout!(g; cn(g));
        if g.grb(&R_ANONYMOUS) && yes_no(g, 0, b"Anonymous") {
            strcpy(&mut self.name, b"<Anonymous>");
            self.flags |= MF_ANON;
        } else {
            set_name(&mut self.name, sender);
            self.flags &= !MF_ANON;
        }
        self.expires = g.time.now.wrapping_add(604800);
        self.created = g.time.now;
        if self.flags & MF_PUBLIC != 0 {
            kout!(g; cn(g), "How many days would you like to leave this public message up? [1-30] ");
            set_defval(g, KIN, 7);
            set_maxval(g, KIN, 30);
            let mut days = in_i16(g, KIN);
            if days < 1 {
                days = 1;
            }
            if days > 30 {
                days = 30;
            }
            self.expires = g.time.now.wrapping_add(86400 * days as i32);
            kout!(g; cn(g));
        }
        true
    }
}

impl Streamable for Message {
    // 58b6:01a4
    fn class_name(&self) -> &'static [u8] {
        MESSAGE_CLASS
    }

    // 58b6:01bb
    fn size(&self) -> i16 {
        (self.nlines as i32 * LINELEN as i32 + 0x87) as i16
    }

    // 58b6:009f
    fn expired(&mut self, g: &mut Sre) -> bool {
        if self.flags & MF_PUBLIC == 0 && self.count == 0 {
            return true;
        }
        g.time.now > self.expires
    }

    // 58b6:011c
    fn on_empire_deleted(&mut self, _g: &mut Sre, id: i16) {
        let mut i: i16 = 0;
        while i < self.count {
            if self.list[i as usize] == id {
                self.count -= 1;
                self.list[i as usize] = self.list[self.count as usize];
                i -= 1;
            }
            i += 1;
        }
        if self.from == id {
            self.count = 0;
            self.flags &= !MF_PUBLIC;
        }
    }

    fn data(&self) -> Vec<u8> {
        let mut b = vec![0u8; DATASIZE];
        self.w0.wr(&mut b[0x00..]);
        self.w2.wr(&mut b[0x02..]);
        self.from.wr(&mut b[0x04..]);
        self.created.wr(&mut b[0x06..]);
        self.expires.wr(&mut b[0x0a..]);
        b[0x0e..0x2e].copy_from_slice(&self.name);
        b[0x2e..0x4e].copy_from_slice(&self.to);
        self.list.wr(&mut b[0x4e..]);
        self.count.wr(&mut b[0x80..]);
        b[0x82] = self.flags;
        self.nlines.wr(&mut b[0x83..]);
        b[HEADER..].copy_from_slice(&self.lines[..DATASIZE - HEADER]);
        b
    }

    fn set_data(&mut self, b: &[u8]) {
        self.w0 = i16::rd(&b[0x00..]);
        self.w2 = i16::rd(&b[0x02..]);
        self.from = i16::rd(&b[0x04..]);
        self.created = i32::rd(&b[0x06..]);
        self.expires = i32::rd(&b[0x0a..]);
        self.name.copy_from_slice(&b[0x0e..0x2e]);
        self.to.copy_from_slice(&b[0x2e..0x4e]);
        self.list = <[i16; 25]>::rd(&b[0x4e..]);
        self.count = i16::rd(&b[0x80..]);
        self.flags = b[0x82];
        self.nlines = i16::rd(&b[0x83..]);
        let n = (b.len() - HEADER).min(self.lines.len());
        self.lines[..n].copy_from_slice(&b[HEADER..HEADER + n]);
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

// 58b6:15bd
fn list_empires(g: &mut Sre, mask: i32) {
    score_list(g, mask, None);
}

// 58b6:0000: the message being written (DS:6300).
fn the_message(g: &mut Sre) -> Box<Message> {
    g.m.message.take().unwrap_or_else(|| Box::new(Message::new()))
}

// 58b6:1bee: write a message from an empire to one other empire; the
// caller files it (the message stays in Sre's message slot).
pub fn compose_message(g: &mut Sre, e: usize, to: i16) -> bool {
    let mut m = the_message(g);
    m.address(g.emp[e].id, to);
    let name = ename(g, e);
    let ok = m.compose(g, &name, 0);
    g.m.message = Some(m);
    ok
}

// 58b6:1ce9
pub fn send_message(g: &mut Sre, from: i16, name: &[u8]) -> bool {
    let mut m = the_message(g);
    let r = send_message_with(g, &mut m, from, name);
    g.m.message = Some(m);
    r
}

fn send_message_with(g: &mut Sre, m: &mut Message, from: i16, name: &[u8]) -> bool {
    m.address(from, -1);
    if !m.choose_recipients(g, from) {
        return false;
    }
    if !m.compose(g, name, 0) {
        return false;
    }
    if from != -1 {
        // Spies planted in the sender's court get copies.
        for i in 0..25usize {
            if g.emp[from as usize].b2d3[i] as u8 > 0 && !m.receives(i as i16) {
                m.add_recipient(i as i16);
                g.emp[from as usize].b2d3[i] = g.emp[from as usize].b2d3[i].wrapping_sub(1);
            }
        }
    }
    append_object(g, MESSAGE_FILE, m);
    true
}

fn as_message(o: &mut Box<dyn Streamable>) -> &mut Message {
    o.as_any().downcast_mut::<Message>().expect("not a message")
}

// 58b6:1eb3
pub fn read_messages(g: &mut Sre, _p: &mut Player, e: usize) -> i16 {
    let mut n: i16 = 0;
    let id = g.emp[e].id;
    let mut f = StreamFile::new(g, MESSAGE_FILE);
    loop {
        let inpos = f.inp.tell();
        let outpos = f.outp.tell();
        let mut o = match f.read(g) {
            Some(o) => o,
            None => break,
        };
        let m = as_message(&mut o);
        if !m.receives(id) {
            f.keep(g, m);
            continue;
        }
        n += 1;
        if !m.addressed_to(id) {
            kout!(g; col(5), "    (", col(13), "reported by a spy", col(5), ")\n");
            m.header(g, n, 5, 0xd);
            m.show_lines(g, 5, 0xd, 0xc, 6);
        } else {
            m.header(g, n, 3, 0xb);
            m.show_lines(g, 3, 0xb, 9, 3);
        }
        if m.from < 0 {
            pause_prompt(g);
            m.mark_read(id);
            f.keep(g, m);
            continue;
        }
        kout!(g; cbr(g), b'[', cbo(g), b'R', cbr(g), b']', cn(g), "eply, ", cbr(g), b'[', cbo(g), b'Q', cbr(g), b']', cn(g),
              "uoted reply, or ", cbr(g), b'[', cbo(g), b'D', cbr(g), b']', cn(g), "elete?  ", cin(g));
        let c = get_choice(g, b"KRDQ", true);
        if c == b'R' {
            kout!(g; "Reply");
        }
        if c == b'Q' {
            kout!(g; "Quoted Reply");
        }
        if c == b'D' {
            kout!(g; "Delete");
        }
        if c == b'K' {
            kout!(g; "Keep");
        }
        kout!(g; cn(g), b'\n');
        if c != b'K' {
            m.mark_read(id);
        }
        f.keep(g, m);
        if c == b'R' || c == b'Q' {
            let to = m.prepare_reply(g, id);
            let mut again = true;
            if to >= -1 {
                let mut quote = b'-';
                if (0..25).contains(&to) {
                    quote = (to as u8).wrapping_add(b'A');
                }
                if c != b'Q' {
                    quote = 0;
                }
                let name = ename(g, e);
                if m.compose(g, &name, quote) {
                    f.keep(g, m);
                    again = false;
                }
            }
            if again {
                // Show the message again.
                f.inp.seek(inpos);
                f.outp.seek(outpos);
            }
        }
    }
    f.close(g);
    n
}

// 58b6:2481
pub fn communications(g: &mut Sre, _p: &mut Player, e: usize) {
    let first: &[u8] = b"Do you wish to send a message";
    let another: &[u8] = b"Do you wish to send another message";
    if !g.grb(&R_MESSAGES) {
        return;
    }
    let mut q = first;
    loop {
        kout!(g; cn(g));
        let yes = yes_no(g, 0, q);
        if yes {
            let (id, name) = (g.emp[e].id, ename(g, e));
            if send_message(g, id, &name) {
                q = another;
            }
        }
        if !yes {
            break;
        }
    }
}

// 58b6:25f9
pub fn messages_empire_deleted(g: &mut Sre, id: i16) -> i16 {
    let mut n = 0;
    let mut f = StreamFile::new(g, MESSAGE_FILE);
    while let Some(mut o) = f.read(g) {
        o.on_empire_deleted(g, id);
        n += 1;
        f.keep(g, o.as_mut());
    }
    f.close(g);
    n
}

// 58b6:26ba: the sysop reads all messages.
pub fn sysop_messages(g: &mut Sre) -> i16 {
    let mut n: i16 = 0;
    let mut f = StreamFile::new(g, MESSAGE_FILE);
    while let Some(mut o) = f.read(g) {
        let m = as_message(&mut o);
        n += 1;
        m.header(g, n, 3, 0xb);
        m.show_info(g, 3);
        m.show_lines(g, 3, 0xb, 9, 3);
        kout!(g; cbr(g), b'[', cbo(g), b'D', cbr(g), b']', cn(g), "elete or ", cbr(g), b'[', cbo(g), b'K', cbr(g), b']',
              cn(g), "eep?  ", cin(g));
        loop {
            let c = in_char(g, KIN);
            if c == b'K' || c == b'k' || c == b'\r' {
                kout!(g; "Keep");
                f.keep(g, m);
                break;
            }
            if c == b'D' || c == b'd' {
                kout!(g; "Delete");
                break;
            }
        }
        kout!(g; cn(g), b'\n');
    }
    f.close(g);
    n
}

// 58b6:2901
pub fn read_public_messages(g: &mut Sre) -> i16 {
    let mut n: i16 = 0;
    let mut f = StreamFile::new(g, MESSAGE_FILE);
    while let Some(mut o) = f.read(g) {
        let m = as_message(&mut o);
        if m.flags & MF_PUBLIC != 0 {
            n += 1;
            m.header(g, n, 3, 0xb);
            m.show_lines(g, 3, 0xb, 9, 3);
            pause_prompt(g);
        }
        f.keep(g, m);
    }
    f.close(g);
    n
}

// 58b6:2a17
pub fn clear_messages(g: &mut Sre) {
    let f = StreamFile::new(g, MESSAGE_FILE);
    f.close(g);
}
