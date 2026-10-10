// Files of streamable objects (segment 1629).
//
// A stream file is a sequence of records, each a 0x50-byte header (data
// size, checksum of the class name, the class name, checksum of the
// data) followed by the encrypted data.  Reading creates objects by
// class name through a registry.  A StreamFile reads the old file and
// writes the objects to keep into a temporary file, which replaces the
// original when it is closed.

use crate::brtl::b_stricmp;
use crate::crypt::{check_checksum, make_checksum, unscramble};
use crate::files::{host_path, host_remove, host_rename};
use crate::game::Checksum;
use crate::layout::Le;
use crate::sre::Sre;
use crate::sstring::{cat, cstr, Str};
use crate::stream::*;
use crate::timeutil::put_time;
use crate::out;
use std::any::Any;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

// Base of the streamable classes (vtable 3836).  The data of an object
// is Size() - 2 bytes (the 2 being the original's vtable pointer).
pub trait Streamable: Any {
    fn class_name(&self) -> &'static [u8]; // [04]
    fn size(&self) -> i16; // [08] 1629:000c
    fn expired(&mut self, _g: &mut Sre) -> bool {
        true // [0c] 1629:0016
    }
    fn on_empire_deleted(&mut self, _g: &mut Sre, _id: i16) {} // [10] 1629:0020
    // The object's data area: its bytes, and setting them.
    fn data(&self) -> Vec<u8>;
    fn set_data(&mut self, b: &[u8]);
    fn as_any(&mut self) -> &mut dyn Any;
}

pub type CreateFn = fn() -> Box<dyn Streamable>;

// The registry of classes (static constructors, DS:43f0), the last
// registered first.
pub fn classes() -> &'static [(&'static [u8], CreateFn)] {
    &[
        (crate::trading::TRADE_CLASS, crate::trading::new_trading_fleet),
        (crate::report::REPORT_CLASS, crate::report::new_report),
        (crate::messages::MESSAGE_CLASS, crate::messages::new_message),
        (crate::events::CLS_BANK, crate::events::new_bank),
        (crate::events::CLS_SETUP, crate::events::new_setup),
        (crate::events::CLS_HOSTAGES, crate::events::new_hostages),
        (crate::events::CLS_DEMORALIZE, crate::events::new_demoralize),
        (crate::events::CLS_DISSENSION, crate::events::new_dissension),
    ]
}

// 1629:004e
pub fn create_object(g: &mut Sre, name: &[u8]) -> Option<Box<dyn Streamable>> {
    for (n, f) in classes() {
        if b_stricmp(n, name) == 0 {
            return Some(f());
        }
    }
    let now = g.time.now;
    put_time(g, ERRLOG, now);
    out!(g, ERRLOG; " Tried to create object of type ", name.to_vec(), ".\n");
    None
}

// Borland's errno values for the cases the game reports.
fn dos_errno(e: &std::io::Error) -> i16 {
    match e.raw_os_error() {
        Some(libc::ENOENT) => 2,
        Some(libc::EMFILE) | Some(libc::ENFILE) => 4,
        Some(libc::EACCES) | Some(libc::EPERM) => 5,
        _ => 6,
    }
}

// File open modes, as Borland's O_ flags.
pub const FM_READ: i16 = 1;
pub const FM_WRITE: i16 = 2;
pub const FM_APPEND: i16 = 0x800;

// A file handle (1629:0b4d).
pub struct RawFile {
    pub fd: Option<File>,
    pub name: Str,
    pub err: i16,
}

impl RawFile {
    pub fn closed() -> RawFile {
        RawFile { fd: None, name: Str::new(), err: 1 }
    }

    pub fn new(g: &mut Sre, name: &[u8], mode: i16) -> RawFile {
        let mut f = RawFile::closed();
        f.open(g, name, mode);
        f
    }

    // 1629:0c06: open (always creating), binary.
    pub fn open(&mut self, g: &mut Sre, n: &[u8], mode: i16) {
        let mut o = OpenOptions::new();
        if mode & FM_READ != 0 {
            o.read(true);
            // O_RDONLY | O_CREAT
            o.write(false);
        } else if mode & FM_WRITE != 0 {
            o.write(true);
        } else {
            o.read(true);
        }
        if mode & FM_APPEND != 0 {
            o.append(true);
        }
        let p = host_path(n);
        let r = if mode & FM_READ != 0 {
            // O_CREAT with read-only access: create it if missing.
            if !p.exists() {
                let _ = OpenOptions::new().write(true).create(true).truncate(false).open(&p);
            }
            o.open(&p)
        } else {
            o.create(true).open(&p)
        };
        self.err = 0;
        match r {
            Ok(f) => self.fd = Some(f),
            Err(e) => {
                self.fd = None;
                self.err = dos_errno(&e);
                let now = g.time.now;
                if self.err == 2 {
                    put_time(g, ERRLOG, now);
                    out!(g, ERRLOG; " File '", n.to_vec(), "' not openable.\n");
                }
                if self.err == 4 {
                    put_time(g, ERRLOG, now);
                    out!(g, ERRLOG; " Too many files open.  Try increasing FILES= in CONFIG.SYS.\n");
                }
            }
        }
        if mode & FM_WRITE != 0 {
            if let Some(f) = self.fd.as_mut() {
                let _ = f.seek(SeekFrom::End(0));
            }
        }
        self.name = n.to_vec();
    }

    // 1629:0d2c
    pub fn close(&mut self) {
        if self.fd.is_none() {
            return;
        }
        self.fd = None;
        self.err = 0;
        self.name.clear();
    }

    // 1629:0d73
    pub fn read(&mut self, buf: &mut [u8]) -> bool {
        let f = match self.fd.as_mut() {
            None => return false,
            Some(f) => f,
        };
        let mut got = 0;
        while got < buf.len() {
            match f.read(&mut buf[got..]) {
                Ok(0) => break,
                Ok(n) => got += n,
                Err(e) => {
                    self.err = dos_errno(&e);
                    return false;
                }
            }
        }
        if got < buf.len() {
            self.err = 1;
            return false;
        }
        true
    }

    // 1629:0dd2
    pub fn write(&mut self, g: &mut Sre, buf: &[u8]) -> bool {
        let f = match self.fd.as_mut() {
            None => return false,
            Some(f) => f,
        };
        match f.write(buf) {
            Err(e) => {
                self.err = dos_errno(&e);
                false
            }
            Ok(n) if n < buf.len() => {
                let now = g.time.now;
                put_time(g, ERRLOG, now);
                let name = self.name.clone();
                out!(g, ERRLOG; " Error writing to file ' ", name, "', disk full?\n");
                self.err = 1;
                false
            }
            Ok(_) => true,
        }
    }

    // 1629:0e9b
    pub fn skip(&mut self, n: i16) {
        if let Some(f) = self.fd.as_mut() {
            let _ = f.seek(SeekFrom::Current(n as i64));
        }
    }

    // 1629:0ec1
    pub fn tell(&mut self) -> i32 {
        match self.fd.as_mut() {
            None => 0,
            Some(f) => f.stream_position().map(|p| p as i32).unwrap_or(0),
        }
    }

    // 1629:0ee5
    pub fn seek(&mut self, pos: i32) {
        if let Some(f) = self.fd.as_mut() {
            let _ = f.seek(SeekFrom::Start(pos as u32 as u64));
        }
    }
}

// The record header (0x50 bytes).
struct RecordHeader {
    size: i16,
    namesum: Checksum,
    name: [u8; 0x1e],
    datasum: Checksum,
}

impl RecordHeader {
    fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0u8; 0x50];
        self.size.wr(&mut b[0..]);
        self.namesum.wr(&mut b[2..]);
        b[0x1a..0x38].copy_from_slice(&self.name);
        self.datasum.wr(&mut b[0x38..]);
        b
    }
    fn from_bytes(b: &[u8]) -> RecordHeader {
        let mut name = [0u8; 0x1e];
        name.copy_from_slice(&b[0x1a..0x38]);
        RecordHeader { size: i16::rd(&b[0..]), namesum: Checksum::rd(&b[2..]), name, datasum: Checksum::rd(&b[0x38..]) }
    }
}

fn log_file(g: &mut Sre, f: &RawFile, pre: &str, post: &str) {
    let now = g.time.now;
    put_time(g, ERRLOG, now);
    let name = f.name.clone();
    out!(g, ERRLOG; pre, name, post);
}

// 1629:0102: the next object of the file, or None at the end.
pub fn read_object(g: &mut Sre, f: &mut RawFile) -> Option<Box<dyn Streamable>> {
    loop {
        let mut hb = vec![0u8; 0x50];
        if !f.read(&mut hb) {
            return None;
        }
        let mut h = RecordHeader::from_bytes(&hb);
        if h.size < 0 {
            log_file(g, f, " File `", "' negative packet size, skipping record.\n");
            return None;
        }
        if !check_checksum(&mut h.name, &h.namesum) {
            log_file(g, f, " File `", "' header corrupted, skipping record.\n");
            f.skip(h.size);
            continue;
        }
        h.name[0x1d] = 0;
        let cname = cstr(&h.name).to_vec();
        let mut o = match create_object(g, &cname) {
            Some(o) => o,
            None => {
                log_file(g, f, " Trying to read from file `", "', couldn't match ");
                out!(g, ERRLOG; "class header {", cname, "}.  It may mean `out of memory', ");
                out!(g, ERRLOG; "but it's probably Amit's fault.\n");
                f.skip(h.size);
                continue;
            }
        };
        let mut d = o.data();
        let clear = (o.size() as i32 - 2).max(0) as usize;
        for b in d.iter_mut().take(clear) {
            *b = 0;
        }
        let n = h.size as usize;
        if d.len() < n {
            d.resize(n, 0);
        }
        if !f.read(&mut d[..n]) {
            log_file(g, f, " Trying to read from file `", "', couldn't read bytes.\n");
            f.close();
        }
        if !check_checksum(&mut d[..n], &h.datasum) {
            log_file(g, f, " Data security check failed.  File `", "' could be corrupt.\n");
            continue;
        }
        o.set_data(&d);
        return Some(o);
    }
}

// 1629:059c
pub fn write_object(g: &mut Sre, f: &mut RawFile, o: &mut dyn Streamable) {
    let mut name = [0u8; 0x1e];
    let cn = o.class_name();
    for (i, &c) in cn.iter().take(0x1d).enumerate() {
        name[i] = c;
    }
    name[0x1d] = 0;
    let size = (o.size() as i32 - 2) as i16;
    let mut d = o.data();
    let n = size as u16 as usize;
    if d.len() < n {
        d.resize(n, 0);
    }
    let datasum = make_checksum(&mut d[..n]);
    let namesum = make_checksum(&mut name);
    let h = RecordHeader { size, namesum, name, datasum };
    f.write(g, &h.to_bytes());
    f.write(g, &d[..n]);
    unscramble(&mut d[..n], &datasum);
    o.set_data(&d);
}

pub struct StreamFile {
    pub inp: RawFile,  // +00
    pub outp: RawFile, // +0c
    pub path: Str,     // +18
    pub temp: Str,     // +20
}

impl StreamFile {
    // 1629:07c7
    pub fn new(g: &mut Sre, name: &[u8]) -> StreamFile {
        let path = cat(&[name, b".II"]);
        let inp = RawFile::new(g, &path, FM_READ);
        let temp = cat(&[name, b".$$$"]);
        host_remove(&temp);
        let outp = RawFile::new(g, &temp, FM_WRITE | FM_APPEND);
        StreamFile { inp, outp, path, temp }
    }

    pub fn read(&mut self, g: &mut Sre) -> Option<Box<dyn Streamable>> {
        read_object(g, &mut self.inp)
    }

    pub fn write(&mut self, g: &mut Sre, o: &mut dyn Streamable) {
        write_object(g, &mut self.outp, o);
    }

    // Write o back unless it has expired (the pattern after each read).
    pub fn keep(&mut self, g: &mut Sre, o: &mut dyn Streamable) {
        if !o.expired(g) {
            self.write(g, o);
        }
    }

    // 1629:095d: the end of the StreamFile's scope.
    pub fn close(mut self, g: &mut Sre) {
        if self.outp.err == 0 {
            self.inp.close();
            self.outp.close();
            host_remove(&self.path);
            host_rename(&self.temp, &self.path);
        } else {
            self.inp.close();
            self.outp.close();
            host_remove(&self.temp);
            let now = g.time.now;
            put_time(g, ERRLOG, now);
            let p = self.path.clone();
            out!(g, ERRLOG; " Error writing to ", p, ".\n");
        }
    }
}

// 1629:0ac6: append an object to a stream file.
pub fn append_object(g: &mut Sre, name: &[u8], o: &mut dyn Streamable) {
    let p = cat(&[name, b".II"]);
    let mut f = RawFile::new(g, &p, FM_WRITE | FM_APPEND);
    write_object(g, &mut f, o);
}
