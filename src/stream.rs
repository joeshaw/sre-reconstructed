// Stream classes of the SRCOMM door library (resident segment 229f).
//
// The original game writes all text through C++ streams ("kout << ...")
// with colour manipulators.  The base output stream keeps the current
// line in an attributed buffer so that it can word-wrap at the stream's
// width, and it emits ANSI colour sequences lazily, just before text.
//
// The streams live in the game's state (Sre::os, Sre::is) and are named
// by index, because writing to one may write to others and wait for
// keys (kout pauses at the end of a page), which waits on the terminal
// and may write to kout again.

use crate::brtl::b_ltoa;
use crate::files::{FileIn, FileOut};
use crate::report::ReportOut;
use crate::sre::Sre;
use crate::sstring::{cstr, Str};
use crate::types::*;

pub type Sid = usize;
pub const KOUT: Sid = 0; // DS:d3b8
pub const KLOCAL: Sid = 1; // DS:d3e2
pub const KMODEM: Sid = 2; // DS:d402
pub const KLOG: Sid = 3; // DS:d422
pub const KWIN: Sid = 4; // DS:d2f6
pub const NEWS: Sid = 5; // DS:d138 g_log2
pub const ERRLOG: Sid = 6; // DS:d160 g_errlog
pub const NFIXED_OS: usize = 7;

pub type Iid = usize;
pub const KIN: Iid = 0; // DS:d3a6
pub const KWININ: Iid = 1; // DS:d316
pub const NFIXED_IS: usize = 2;

// A colour manipulator: kout << Color(fg, bg).  bg == -1 keeps the
// current background.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Color {
    pub fg: i16,
    pub bg: i16,
}

pub const fn col(fg: i16) -> Color {
    Color { fg, bg: -1 }
}
pub const fn col2(fg: i16, bg: i16) -> Color {
    Color { fg, bg }
}
// Colour byte (bg << 4 | fg) as a manipulator.
pub fn byte_color(c: u8) -> Color {
    Color { fg: (c % 16) as i16, bg: (c / 16) as i16 }
}

// Line buffer with per-character colours (229f:0bea).
pub struct AttrString {
    pub buf: Option<Vec<u8>>,
    pub fg: Vec<i16>,
    pub bg: Vec<i16>,
    pub len: i16,
    pub size: i16,
    pub maxsize: i16,
}

impl AttrString {
    pub fn new(n: i16) -> AttrString {
        let (buf, fg, bg) = if n > 0 {
            let k = n as usize + 2;
            (Some(vec![0u8; k]), vec![0i16; k], vec![0i16; k])
        } else {
            (None, Vec::new(), Vec::new())
        };
        AttrString { buf, fg, bg, len: 0, size: n, maxsize: n }
    }

    pub fn append(&mut self, c: u8, f: i16, b: i16) {
        if self.len <= self.size {
            if let Some(buf) = self.buf.as_mut() {
                let l = self.len as usize;
                self.fg[l] = f;
                self.bg[l] = b;
                buf[l] = c;
                self.len += 1;
                buf[l + 1] = 0;
            }
        }
    }

    pub fn pop(&mut self) {
        if let Some(buf) = self.buf.as_mut() {
            if self.len > 0 {
                self.len -= 1;
                buf[self.len as usize] = 0;
            }
        }
    }

    pub fn setsize(&mut self, n: i16) {
        self.size = if self.maxsize < n { self.maxsize } else { n };
    }

    pub fn clear(&mut self) {
        self.len = 0;
        if let Some(buf) = self.buf.as_mut() {
            buf[0] = 0;
        }
    }

    pub fn char_at(&self, i: i16) -> u8 {
        match &self.buf {
            Some(buf) if i >= 0 && self.size > i => buf[i as usize],
            _ => 0,
        }
    }

    pub fn fg_at(&self, i: i16) -> i16 {
        if self.buf.is_some() && i >= 0 && self.size > i {
            self.fg[i as usize]
        } else {
            0
        }
    }

    pub fn bg_at(&self, i: i16) -> i16 {
        if self.buf.is_some() && i >= 0 && self.size > i {
            self.bg[i as usize]
        } else {
            0
        }
    }

    pub fn text(&self) -> &[u8] {
        match &self.buf {
            Some(b) => cstr(b),
            None => &[],
        }
    }
}

// 229f:0d5e
impl Clone for AttrString {
    fn clone(&self) -> AttrString {
        let mut r = AttrString::new(self.size);
        r.maxsize = self.size;
        if let (Some(dst), Some(src)) = (r.buf.as_mut(), self.buf.as_ref()) {
            let t = cstr(src);
            let n = t.len().min(dst.len() - 1);
            dst[..n].copy_from_slice(&t[..n]);
            dst[n] = 0;
            r.len = self.len;
            let k = r.fg.len().min(self.fg.len());
            r.fg[..k].copy_from_slice(&self.fg[..k]);
            r.bg[..k].copy_from_slice(&self.bg[..k]);
        } else {
            r.len = 0;
        }
        r
    }
}

// The kinds of output stream (the original's vtables).
pub enum Kind {
    Multi(crate::devices::MultiOut), // kout (c0b5)
    Modem,                           // c07d
    Local,                           // c099
    Window,                          // bb65
    File(FileOut),                   // be43
    News(Str),                       // 683f
    SysopLog(AttrString),            // 685f
    Report(Box<ReportOut>),          // 924d
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tag {
    Multi,
    Modem,
    Local,
    Window,
    File,
    News,
    SysopLog,
    Report,
}

pub struct OStream {
    pub line: AttrString,
    pub f14: i16,
    pub width: i16, // field width for the next string/number
    pub raw: i16,   // nonzero: no buffering / word-wrap
    pub curfg: i8,  // colours already sent
    pub curbg: i8,
    pub newfg: i8, // colours requested
    pub newbg: i8,
    pub noansi: i16, // nonzero: never emit colour sequences
    pub kind: Kind,
}

impl OStream {
    pub fn new(wrapwidth: i16, kind: Kind) -> OStream {
        OStream {
            line: AttrString::new(wrapwidth),
            f14: 0,
            width: 0,
            raw: 0,
            curfg: 0,
            curbg: 0,
            newfg: 0,
            newbg: 0,
            noansi: 0,
            kind,
        }
    }

    fn tag(&self) -> Tag {
        match self.kind {
            Kind::Multi(_) => Tag::Multi,
            Kind::Modem => Tag::Modem,
            Kind::Local => Tag::Local,
            Kind::Window => Tag::Window,
            Kind::File(_) => Tag::File,
            Kind::News(_) => Tag::News,
            Kind::SysopLog(_) => Tag::SysopLog,
            Kind::Report(_) => Tag::Report,
        }
    }

    pub fn multi(&mut self) -> &mut crate::devices::MultiOut {
        match &mut self.kind {
            Kind::Multi(m) => m,
            _ => panic!("not kout"),
        }
    }
}

// ---------------------------------------------------------------------
// Virtual functions.

// 229f:1d69 (1fd3:0342 for the local window)
pub fn set_color(g: &mut Sre, s: Sid, bg: i16, fg: i16) {
    if g.ost(s).tag() == Tag::Window {
        crate::conio::window_set_color(g, bg, fg);
        return;
    }
    let o = g.ost(s);
    o.newfg = fg as i8;
    o.newbg = if bg != -1 { bg as i8 } else { o.curbg };
}

pub fn emit_color(g: &mut Sre, s: Sid) {
    match g.ost(s).tag() {
        Tag::Multi => crate::devices::multi_emit_color(g),
        Tag::Window => {}
        Tag::News | Tag::Report => {
            // 5b6a:0b22, 69d7:0acd: colours as "^\BF" codes.
            const HEX: &[u8] = b"0123456789ABCDEF";
            let o = g.ost(s);
            let (bg, fg) = (o.newbg, o.newfg);
            let report = o.tag() == Tag::Report;
            if !report {
                o.curfg = fg;
                o.curbg = bg;
            }
            put_raw(g, s, b'^');
            put_raw(g, s, b'\\');
            put_raw(g, s, HEX[(bg as i32 & 15) as usize]);
            put_raw(g, s, HEX[(fg as i32 & 15) as usize]);
            if report {
                let o = g.ost(s);
                o.curfg = fg;
                o.curbg = bg;
            }
        }
        _ => base_emit_color(g, s),
    }
}

// 229f:1664: build "ESC[...m" for the change from cur* to new*.
fn base_emit_color(g: &mut Sre, s: Sid) {
    const ANSI: &[u8] = b"04261537";
    let o = g.ost(s);
    if o.noansi != 0 {
        return;
    }
    let mut seq: Vec<u8> = vec![0x1b, b'['];
    if o.newbg == -1 {
        o.newbg = o.curbg;
    }
    if (o.curfg >= 8 && o.newfg < 8) || (o.newfg == 7 && o.newbg == 0) {
        o.curfg = 7;
        o.curbg = 0;
        seq.extend_from_slice(b"0;");
    }
    if o.curfg < 8 && o.newfg >= 8 {
        o.curfg |= 8;
        seq.extend_from_slice(b"1;");
    }
    if o.curfg != o.newfg {
        o.curfg = o.newfg;
        seq.push(b'3');
        seq.push(ANSI[(o.newfg & 7) as usize]);
        seq.push(b';');
    }
    if o.curbg != o.newbg {
        o.curbg = o.newbg;
        seq.push(b'4');
        seq.push(ANSI[(o.newbg & 7) as usize]);
        seq.push(b';');
    }
    let n = seq.len() - 1;
    seq[n] = b'm';
    if n > 1 {
        raw_write(g, s, &seq);
    }
}

pub fn raw_write(g: &mut Sre, s: Sid, data: &[u8]) {
    match g.ost(s).tag() {
        Tag::Multi => crate::devices::multi_raw_write(g, data),
        Tag::Modem => crate::devices::modem_raw_write(g, data),
        Tag::Local => crate::devices::local_raw_write(g, data),
        Tag::Window => crate::conio::cputs(g, data),
        _ => {
            // 229f:1d26
            for &c in data {
                put_raw(g, s, c);
            }
        }
    }
}

pub fn put_raw(g: &mut Sre, s: Sid, c: u8) -> u8 {
    match g.ost(s).tag() {
        Tag::File => crate::files::ofstream_put_raw(g, s, c),
        Tag::News => crate::news::news_put_raw(g, c),
        Tag::SysopLog => crate::news::sysoplog_put_raw(g, c),
        Tag::Report => crate::report::reportout_put_raw(g, s, c),
        _ => c,
    }
}

// ---------------------------------------------------------------------
// 229f:182e: buffered write with word wrap.
pub fn write(g: &mut Sre, s: Sid, data: &[u8]) {
    {
        let o = g.ost(s);
        if o.curfg != o.newfg || o.curbg != o.newbg {
            emit_color(g, s);
        }
    }
    let n = data.len();
    let mut seg = 0usize;
    let mut seglen: isize = 0;
    let seg_of = |seg: usize, len: isize| -> &[u8] {
        if len <= 0 {
            &data[seg..seg]
        } else {
            &data[seg..seg + len as usize]
        }
    };

    if g.ost(s).raw != 0 {
        for i in 0..n {
            if data[i] == b'\r' {
                raw_write(g, s, seg_of(seg, seglen));
                seglen = -1;
                seg = i + 1;
            } else if data[i] == b'\n' {
                raw_write(g, s, seg_of(seg, seglen));
                raw_write(g, s, b"\r\n");
                seglen = -1;
                seg = i + 1;
            }
            seglen += 1;
        }
        raw_write(g, s, seg_of(seg, seglen));
        return;
    }

    let wrap = g.ost(s).line.size;
    for i in 0..n {
        let c = data[i];
        'body: {
            if c == b'\r' {
                raw_write(g, s, seg_of(seg, seglen));
                seglen = -1;
                seg = i + 1;
                break 'body;
            }
            if c == b'\n' {
                raw_write(g, s, seg_of(seg, seglen));
                raw_write(g, s, b"\r\n");
                seglen = -1;
                seg = i + 1;
                g.ost(s).line.clear();
                break 'body;
            }
            if c == 8 {
                g.ost(s).line.pop();
                break 'body;
            }
            let mut pos;
            {
                let o = g.ost(s);
                if ctype_of(c) & CT_CTL == 0 {
                    let (f, b) = (o.curfg as i16, o.curbg as i16);
                    o.line.append(c, f, b);
                }
                pos = o.line.len;
            }
            if wrap <= 0 || pos < wrap {
                break 'body;
            }

            // The line is full: look back for a space within the last
            // quarter of the line.
            let limit = (wrap as i32 * 3 / 4) as i16;
            while ctype_of(g.ost(s).line.char_at(pos)) & CT_SP == 0 && pos > limit {
                pos -= 1;
            }
            raw_write(g, s, seg_of(seg, seglen + 1));
            seglen = -1;
            seg = i + 1;
            if pos > limit {
                // Erase the partial word, newline, and re-print it.
                pos += 1;
                let (fg, bg) = (g.ost(s).newfg as i16, g.ost(s).newbg as i16);
                set_color(g, s, 0, fg);
                emit_color(g, s);
                let len = g.ost(s).line.len;
                let mut j = pos;
                while j < len {
                    raw_write(g, s, b"\x08 \x08");
                    j += 1;
                }
                raw_write(g, s, b"\r\n");
                set_color(g, s, bg, fg);
                emit_color(g, s);
                let word = g.ost(s).line.clone();
                g.ost(s).line.clear();
                while pos < word.len {
                    set_color(g, s, word.bg_at(pos), word.fg_at(pos));
                    let wc = word.char_at(pos);
                    {
                        let o = g.ost(s);
                        if ctype_of(wc) & CT_CTL == 0 {
                            let (f, b) = (o.curfg as i16, o.curbg as i16);
                            o.line.append(wc, f, b);
                        }
                    }
                    raw_write(g, s, &[wc]);
                    pos += 1;
                }
            } else {
                let (fg, bg) = (g.ost(s).newfg as i16, g.ost(s).newbg as i16);
                set_color(g, s, 0, fg);
                emit_color(g, s);
                g.ost(s).line.clear();
                raw_write(g, s, b"\r\n");
                set_color(g, s, bg, fg);
                emit_color(g, s);
            }
        }
        seglen += 1;
    }
    raw_write(g, s, seg_of(seg, seglen));
}

// ---------------------------------------------------------------------
// The output operators.

// 229f:130c: a string (translated if the hook is set), padded to the
// field width (negative: right-justified).
pub fn put_str(g: &mut Sre, s: Sid, text: &[u8]) {
    let text = cstr(text);
    let t: Str = if g.translate_hook { crate::translate::translate(g, text) } else { text.to_vec() };
    let saved = std::mem::replace(&mut g.translate_hook, false);
    let width = g.ost(s).width;
    let mut pad = width.wrapping_add(t.len() as i16);
    while pad < 0 {
        write(g, s, b" ");
        pad += 1;
    }
    let mut pad = t.len() as i16;
    write(g, s, &t);
    while g.ost(s).width > pad {
        write(g, s, b" ");
        pad += 1;
    }
    g.translate_hook = saved;
}

// 14f6:03ef: a string with the translation hook disabled.
pub fn put_untranslated(g: &mut Sre, s: Sid, text: &[u8]) {
    let saved = std::mem::replace(&mut g.translate_hook, false);
    put_str(g, s, text);
    g.translate_hook = saved;
}

// 229f:1267
pub fn put_char(g: &mut Sre, s: Sid, c: u8) {
    write(g, s, &[c]);
}

// 229f:1425: decimal with thousands separators, right-justified in width.
pub fn put_num(g: &mut Sre, s: Sid, mut v: i32) {
    let mut digits = [0u8; 0x52];
    let mut n: i8 = 0;
    let mut neg = false;
    let comma = g.comma;
    if v < 0 {
        neg = true;
        v = v.wrapping_neg();
    }
    if v == 0 {
        digits[0] = b'0';
        n = 1;
    } else {
        while v > 0 {
            digits[n as usize] = b'0' + (v % 10) as u8;
            n += 1;
            v /= 10;
            if n % 4 == 3 && comma != 0 {
                digits[n as usize] = b',';
                n += 1;
            }
        }
        if n > 0 && digits[(n - 1) as usize] == b',' && comma != 0 {
            n -= 1;
            digits[n as usize] = 0;
        }
        if n == 5 && comma != 0 {
            digits[3] = digits[4];
            digits[4] = 0;
            n -= 1;
        }
    }
    if neg {
        digits[n as usize] = b'-';
        n += 1;
    }
    let mut w = g.ost(s).width as i8;
    if w < 0 {
        w = w.wrapping_neg();
    }
    while w > n {
        write(g, s, b" ");
        w -= 1;
    }
    loop {
        n -= 1;
        if n < 0 {
            break;
        }
        let d = digits[n as usize];
        if d == b',' {
            write(g, s, &[comma]);
        } else {
            write(g, s, &[d]);
        }
    }
}

// 229f:1dd4
pub fn put_color(g: &mut Sre, s: Sid, c: Color) {
    set_color(g, s, c.bg, c.fg);
}

// 229f:10e0
pub fn put_attr(g: &mut Sre, s: Sid, a: &AttrString) {
    let buf = match &a.buf {
        Some(b) => b,
        None => return,
    };
    for i in 0..a.len.max(0) as usize {
        put_color(g, s, col2(a.fg[i], a.bg[i]));
        put_char(g, s, buf[i]);
    }
}

// Manipulators that set the field width (229f:12ac, 229f:12c5).
#[derive(Clone, Copy)]
pub struct SetW(pub i16);
pub fn setw(w: i16) -> SetW {
    SetW(w)
}
pub fn setwr(w: i16) -> SetW {
    SetW(w.wrapping_neg())
}

// A string written with the translation hook disabled.
pub struct Untr(pub Str);

pub trait Put {
    fn put(self, g: &mut Sre, s: Sid);
}

impl Put for &str {
    fn put(self, g: &mut Sre, s: Sid) {
        put_str(g, s, self.as_bytes());
    }
}
impl Put for &[u8] {
    fn put(self, g: &mut Sre, s: Sid) {
        put_str(g, s, self);
    }
}
impl<const N: usize> Put for &[u8; N] {
    fn put(self, g: &mut Sre, s: Sid) {
        put_str(g, s, self);
    }
}
impl Put for Str {
    fn put(self, g: &mut Sre, s: Sid) {
        put_str(g, s, &self);
    }
}
impl Put for &Str {
    fn put(self, g: &mut Sre, s: Sid) {
        put_str(g, s, self);
    }
}
impl Put for u8 {
    fn put(self, g: &mut Sre, s: Sid) {
        put_char(g, s, self);
    }
}
impl Put for i32 {
    fn put(self, g: &mut Sre, s: Sid) {
        put_num(g, s, self);
    }
}
impl Put for i16 {
    fn put(self, g: &mut Sre, s: Sid) {
        put_num(g, s, self as i32);
    }
}
impl Put for u16 {
    fn put(self, g: &mut Sre, s: Sid) {
        put_num(g, s, self as u32 as i32);
    }
}
impl Put for Color {
    fn put(self, g: &mut Sre, s: Sid) {
        put_color(g, s, self);
    }
}
impl Put for SetW {
    fn put(self, g: &mut Sre, s: Sid) {
        g.ost(s).width = self.0;
    }
}
impl Put for &AttrString {
    fn put(self, g: &mut Sre, s: Sid) {
        put_attr(g, s, self);
    }
}
impl Put for Untr {
    fn put(self, g: &mut Sre, s: Sid) {
        put_untranslated(g, s, &self.0);
    }
}

// out!(g, stream; item, item, ...): the original's "stream << item << item".
#[macro_export]
macro_rules! out {
    ($g:expr, $s:expr; $($x:expr),* $(,)?) => {{
        let __s: $crate::stream::Sid = $s;
        $( $crate::stream::Put::put($x, $g, __s); )*
    }};
}

// kout!(g; item, ...): to kout.
#[macro_export]
macro_rules! kout {
    ($g:expr; $($x:expr),* $(,)?) => { $crate::out!($g, $crate::stream::KOUT; $($x),*) };
}

// ---------------------------------------------------------------------
// Input streams.

pub enum IKind {
    Key(crate::devices::KeyIn), // kin (c0d1)
    Window { pending: i16 },    // kwinin (bb49)
    File(FileIn),               // be27
}

pub struct IStream {
    pub maxlen: i16, // max characters for line input
    pub defval: i32, // default for numeric input, -1 = none
    pub maxval: i32, // max for numeric input, -1 = none
    pub kind: IKind,
}

impl IStream {
    pub fn new(kind: IKind) -> IStream {
        IStream { maxlen: 0, defval: -1, maxval: -1, kind }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ITag {
    Key,
    Window,
    File,
}

fn itag(g: &mut Sre, i: Iid) -> ITag {
    match g.ist(i).kind {
        IKind::Key(_) => ITag::Key,
        IKind::Window { .. } => ITag::Window,
        IKind::File(_) => ITag::File,
    }
}

pub fn echo(g: &mut Sre, i: Iid, c: u8) {
    match itag(g, i) {
        ITag::Key => put_char(g, KOUT, c),     // 247e:026b
        ITag::Window => put_char(g, KWIN, c),  // 1fd3:0322
        ITag::File => {}
    }
}

pub fn key_ready(g: &mut Sre, i: Iid) -> bool {
    match itag(g, i) {
        ITag::Key => crate::devices::kin_key_ready(g),
        ITag::Window => crate::conio::kbhit(g),
        ITag::File => match &g.ist(i).kind {
            IKind::File(f) => f.is_open() && f.err == 0,
            _ => false,
        },
    }
}

pub fn get_key(g: &mut Sre, i: Iid) -> u8 {
    match itag(g, i) {
        ITag::Key => crate::devices::kin_get_key(g),
        ITag::Window => crate::conio::window_get_key(g),
        ITag::File => crate::files::ifstream_get_key(g, i),
    }
}

fn begin_input(g: &mut Sre, i: Iid) {
    if itag(g, i) == ITag::Key {
        crate::devices::kin_begin_input(g);
    }
}

fn end_input(g: &mut Sre, i: Iid) {
    if itag(g, i) == ITag::Key {
        crate::devices::kin_end_input(g);
    }
}

pub fn set_maxlen(g: &mut Sre, i: Iid, n: i16) {
    g.ist(i).maxlen = n;
}
pub fn set_defval(g: &mut Sre, i: Iid, v: i32) {
    g.ist(i).defval = v;
}
pub fn set_maxval(g: &mut Sre, i: Iid, v: i32) {
    g.ist(i).maxval = v;
}

// 229f:006c: line input; buf may hold a default that is shown first and
// replaced if the first key typed is printable.
pub fn get_line(g: &mut Sre, i: Iid, buf: &mut Str) {
    let mut n: i8 = 0;
    let mut first = true;
    // The buffer as the original's char array: text, NUL.
    let mut b: Vec<u8> = cstr(buf).to_vec();
    b.push(0);
    let at = |b: &Vec<u8>, k: i8| -> u8 { b.get(k as u8 as usize).cloned().unwrap_or(0) };
    let set = |b: &mut Vec<u8>, k: i8, c: u8| {
        let k = k as u8 as usize;
        if b.len() <= k {
            b.resize(k + 1, 0);
        }
        b[k] = c;
    };

    begin_input(g, i);
    while at(&b, n) != 0 {
        let c = at(&b, n);
        echo(g, i, c);
        n = n.wrapping_add(1);
    }
    loop {
        let mut c = get_key(g, i);
        if c == 9 {
            c = b' ';
        }
        if first && c >= 0x20 && c != 0x7f {
            while n > 0 {
                echo(g, i, 8);
                echo(g, i, b' ');
                echo(g, i, 8);
                n -= 1;
            }
            set(&mut b, 0, 0);
        }
        first = false;
        if c == 8 {
            if n > 0 {
                n -= 1;
                set(&mut b, n, 0);
                echo(g, i, 8);
                echo(g, i, b' ');
                echo(g, i, 8);
            }
        } else if (0x20..=0xfe).contains(&c) && c != 0x7f {
            let maxlen = g.ist(i).maxlen;
            if (n as i16) < maxlen || maxlen == 0 {
                set(&mut b, n, c);
                n = n.wrapping_add(1);
                set(&mut b, n, 0);
                echo(g, i, c);
            }
        }
        if c == b'\n' || c == b'\r' || c == 0 {
            break;
        }
    }
    echo(g, i, b'\n');
    end_input(g, i);
    *buf = cstr(&b).to_vec();
}

fn echo_back(g: &mut Sre, i: Iid) {
    echo(g, i, 8);
    echo(g, i, b' ');
    echo(g, i, 8);
}

// 229f:02ec: numeric input with default, maximum, "%", ">" and K/M.
pub fn get_number(g: &mut Sre, i: Iid, mut max: i32) -> i32 {
    let mut n: i8 = 0;
    let mut v: i32 = -1;
    let mut digits_typed = false;
    let savedmax = g.ist(i).maxlen;
    let mut pending: Vec<u8> = Vec::new();
    let mut c: u8;

    let maxval = g.ist(i).maxval;
    if !(maxval >= max) && maxval >= 0 {
        max = maxval;
    }
    {
        let mut t = max;
        let mut ml: i16 = 1;
        if t == 0 {
            ml = 2;
        }
        while t > 0 {
            ml += 1;
            t /= 10;
        }
        g.ist(i).maxlen = ml;
    }
    begin_input(g, i);
    g.ist(i).maxlen = savedmax;
    let defval = g.ist(i).defval;
    if defval >= 0 {
        pending = b_ltoa(defval, 10);
    }
    let mut first = true;
    loop {
        'next: {
            if pending.is_empty() {
                c = b_toupper(get_key(g, i));
                if first && (ctype_signed(c) & (CT_DIG | CT_UPP | CT_LOW)) != 0 {
                    first = false;
                    pending = vec![8u8; n.max(0) as usize];
                    pending.push(c);
                    break 'next;
                }
                first = false;
            } else {
                c = pending.remove(0);
            }

            let maxval = g.ist(i).maxval;
            if c == 8 && n > 0 {
                v = if v > 9 { v / 10 } else { -1 };
                echo_back(g, i);
                n -= 1;
                digits_typed = true;
            } else if c == b'%' && (0..=100).contains(&v) && maxval >= 0 {
                while n > 0 {
                    echo_back(g, i);
                    n -= 1;
                }
                let mut r = if maxval > 10000000 { (maxval / 100).wrapping_mul(v) } else { v.wrapping_mul(maxval) / 100 };
                if !(maxval >= r) {
                    r = maxval;
                }
                v = r;
                n = 0;
                let mut t = v;
                while t > 0 {
                    n = n.wrapping_add(1);
                    t /= 10;
                }
                let mut k: i16 = n as i16 - 1;
                while k >= 0 {
                    let mut t = r;
                    for _ in 0..k {
                        t /= 10;
                    }
                    echo(g, i, b'0' + (t % 10) as u8);
                    k -= 1;
                }
                digits_typed = true;
            } else if c.is_ascii_digit() {
                let mut t: i32;
                if v == -1 {
                    t = (c - b'0') as i32;
                    if t > max {
                        t = -1;
                    }
                    if t >= 0 {
                        n = n.wrapping_add(1);
                        echo(g, i, c);
                        v = t;
                    }
                } else {
                    if v == 0 {
                        echo(g, i, 8);
                        n = n.wrapping_sub(1);
                    }
                    t = v.wrapping_mul(10).wrapping_add((c - b'0') as i32);
                    if t > max {
                        t = -1;
                    } else if t / 10 != v {
                        t = -1;
                    }
                    if t >= 0 {
                        n = n.wrapping_add(1);
                        echo(g, i, c);
                        v = t;
                    } else if v == 0 {
                        v = -1;
                    }
                }
                digits_typed = true;
            } else if c == b'>' && maxval >= 0 {
                while n > 0 {
                    echo_back(g, i);
                    n -= 1;
                }
                v = maxval;
                n = 0;
                let mut t = v;
                while t > 0 {
                    n = n.wrapping_add(1);
                    t /= 10;
                }
                let mut k: i16 = n as i16 - 1;
                while k >= 0 {
                    let mut t = maxval;
                    for _ in 0..k {
                        t /= 10;
                    }
                    echo(g, i, b'0' + (t % 10) as u8);
                    k -= 1;
                }
                v = maxval;
                digits_typed = true;
            } else if c == b'K' || c == b'M' {
                if digits_typed {
                    if c == b'K' {
                        pending.extend_from_slice(b"000");
                    }
                    if c == b'M' {
                        pending.extend_from_slice(b"000000");
                    }
                }
            } else {
                digits_typed = false;
            }
        }
        if c == b'\n' || c == b'\r' || c == 0 {
            break;
        }
    }

    if n == 0 {
        v = -1;
    }
    if v == -1 {
        let defval = g.ist(i).defval;
        let tmp = if defval == -1 { Vec::new() } else { b_ltoa(defval, 10) };
        for &ch in &tmp {
            echo(g, i, ch);
        }
        v = defval;
    }
    g.ist(i).defval = -1;
    g.ist(i).maxval = -1;
    end_input(g, i);
    echo(g, i, b'\n');
    v
}

// 229f:029d
pub fn in_char(g: &mut Sre, i: Iid) -> u8 {
    get_key(g, i)
}

// 229f:02c1
pub fn in_line(g: &mut Sre, i: Iid) -> Str {
    let mut s = Str::new();
    get_line(g, i, &mut s);
    s
}

// 229f:0ad7
pub fn in_i32(g: &mut Sre, i: Iid) -> i32 {
    get_number(g, i, 0x7fffffff)
}

// 229f:0b18
pub fn in_i16(g: &mut Sre, i: Iid) -> i16 {
    let r = get_number(g, i, 0x7fff);
    if r == -1 {
        -1
    } else {
        r as i16
    }
}

// 229f:0b73
#[allow(dead_code)] // unused, as in the original
pub fn in_u16(g: &mut Sre, i: Iid) -> u16 {
    get_number(g, i, 0xffff) as u16
}

// 229f:0baf
#[allow(dead_code)] // unused, as in the original
pub fn in_u8(g: &mut Sre, i: Iid) -> u8 {
    get_number(g, i, 0xff) as u8
}
