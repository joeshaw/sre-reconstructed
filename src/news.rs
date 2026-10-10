// News and the sysop's log (segment 5b6a).

use crate::brtl::b_time;
use crate::files::{ofstream, ofstream_done, read_line, ifstream, ifstream_done, ifstream_fail};
use crate::resource::Resource;
use crate::screens::put_coded;
use crate::sre::Sre;
use crate::sstring::{at, cat, erase, insert, str_to_long, substr, Str};
use crate::stream::*;
use crate::types::*;
use crate::out;

// The news stream (vtable 683f, DS:d138): each line written becomes a
// news item in NEWS/NEWS0.MSG, with colours as "^\BF" codes.
pub fn new_news_stream() -> OStream {
    OStream::new(0, Kind::News(Str::new()))
}

// The sysop's log (vtable 685f, DS:d160): lines go to SYSOP.LOG.
pub fn new_sysop_log() -> OStream {
    let mut o = OStream::new(0xff, Kind::SysopLog(AttrString::new(0xff)));
    o.noansi = 1;
    o.curfg = 0xe;
    o
}

// 5b6a:04d6: length without colour codes.
fn visible_len(s: &[u8]) -> i16 {
    let mut n: i16 = 0;
    let mut i: i16 = 0;
    while i < s.len() as i16 {
        if at(s, i) == b'^' && at(s, i + 1) == b'\\' && at(s, i + 2) != 0 && at(s, i + 3) != 0 {
            i += 3;
        } else {
            n += 1;
        }
        i += 1;
    }
    n
}

// 5b6a:055a: split s at a space so that the first part is at most width
// visible characters; a short last part is padded to width.
fn split_line(s: &[u8], width: i16) -> (Str, Str) {
    let mut brk: i16 = -1;
    let mut vis: i16 = 0;
    let mut i: i16 = 0;
    while i < s.len() as i16 {
        if (ctype_signed(at(s, i)) & CT_SP) != 0 && i >= width / 2 {
            brk = i;
        }
        if at(s, i) == b'^' && at(s, i + 1) == b'\\' && at(s, i + 2) != 0 && at(s, i + 3) != 0 {
            if i >= width / 2 {
                brk = i;
            }
            i += 3;
            i += 1;
            continue;
        }
        if vis >= width {
            if brk == -1 {
                brk = vis;
            }
            let a = substr(s, 0, brk);
            while ctype_signed(at(s, brk)) & CT_SP != 0 {
                brk += 1;
            }
            let b = substr(s, brk, -1);
            return (a, b);
        }
        vis += 1;
        i += 1;
    }
    let mut a = s.to_vec();
    while vis < width {
        a.push(b' ');
        vis += 1;
    }
    (a, Str::new())
}

// 5b6a:0705
fn format_news(s: &mut Str, first: &[u8], cont: &[u8], eol1: &[u8], eol2: &[u8]) {
    let mut eol = eol1;
    *s = cat(&[first, s]);
    let mut outs = Str::new();
    loop {
        let (a, b) = split_line(s, 0x4b - visible_len(eol));
        outs.extend_from_slice(&a);
        outs.extend_from_slice(eol);
        outs.push(b'\n');
        eol = eol2;
        if b.is_empty() {
            break;
        }
        *s = cat(&[cont, &b]);
    }
    *s = outs;
}

fn news_line(g: &mut Sre) -> &mut Str {
    match &mut g.ost(NEWS).kind {
        Kind::News(l) => l,
        _ => panic!("not the news stream"),
    }
}

// 5b6a:0929
pub fn news_put_raw(g: &mut Sre, c: u8) -> u8 {
    if c == b'\r' {
        return c;
    }
    if c != b'\n' {
        news_line(g).push(c);
        return c;
    }
    let mut line = std::mem::take(news_line(g));
    if at(&line, 0) == b'\\' {
        erase(&mut line, 0, 1);
    } else if at(&line, 0) == b'^' && at(&line, 1) == b'\\' && at(&line, 4) == b'\\' {
        erase(&mut line, 4, 1);
    } else if !line.is_empty() {
        format_news(&mut line, b"^\\01\xc4^\\09\xc4^\\03\xc4^\\0B\xc4^\\07", b"^\\01 \xfa  ^\\07", b"", b"");
    }
    let n = line.len() as i16;
    insert(&mut line, n, b"\n");
    let f = ofstream(g, b"NEWS/NEWS0.MSG", true, 0);
    let now = b_time();
    let comma = g.comma;
    g.comma = 0;
    out!(g, f; "#date ", now, b'\n');
    g.comma = comma;
    out!(g, f; line);
    ofstream_done(g, f);
    c
}

// 5b6a:0cf7
pub fn sysoplog_put_raw(g: &mut Sre, c: u8) -> u8 {
    g.ost(ERRLOG).curfg = 0xe;
    if c == b'\n' {
        let text = g.ost(ERRLOG).line.clone();
        if let Kind::SysopLog(t) = &mut g.ost(ERRLOG).kind {
            *t = text.clone();
        }
        let f = ofstream(g, b"SYSOP.LOG", true, 0);
        g.ost(f).noansi = 1;
        out!(g, f; &text, b'\n');
        ofstream_done(g, f);
    }
    c
}

// 5b6a:0000: show a news file from the given date on.
pub fn show_news_to(g: &mut Sre, o: Sid, input: Option<Iid>, file: &[u8], since: i32) -> bool {
    let f = ifstream(g, file, false);
    if ifstream_fail(g, f) {
        ifstream_done(g, f);
        return false;
    }
    let mut show = true;
    out!(g, o; col(7));
    while !ifstream_fail(g, f) {
        let mut line = read_line(g, f);
        if let Some(i) = input {
            if key_ready(g, i) {
                let c = in_char(g, i);
                if c == b' ' {
                    break;
                }
            }
        }
        if substr(&line, 0, 5) == b"#date" {
            erase(&mut line, 0, 6);
            let t = str_to_long(&line);
            show = t >= since;
            continue;
        }
        if show {
            put_coded(g, &line, o);
            out!(g, o; "\n");
        }
    }
    out!(g, o; col(7));
    ifstream_done(g, f);
    true
}

// 5b6a:01fb
pub fn show_news(g: &mut Sre, file: &[u8], since: i32) -> bool {
    show_news_to(g, KOUT, Some(KIN), file, since)
}

// 5b6a:0228: copy news file n into the files named by News<n>.File.*.
pub fn news_files(g: &mut Sre, n: i16) {
    let digit = [(n as u8).wrapping_add(b'0')];
    let msg = cat(&[b"News/News", &digit, b".MSG"]);
    let ansikey = cat(&[b"News", &digit, b".File.ANSI"]);
    let textkey = cat(&[b"News", &digit, b".File.Text"]);
    let mut ra = Resource::text(&ansikey, Some(b""));
    let mut rt = Resource::text(&textkey, Some(b""));
    let a = ra.str(&mut g.resdb);
    if !a.is_empty() {
        let f = ofstream(g, &a, false, 0);
        g.ost(f).noansi = 0;
        show_news_to(g, f, None, &msg, 0);
        ofstream_done(g, f);
    }
    let t = rt.str(&mut g.resdb);
    if !t.is_empty() {
        let f = ofstream(g, &t, false, 0);
        g.ost(f).noansi = 1;
        show_news_to(g, f, None, &msg, 0);
        ofstream_done(g, f);
    }
}

// 5b6a:0bc4: read a key until it is one of valid; Enter gives valid[0].
pub fn get_choice(g: &mut Sre, valid: &[u8], upper: bool) -> u8 {
    let mut def = 0;
    let valid: &[u8] = if valid.is_empty() || valid[0] == 0 {
        if valid.is_empty() {
            valid
        } else {
            &valid[1..]
        }
    } else {
        def = valid[0];
        valid
    };
    loop {
        let mut c = in_char(g, KIN);
        if c == b'\r' {
            c = def;
        }
        if upper {
            c = b_toupper(c);
        }
        if c != 0 && valid.contains(&c) {
            return c;
        }
    }
}
