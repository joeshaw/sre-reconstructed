// File streams and record files of the SRCOMM library (segment 2144).

use crate::brtl::b_stricmp;
use crate::sre::Sre;
use crate::sstring::{cstr, erase, Str};
use crate::stream::*;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};

// ---------------------------------------------------------------------
// Path mapping.  The game uses DOS paths relative to its directory with
// either separator; on the host each component is matched without
// regard to case, and new names are created as given.

pub fn host_path(dospath: &[u8]) -> std::path::PathBuf {
    use std::os::unix::ffi::OsStrExt;
    let mut p = cstr(dospath);
    if p.len() >= 2 && p[1] == b':' {
        p = &p[2..];
    }
    let mut out: Vec<u8> = b".".to_vec();
    for comp in p.split(|&c| c == b'\\' || c == b'/') {
        if comp.is_empty() || comp == b"." {
            continue;
        }
        let mut found: Vec<u8> = comp.to_ascii_uppercase();
        if let Ok(rd) = std::fs::read_dir(std::ffi::OsStr::from_bytes(&out)) {
            for e in rd.flatten() {
                let name = e.file_name();
                if b_stricmp(name.as_bytes(), comp) == 0 {
                    found = name.as_bytes().to_vec();
                    break;
                }
            }
        }
        out.push(b'/');
        out.extend_from_slice(&found);
    }
    std::path::PathBuf::from(std::ffi::OsStr::from_bytes(&out))
}

pub fn host_remove(dospath: &[u8]) -> i32 {
    let p = host_path(dospath);
    // remove(): a file, or an empty directory.
    if std::fs::remove_file(&p).is_ok() || std::fs::remove_dir(&p).is_ok() {
        0
    } else {
        -1
    }
}

pub fn host_rename(from: &[u8], to: &[u8]) -> i32 {
    if std::fs::rename(host_path(from), host_path(to)).is_ok() {
        0
    } else {
        -1
    }
}

pub fn host_mkdir(dospath: &[u8]) -> i32 {
    if std::fs::create_dir(host_path(dospath)).is_ok() {
        0
    } else {
        -1
    }
}

// The file error handler (1848:0067 logs to SYSOP.LOG).
pub type FileErrorFn = fn(&mut Sre, &[u8]);

pub fn set_file_error_handler(g: &mut Sre, f: Option<FileErrorFn>) {
    g.fileerror = f;
}

// 2144:1054: expand %s and pass to the error logger.
fn file_error(g: &mut Sre, fmt: &[u8], name: &[u8]) {
    let mut out = Str::new();
    let mut i = 0;
    while i < fmt.len() {
        if fmt[i] == b'%' && fmt.get(i + 1) == Some(&b's') {
            out.extend_from_slice(cstr(name));
            i += 1;
        } else {
            out.push(fmt[i]);
        }
        i += 1;
    }
    if let Some(f) = g.fileerror {
        f(g, &out);
    }
}

// ---------------------------------------------------------------------
// Output text file (vtable be43): an OStream whose bytes go to a file.

#[derive(Default)]
pub struct FileOut {
    pub fp: Option<BufWriter<File>>,
    pub err: i16,
}

fn fileout(g: &mut Sre, s: Sid) -> &mut FileOut {
    match &mut g.ost(s).kind {
        Kind::File(f) => f,
        _ => panic!("not a file stream"),
    }
}

// An OFStream: OFStream(name, append, binary, wrap).
pub fn ofstream(g: &mut Sre, name: &[u8], append: bool, wrap: i16) -> Sid {
    let s = g.alloc_os(OStream::new(wrap, Kind::File(FileOut::default())));
    ofstream_open(g, s, name, append);
    s
}

// 2144:0234
pub fn ofstream_open(g: &mut Sre, s: Sid, name: &[u8], append: bool) -> i16 {
    ofstream_close(g, s);
    let p = host_path(name);
    let f = if append {
        OpenOptions::new().append(true).create(true).open(&p)
    } else {
        File::create(&p)
    };
    match f {
        Ok(f) => {
            let fo = fileout(g, s);
            fo.fp = Some(BufWriter::new(f));
            fo.err = 0;
            0
        }
        Err(_) => {
            fileout(g, s).err = 2;
            file_error(g, b"File '%s' not openable for output.\n", name);
            -1
        }
    }
}

pub fn ofstream_close(g: &mut Sre, s: Sid) {
    let fo = fileout(g, s);
    if let Some(mut f) = fo.fp.take() {
        let _ = f.flush();
    }
    fo.err = 0;
}

// Close and free a file stream (the end of the OFStream's scope).
pub fn ofstream_done(g: &mut Sre, s: Sid) {
    ofstream_close(g, s);
    g.free_os(s);
}

// 2144:03fa
pub fn ofstream_put_raw(g: &mut Sre, s: Sid, c: u8) -> u8 {
    let fo = fileout(g, s);
    match fo.fp.as_mut() {
        None => 0,
        Some(f) => {
            if f.write_all(&[c]).is_err() {
                0
            } else {
                c
            }
        }
    }
}

// ---------------------------------------------------------------------
// Input file (vtable be27): an IStream reading characters from a file.
// Text mode strips every CR and stops at ^Z, like Borland's read().

#[derive(Default)]
pub struct FileIn {
    pub fp: Option<BufReader<File>>,
    pub err: i16, // 1 at end of file
    pub binary: bool,
}

impl FileIn {
    pub fn is_open(&self) -> bool {
        self.fp.is_some()
    }
    pub fn fail(&self) -> bool {
        self.fp.is_none() || self.err != 0
    }

    // 2144:06c4
    pub fn open(name: &[u8], binary: bool) -> FileIn {
        match File::open(host_path(name)) {
            Ok(f) => FileIn { fp: Some(BufReader::new(f)), err: 0, binary },
            Err(_) => FileIn { fp: None, err: 2, binary },
        }
    }

    // 2144:07e5 (with Borland text-mode read(): CRs dropped, ^Z = EOF)
    pub fn getc(&mut self) -> u8 {
        let fp = match self.fp.as_mut() {
            None => return 0,
            Some(f) => f,
        };
        loop {
            let mut b = [0u8; 1];
            match fp.read(&mut b) {
                Ok(1) => {}
                _ => {
                    self.err = 1;
                    return 0;
                }
            }
            let c = b[0];
            if !self.binary {
                if c == b'\r' {
                    continue;
                }
                if c == 0x1a {
                    self.err = 1;
                    return 0;
                }
            }
            return c;
        }
    }
}

// An IFStream as an input stream (for code that reads it like kin).
pub fn ifstream(g: &mut Sre, name: &[u8], binary: bool) -> Iid {
    g.alloc_is(IStream::new(IKind::File(FileIn::open(name, binary))))
}

pub fn filein(g: &mut Sre, i: Iid) -> &mut FileIn {
    match &mut g.ist(i).kind {
        IKind::File(f) => f,
        _ => panic!("not a file stream"),
    }
}

pub fn ifstream_fail(g: &mut Sre, i: Iid) -> bool {
    filein(g, i).fail()
}

pub fn ifstream_done(g: &mut Sre, i: Iid) {
    g.free_is(i);
}

pub fn ifstream_get_key(g: &mut Sre, i: Iid) -> u8 {
    filein(g, i).getc()
}

// 260c:0009: read one line (tabs become spaces, backspace edits).
pub fn read_line(g: &mut Sre, i: Iid) -> Str {
    let mut s = Str::new();
    loop {
        let mut c = in_char(g, i);
        if c == 9 {
            c = b' ';
        }
        if c == 8 {
            let n = s.len() as i16 - 1;
            erase(&mut s, n, 1);
        } else if (0x20..=0xfe).contains(&c) && c != 0x7f {
            s.push(c);
        }
        if c == b'\n' || c == b'\r' || c == 0 {
            break;
        }
    }
    s
}

// The same for a FileIn used directly.
pub fn read_line_file(f: &mut FileIn) -> Str {
    let mut s = Str::new();
    loop {
        let mut c = f.getc();
        if c == 9 {
            c = b' ';
        }
        if c == 8 {
            let n = s.len() as i16 - 1;
            erase(&mut s, n, 1);
        } else if (0x20..=0xfe).contains(&c) && c != 0x7f {
            s.push(c);
        }
        if c == b'\n' || c == b'\r' || c == 0 {
            break;
        }
    }
    s
}

// ---------------------------------------------------------------------
// Record file with XOR scrambling (2144:096f).

pub struct DataFile {
    pub name: Str,
    pub fp: Option<File>,
    pub recsize: i16,
    pub err: i16,
    pub key: u8,
    pub modn: u8,
}

impl DataFile {
    pub fn new(g: &mut Sre, name: &[u8], recsize: i16, key: u8, modn: u8) -> DataFile {
        let mut d = DataFile { name: Str::new(), fp: None, recsize, err: 0, key, modn: if modn < 1 { 1 } else { modn } };
        if !name.is_empty() {
            d.open(g, name);
        }
        d
    }

    // 2144:0a56
    pub fn open(&mut self, g: &mut Sre, n: &[u8]) -> i16 {
        if self.fp.is_some() {
            self.close();
        }
        let hp = host_path(n);
        self.fp = OpenOptions::new().read(true).write(true).open(&hp).ok();
        if self.fp.is_none() {
            self.fp = OpenOptions::new().read(true).write(true).create(true).truncate(true).open(&hp).ok();
        }
        if self.fp.is_none() {
            self.err = 2;
            file_error(g, b"File '%s' not openable.\n", n);
        }
        let n = cstr(n);
        self.name = n[..n.len().min(0x7f)].to_vec();
        0
    }

    pub fn close(&mut self) {
        if self.fp.is_none() {
            return;
        }
        self.fp = None;
        self.err = 0;
        self.name.clear();
    }

    // 2144:0b5c: symmetric scrambling of count records.
    pub fn scramble(&self, buf: &mut [u8], count: i16) {
        if self.key == 0 && self.modn == 1 {
            return;
        }
        let mut p = self.recsize as i32 as isize * count as isize;
        let mut r = count - 1;
        while r >= 0 {
            let mut j = self.recsize - 1;
            while j >= 0 {
                p -= 1;
                buf[p as usize] ^= self.key.wrapping_add((j % self.modn as i16) as u8);
                j -= 1;
            }
            r -= 1;
        }
    }

    // 2144:0f6d
    pub fn numrecs(&mut self) -> i32 {
        let recsize = self.recsize;
        match self.fp.as_mut() {
            None => 0,
            Some(f) => {
                let len = f.metadata().map(|m| m.len()).unwrap_or(0) as i64;
                (len / recsize as i64) as i32
            }
        }
    }

    // 2144:0be7
    pub fn read(&mut self, rec: i32, buf: &mut [u8], count: i16) -> bool {
        let recsize = self.recsize;
        let f = match self.fp.as_mut() {
            None => return false,
            Some(f) => f,
        };
        if f.seek(SeekFrom::Start(rec as i64 as u64 * recsize as i64 as u64)).is_err() {
            self.err = 6;
            return false;
        }
        let want = (recsize as i32 * count as i32) as i16 as u16 as usize;
        let mut got = 0;
        while got < want {
            match f.read(&mut buf[got..want]) {
                Ok(0) | Err(_) => break,
                Ok(n) => got += n,
            }
        }
        if (got as i16) < want as i16 {
            self.err = 1;
            return false;
        }
        self.scramble(buf, count);
        true
    }

    // 2144:0d28
    pub fn write(&mut self, g: &mut Sre, rec: i32, buf: &[u8], count: i16) -> bool {
        let recsize = self.recsize;
        let want = (recsize as i32 * count as i32) as i16 as u16 as usize;
        let mut tmp = buf[..want].to_vec();
        self.scramble(&mut tmp, count);
        let f = match self.fp.as_mut() {
            None => return false,
            Some(f) => f,
        };
        if f.seek(SeekFrom::Start(rec as i64 as u64 * recsize as i64 as u64)).is_err() {
            self.err = 6;
            return false;
        }
        if f.write_all(&tmp).is_err() {
            let name = self.name.clone();
            file_error(g, b"Error writing to file '%s', disk full?\n", &name);
            self.err = 1;
            return false;
        }
        true
    }

    // 2144:0fcf: delete the file and reopen it empty.
    #[allow(dead_code)] // unused, as in the original
    pub fn erase(&mut self, g: &mut Sre) -> bool {
        let n = self.name.clone();
        self.close();
        let r = host_remove(&n);
        self.open(g, &n);
        r != -1
    }

}
