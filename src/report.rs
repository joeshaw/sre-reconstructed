// Reports: messages to empires, kept in stream files (segment 69d7).

use crate::game::{Player, MAXEMPIRES};
use crate::layout::{record, Le};
use crate::sre::Sre;
use crate::streamfile::{append_object, StreamFile, Streamable};
use crate::stream::*;
use crate::timeutil::put_time;
use crate::types::b_toupper;
use crate::kout;
use std::any::Any;

pub const REPORT_FILE: &[u8] = b"Data/Report"; // DS:91e3
pub const REPORT_CLASS: &[u8] = b"Report"; // DS:91ce

record! {
    pub struct ReportData {
        pub created: i32,        // 02
        pub expires: i32,        // 06
        pub count: i16,          // 0a
        pub to: [i16; 25],       // 0c recipients
        pub len: u16,            // 3e
        pub text: [u8; 0x200],   // 40
    }
}

// Class "Report" (vtable 9269, 0x240 bytes).
pub struct Report {
    pub d: ReportData,
}

pub fn new_report() -> Box<dyn Streamable> {
    Box::new(Report::new_raw(0))
}

impl Report {
    fn new_raw(now: i32) -> Report {
        let mut d = ReportData::default();
        d.created = now;
        d.expires = now.wrapping_add(0x93a80);
        d.count = 0;
        d.len = 0;
        d.to = [-1; 25];
        Report { d }
    }

    // 69d7:022f
    pub fn new(g: &Sre) -> Report {
        Report::new_raw(g.time.now)
    }

    // 69d7:018d
    pub fn has_recipient(&self, id: i16) -> bool {
        (0..self.d.count.max(0) as usize).any(|i| self.d.to[i] == id)
    }

    // 69d7:0571
    pub fn add_recipient(&mut self, id: i16) -> i16 {
        for i in 0..self.d.count.max(0) as usize {
            if self.d.to[i] == id {
                return id;
            }
        }
        let n = self.d.count as usize;
        if n < 25 {
            self.d.to[n] = id;
        }
        self.d.count += 1;
        id
    }

    // 69d7:01c9 / 05c5 (all occurrences)
    pub fn remove_recipient(&mut self, id: i16) -> i16 {
        let mut i: i16 = 0;
        while i < self.d.count {
            if self.d.to[i as usize] == id {
                self.d.count -= 1;
                self.d.to[i as usize] = self.d.to[self.d.count as usize];
                i -= 1;
            }
            i += 1;
        }
        self.d.count
    }

    // 69d7:0624
    pub fn add_all(&mut self, g: &Sre) {
        for i in 0..MAXEMPIRES {
            if g.emp[i].exists() {
                self.add_recipient(i as i16);
            }
        }
    }

    // 69d7:066e
    pub fn clear_recipients(&mut self) {
        self.d.count = 0;
    }

    // 69d7:0362: "──[n]──", with the date if wanted.
    pub fn show_header(&self, g: &mut Sre, n: i16, showtime: bool, c1: i16, c2: i16) {
        kout!(g; col(c1), b"\xc4\xc4[", col(c2), n, col(c1), b"]\xc4\xc4");
        if showtime {
            kout!(g; "      ", col(8));
            put_time(g, KOUT, self.d.created);
        }
        let cn = g.dev.c_normal;
        kout!(g; cn, b'\n');
    }

    // 69d7:0454: the text, with "^\BF" colour codes.
    pub fn show(&self, g: &mut Sre) {
        let cn = g.dev.c_normal;
        kout!(g; cn);
        let t = &self.d.text;
        let at = |i: usize| t.get(i).cloned().unwrap_or(0);
        let mut i: u16 = 0;
        while i < self.d.len {
            let c = at(i as usize);
            if c == b'^' && at(i as usize + 1) == b'\\' {
                let mut cl = col2(7, -1);
                cl.bg = hex_digit(at(i as usize + 2), cl.bg);
                cl.fg = hex_digit(at(i as usize + 3), cl.fg);
                kout!(g; cl);
                i += 3;
            } else {
                kout!(g; c);
            }
            i += 1;
        }
        kout!(g; b'\n');
    }
}

fn hex_digit(c: u8, def: i16) -> i16 {
    let c = b_toupper(c);
    if c.is_ascii_digit() {
        return (c - b'0') as i16;
    }
    if (b'A'..=b'F').contains(&c) {
        return (c - b'A' + 10) as i16;
    }
    def
}

impl Streamable for Report {
    // 69d7:005f
    fn class_name(&self) -> &'static [u8] {
        REPORT_CLASS
    }
    // 69d7:006d: as long as the text
    fn size(&self) -> i16 {
        (self.d.len as i32 + 0x41) as i16
    }
    // 69d7:007e: expired when nobody is left to read it, or after a week.
    fn expired(&mut self, g: &mut Sre) -> bool {
        if g.open != 0 {
            for i in 0..MAXEMPIRES {
                if g.emp[i].name[0] == 0 {
                    self.remove_recipient(i as i16);
                }
            }
        }
        if self.d.count == 0 {
            return true;
        }
        g.time.now > self.d.expires
    }
    // 69d7:01c9
    fn on_empire_deleted(&mut self, _g: &mut Sre, id: i16) {
        self.remove_recipient(id);
    }
    fn data(&self) -> Vec<u8> {
        self.d.to_bytes()
    }
    fn set_data(&mut self, b: &[u8]) {
        self.d = ReportData::rd(b);
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

// ---------------------------------------------------------------------
// An output stream that collects a report and files it when it is done
// (vtable 924d, 0x264 bytes).
pub struct ReportOut {
    pub rep: Report,       // +20
    pub file: &'static [u8], // +260
}

// 69d7:089d (no recipient)
pub fn report_out(g: &mut Sre, file: Option<&'static [u8]>) -> Sid {
    let r = ReportOut { rep: Report::new(g), file: file.unwrap_or(REPORT_FILE) };
    let s = g.alloc_os(OStream::new(0, Kind::Report(Box::new(r))));
    g.ost(s).raw = 1;
    s
}

// 69d7:0920
pub fn report_out_to(g: &mut Sre, id: i16, file: Option<&'static [u8]>) -> Sid {
    if id < 0 || id as usize >= MAXEMPIRES {
        crate::out!(g, ERRLOG; "Warning:  ReportOut received ID ", id, " in constructor.\n");
    }
    let s = report_out(g, file);
    rep_of(g, s).rep.add_recipient(id);
    s
}

pub fn rep_of(g: &mut Sre, s: Sid) -> &mut ReportOut {
    match &mut g.ost(s).kind {
        Kind::Report(r) => r,
        _ => panic!("not a report stream"),
    }
}

// The report's date.
pub fn report_set_time(g: &mut Sre, s: Sid, t: i32) {
    rep_of(g, s).rep.d.created = t;
}

// 69d7:09eb: the end of the ReportOut's scope: file the report.
pub fn report_done(g: &mut Sre, s: Sid) {
    let os = g.free_os(s);
    if let Kind::Report(mut r) = os.kind {
        let file = r.file;
        append_object(g, file, &mut r.rep);
        if file == REPORT_FILE {
            g.newreports = 1;
        }
    }
}

// 69d7:0a7b
pub fn reportout_put_raw(g: &mut Sre, s: Sid, c: u8) -> u8 {
    let r = rep_of(g, s);
    if r.rep.d.len >= 0x1ff {
        return 0;
    }
    let l = r.rep.d.len as usize;
    r.rep.d.text[l] = c;
    r.rep.d.len += 1;
    r.rep.d.text[l + 1] = 0;
    c
}

// 69d7:067c: show the reports for an empire and remove it from their
// recipients.
pub fn show_reports(g: &mut Sre, id: i16, file: &[u8], showtime: bool, c1: i16, c2: i16) -> i16 {
    let mut n: i16 = 0;
    let mut f = StreamFile::new(g, file);
    while let Some(mut o) = f.read(g) {
        if let Some(r) = o.as_any().downcast_mut::<Report>() {
            if r.has_recipient(id) {
                n += 1;
                r.show_header(g, n, showtime, c1, c2);
                r.show(g);
                r.remove_recipient(id);
            }
        }
        f.keep(g, o.as_mut());
    }
    f.close(g);
    n
}

// 69d7:0777
pub fn show_new_reports(g: &mut Sre, p: &Player, e: usize) -> i16 {
    if g.newreports == 0 {
        return 0;
    }
    g.newreports = 0;
    let id = g.emp[e].id;
    show_reports(g, id, REPORT_FILE, (p.prefs2 >> 3) & 1 != 0, 1, 3)
}

// 69d7:07bb
pub fn reports_empire_deleted(g: &mut Sre, id: i16, file: Option<&[u8]>) -> i16 {
    let mut n: i16 = 0;
    let mut f = StreamFile::new(g, file.unwrap_or(REPORT_FILE));
    while let Some(mut o) = f.read(g) {
        o.on_empire_deleted(g, id);
        n += 1;
        f.keep(g, o.as_mut());
    }
    f.close(g);
    n
}

// 69d7:0b6f: empty a stream file.
pub fn clear_stream_file(g: &mut Sre, file: Option<&[u8]>) {
    let f = StreamFile::new(g, file.unwrap_or(REPORT_FILE));
    f.close(g);
}
