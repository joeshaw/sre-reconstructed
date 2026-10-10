// Terminal devices of the SRCOMM door library (segment 247e).
//
// kout is a multiplexing output stream: text is word-wrapped once (at 79
// columns) and the resulting bytes are copied to the sysop's screen, the
// remote user's modem and an optional log file.  In this reconstruction
// the "modem" is the user's terminal (stdout/stdin).  Without a remote
// user (local play and the sysop functions) the terminal is the sysop's
// local screen and keyboard instead.

use crate::brtl::{b_delay, b_time};
use crate::sre::Sre;
use crate::stream::*;
use crate::{kout, out};

// kout's own fields (vtable c0b5, DS:d3b8).
#[derive(Default)]
pub struct MultiOut {
    pub logopen: i16,  // +20 (d3d8)
    pub logon: i16,    // +22 (d3da) copy output to the log
    pub mode: i16,     // +24 (d3dc)
    pub localon: i16,  // +26 (d3de) copy output to the local screen
    pub remoteon: i16, // +28 (d3e0)
}

// kin's own fields (vtable c0d1, DS:d3a6).
pub struct KeyIn {
    pub disabled: i16, // +10 (d3b6)
    pub savefg: i16,   // +0c
    pub savebg: i16,   // +0e
}

// The device globals (DS:bee0..bf19).
pub struct DevState {
    pub timeout: i16,    // bee4 input timeout in seconds
    pub lastremote: i16, // bee6
    pub ansi: i16,       // bee8
    pub remote: i16,     // beea com port, 0 = local
    pub nonstop: i16,    // beec
    pub ibm: i16,        // beee
    pub effects: i16,    // bef0 SpecialEffects
    pub linecount: i16,  // bef2
    pub pagelen: i16,    // bef4
    pub breakch: u8,     // bef6
    pub stopch: u8,      // bef7
    pub c_normal: Color, // bef8
    pub c_input: Color,  // befc
    pub sbback: u8,      // bf04 status bar background
    pub sbmid: u8,       // bf05
    pub sbfore: u8,      // bf06
    pub pushback: i16,   // bf19
    pub console: bool,   // the host terminal is the local console
    pub idle: Option<fn(&mut Sre)>, // bee0
    pub keyready_calls: i16, // DS:bf0b
    pub term: Term,
}

impl Default for DevState {
    fn default() -> Self {
        DevState {
            timeout: 60,
            lastremote: 0,
            ansi: 0,
            remote: 0,
            nonstop: 0,
            ibm: 1,
            effects: 0,
            linecount: 0,
            pagelen: 0x15,
            breakch: 3,
            stopch: 4,
            c_normal: col2(7, 0),
            c_input: col2(15, -1),
            sbback: 0,
            sbmid: 7,
            sbfore: 15,
            pushback: 0,
            console: false,
            idle: None,
            keyready_calls: 0,
            term: Term::default(),
        }
    }
}

// ---------------------------------------------------------------------
// Host terminal: stdout/stdin stand in for the FOSSIL driver.

pub struct Term {
    saved: Option<libc::termios>,
    outbuf: Vec<u8>,
    peek: i32,
    // Test scripts (SRE_TESTMARK set) may hold back input with a NUL
    // byte: what follows is only seen once the program waits for a key.
    pub testmark: bool,
    waiting: bool,
    utf8: Option<bool>,
}

impl Default for Term {
    fn default() -> Self {
        Term {
            saved: None,
            outbuf: Vec::with_capacity(4096),
            peek: -2,
            testmark: std::env::var_os("SRE_TESTMARK").is_some(),
            waiting: false,
            utf8: None,
        }
    }
}

pub fn term_restore(g: &mut Sre) {
    term_flush(g);
    if let Some(t) = g.dev.term.saved.take() {
        // SAFETY: restoring the attributes read from fd 0.
        unsafe { libc::tcsetattr(0, libc::TCSANOW, &t) };
    }
}

pub fn term_init(g: &mut Sre) {
    // SAFETY: plain termios calls on fd 0.
    unsafe {
        let mut t: libc::termios = std::mem::zeroed();
        if libc::isatty(0) != 0 && libc::tcgetattr(0, &mut t) == 0 {
            let saved = t;
            t.c_iflag &= !(libc::ICRNL | libc::INLCR | libc::IXON | libc::ISTRIP);
            t.c_lflag &= !(libc::ICANON | libc::ECHO | libc::ISIG | libc::IEXTEN);
            t.c_cc[libc::VMIN] = 1;
            t.c_cc[libc::VTIME] = 0;
            libc::tcsetattr(0, libc::TCSANOW, &t);
            g.dev.term.saved = Some(saved);
        }
    }
}

pub fn term_flush(g: &mut Sre) {
    let buf = std::mem::take(&mut g.dev.term.outbuf);
    let mut off = 0;
    while off < buf.len() {
        // SAFETY: writing from a live buffer.
        let r = unsafe { libc::write(1, buf[off..].as_ptr() as *const libc::c_void, buf.len() - off) };
        if r <= 0 {
            if r < 0 && std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            break;
        }
        off += r as usize;
    }
    let mut buf = buf;
    buf.clear();
    g.dev.term.outbuf = buf;
}

// The game's text is in the IBM PC character set (code page 437).  A
// terminal that expects UTF-8 (as macOS's does) gets it translated:
// SRE_CHARSET=utf8 or cp437 chooses; by default a UTF-8 locale on a
// terminal does.
const CP437: [&str; 128] = [
    "Ç", "ü", "é", "â", "ä", "à", "å", "ç", "ê", "ë", "è", "ï", "î", "ì", "Ä", "Å", //
    "É", "æ", "Æ", "ô", "ö", "ò", "û", "ù", "ÿ", "Ö", "Ü", "¢", "£", "¥", "₧", "ƒ", //
    "á", "í", "ó", "ú", "ñ", "Ñ", "ª", "º", "¿", "⌐", "¬", "½", "¼", "¡", "«", "»", //
    "░", "▒", "▓", "│", "┤", "╡", "╢", "╖", "╕", "╣", "║", "╗", "╝", "╜", "╛", "┐", //
    "└", "┴", "┬", "├", "─", "┼", "╞", "╟", "╚", "╔", "╩", "╦", "╠", "═", "╬", "╧", //
    "╨", "╤", "╥", "╙", "╘", "╒", "╓", "╫", "╪", "┘", "┌", "█", "▄", "▌", "▐", "▀", //
    "α", "ß", "Γ", "π", "Σ", "σ", "µ", "τ", "Φ", "Θ", "Ω", "δ", "∞", "φ", "ε", "∩", //
    "≡", "±", "≥", "≤", "⌠", "⌡", "÷", "≈", "°", "∙", "·", "√", "ⁿ", "²", "■", "\u{a0}", //
];

fn utf8_terminal(g: &mut Sre) -> bool {
    if let Some(u) = g.dev.term.utf8 {
        return u;
    }
    let env = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
    let u = match env("SRE_CHARSET") {
        Some(cs) => cs.contains("utf") || cs.contains("UTF"),
        None => {
            let loc = env("LC_ALL").or_else(|| env("LC_CTYPE")).or_else(|| env("LANG"));
            // SAFETY: isatty on fd 1.
            let tty = unsafe { libc::isatty(1) } != 0;
            tty && loc.map_or(false, |l| {
                l.contains("UTF-8") || l.contains("utf8") || l.contains("UTF8") || l.contains("utf-8")
            })
        }
    };
    g.dev.term.utf8 = Some(u);
    u
}

fn term_put(g: &mut Sre, c: u8) {
    if g.dev.term.outbuf.len() == 4096 {
        term_flush(g);
    }
    g.dev.term.outbuf.push(c);
}

pub fn term_write(g: &mut Sre, s: &[u8]) {
    let utf8 = utf8_terminal(g);
    for &c in s {
        if c >= 0x80 && utf8 {
            for &b in CP437[(c - 0x80) as usize].as_bytes() {
                term_put(g, b);
            }
        } else {
            term_put(g, c);
        }
    }
}

// Text for the screen: control characters other than CR, LF, BS and BEL
// are the PC's glyphs.
pub fn term_write_glyphs(g: &mut Sre, s: &[u8]) {
    const GLYPHS: [&str; 32] = [
        " ", "\u{263a}", "\u{263b}", "\u{2665}", "\u{2666}", "\u{2663}", "\u{2660}", "\u{2022}", //
        "\u{25d8}", "\u{25cb}", "\u{25d9}", "\u{2642}", "\u{2640}", "\u{266a}", "\u{266b}", "\u{263c}", //
        "\u{25ba}", "\u{25c4}", "\u{2195}", "\u{203c}", "\u{00b6}", "\u{00a7}", "\u{25ac}", "\u{21a8}", //
        "\u{2191}", "\u{2193}", "\u{2192}", "\u{2190}", "\u{221f}", "\u{2194}", "\u{25b2}", "\u{25bc}", //
    ];
    if !utf8_terminal(g) {
        term_write(g, s);
        return;
    }
    for &c in s {
        if c < 0x20 && c != b'\r' && c != b'\n' && c != 8 && c != 7 {
            for &b in GLYPHS[c as usize].as_bytes() {
                term_put(g, b);
            }
        } else {
            term_write(g, &[c]);
        }
    }
}

// 1 if a character (or end of input) is waiting.
pub fn term_ready(g: &mut Sre) -> bool {
    if g.dev.term.peek != -2 {
        return true;
    }
    term_flush(g);
    let mut p = libc::pollfd { fd: 0, events: libc::POLLIN, revents: 0 };
    // SAFETY: polling one valid pollfd.
    unsafe { libc::poll(&mut p, 1, 0) > 0 }
}

// The next byte of input; -1 at the end.
pub fn term_getc(g: &mut Sre) -> i32 {
    if g.dev.term.peek != -2 {
        let c = g.dev.term.peek;
        g.dev.term.peek = -2;
        return c;
    }
    term_flush(g);
    let mut c = 0u8;
    loop {
        // SAFETY: reading one byte into a local.
        let r = unsafe { libc::read(0, &mut c as *mut u8 as *mut libc::c_void, 1) };
        if r == 1 {
            return c as i32;
        }
        if r < 0 && std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
            continue;
        }
        return -1;
    }
}

// The next term_getc without taking it; -2 if none yet.
pub fn term_peek(g: &mut Sre) -> i32 {
    if g.dev.term.peek == -2 {
        if !term_ready(g) {
            return -2;
        }
        g.dev.term.peek = term_getc(g);
    }
    g.dev.term.peek
}

// Carrier: lost when the user's input stream ends.
fn carrier(g: &mut Sre) -> bool {
    if g.dev.term.peek == -1 {
        return false;
    }
    if term_ready(g) && g.dev.term.peek == -2 {
        g.dev.term.peek = term_getc(g);
        if g.dev.term.peek == -1 {
            return false;
        }
    }
    true
}

fn remote_char_ready(g: &mut Sre) -> bool {
    loop {
        if !term_ready(g) {
            return false;
        }
        if g.dev.term.peek == -2 {
            g.dev.term.peek = term_getc(g);
        }
        if g.dev.term.peek == 0 && g.dev.term.testmark {
            if !g.dev.term.waiting {
                return false;
            }
            g.dev.term.peek = -2;
            continue;
        }
        return g.dev.term.peek >= 0;
    }
}

// The host terminal is the local keyboard when nobody is connected.
fn local_key_ready(g: &mut Sre) -> bool {
    g.dev.console && g.dev.remote == 0 && remote_char_ready(g)
}

fn remote_getc(g: &mut Sre) -> u8 {
    let c = term_getc(g);
    if c < 0 {
        0
    } else {
        c as u8
    }
}

// ---------------------------------------------------------------------
// 247e:10a7: the modem; converts IBM box characters when the user has
// no IBM character set.
pub fn modem_raw_write(g: &mut Sre, s: &[u8]) {
    if g.dev.remote == 0 {
        return;
    }
    if g.dev.ibm == 0 {
        let conv: Vec<u8> = s
            .iter()
            .map(|&c| match c {
                0xb3 | 0xba => b'|',
                0xc4 => b'-',
                0xcd => b'=',
                c if c > 0x7f => b'#',
                c => c,
            })
            .collect();
        term_write(g, &conv);
    } else {
        term_write(g, s);
    }
}

// 247e:0fda: the original writes to its local screen window (or the raw
// screen in mode 1).
pub fn local_raw_write(g: &mut Sre, s: &[u8]) {
    if g.dev.console && g.dev.remote == 0 {
        term_write(g, s);
    }
}

// ---------------------------------------------------------------------
// 247e:0c6a
pub fn new_kout() -> OStream {
    OStream::new(0x4f, Kind::Multi(MultiOut { logopen: 0, logon: 0, localon: 1, remoteon: 1, mode: 0 }))
}

pub fn kout_m(g: &mut Sre) -> &mut MultiOut {
    g.ost(KOUT).multi()
}

// 247e:0d2b
pub fn multi_emit_color(g: &mut Sre) {
    let o = g.ost(KOUT);
    o.curfg = o.newfg;
    o.curbg = o.newbg;
    let (fg, bg) = (o.newfg as i16, o.newbg as i16);
    if g.dev.ansi == 0 {
        return;
    }
    if kout_m(g).localon != 0 {
        out!(g, KLOCAL; col2(fg, bg));
    }
    out!(g, KMODEM; col2(fg, bg));
}

// 247e:0db4
pub fn open_log(g: &mut Sre, name: &[u8]) {
    if kout_m(g).logopen != 0 {
        crate::files::ofstream_close(g, KLOG);
    }
    kout_m(g).logopen = 1;
    crate::files::ofstream_open(g, KLOG, name, false);
    if g.dev.ansi == 0 {
        g.ost(KLOG).noansi = 1;
    }
    kout_m(g).logon = 1;
}

// 247e:0e07
pub fn close_log(g: &mut Sre) {
    crate::files::ofstream_close(g, KLOG);
    kout_m(g).logopen = 0;
    kout_m(g).logon = 0;
}

// 247e:0e2a
pub fn multi_raw_write(g: &mut Sre, s: &[u8]) {
    let mut pause = false;
    if g.ost(KOUT).raw == 0 {
        for &c in s {
            if c == b'\n' {
                g.dev.linecount = g.dev.linecount.wrapping_add(1);
            }
        }
        if g.dev.linecount > g.dev.pagelen && g.dev.pagelen > 12 {
            g.dev.linecount = 0;
            if g.dev.nonstop == 0 {
                pause = true;
            }
        }
    }
    let m = kout_m(g);
    let (logon, localon, remoteon) = (m.logon, m.localon, m.remoteon);
    if logon != 0 {
        raw_write(g, KLOG, s);
    }
    if localon != 0 {
        write(g, KLOCAL, s);
    }
    if remoteon != 0 && g.dev.remote != 0 {
        write(g, KMODEM, s);
    }
    if pause && kin_disabled(g) == 0 {
        pause_prompt(g);
    }
}

pub fn set_mode(g: &mut Sre, m: i16) -> i16 {
    kout_m(g).mode = m;
    m
}

// ---------------------------------------------------------------------
pub fn new_kin() -> IStream {
    IStream::new(IKind::Key(KeyIn { disabled: 0, savefg: 7, savebg: 0 }))
}

pub fn kin_k(g: &mut Sre) -> &mut KeyIn {
    match &mut g.ist(KIN).kind {
        IKind::Key(k) => k,
        _ => panic!("not kin"),
    }
}

pub fn kin_disabled(g: &mut Sre) -> i16 {
    kin_k(g).disabled
}

// 247e:0285
pub fn kin_begin_input(g: &mut Sre) {
    let (fg, bg) = (g.ost(KOUT).newfg as i16, g.ost(KOUT).newbg as i16);
    kin_k(g).savefg = fg;
    kin_k(g).savebg = bg;
    let ci = g.dev.c_input;
    kout!(g; ci);
    if ci.bg <= 0 {
        return;
    }
    if g.dev.ansi == 0 {
        return;
    }
    let savedraw = g.ost(KOUT).raw;
    let savedmode = kout_m(g).mode;
    g.ost(KOUT).raw = 1;
    set_mode(g, 1);
    let maxlen = g.ist(KIN).maxlen;
    if maxlen > 0 {
        for _ in 0..maxlen {
            kout!(g; b' ');
        }
        if g.dev.ansi != 0 {
            kout!(g; "\x1b[", maxlen, b'D');
        } else {
            for _ in 0..maxlen {
                kout!(g; 8u8);
            }
        }
    }
    if savedraw == 0 {
        g.ost(KOUT).raw = 0;
    }
    set_mode(g, savedmode);
}

// 247e:039e
pub fn kin_end_input(g: &mut Sre) {
    let (fg, bg) = (kin_k(g).savefg, kin_k(g).savebg);
    kout!(g; col2(fg, bg));
}

// 247e:17dd: set the function called while waiting for keys; the old one.
pub fn set_idle(g: &mut Sre, f: Option<fn(&mut Sre)>) -> Option<fn(&mut Sre)> {
    std::mem::replace(&mut g.dev.idle, f)
}

// 247e:03ce
fn idle(g: &mut Sre) {
    if let Some(f) = g.dev.idle {
        f(g);
    }
}

// 247e:03e9
pub fn kin_key_ready(g: &mut Sre) -> bool {
    let calls = g.dev.keyready_calls;
    g.dev.keyready_calls = calls.wrapping_add(1);
    if calls == 10 {
        idle(g);
        g.dev.keyready_calls = 0;
    }
    // Carrier loss is detected by the blocking read in GetKey; here the
    // end of the input just means no key.
    if g.dev.remote != 0 && !carrier(g) {
        return false;
    }
    if kin_disabled(g) != 0 {
        return false;
    }
    if g.dev.remote != 0 && remote_char_ready(g) {
        return true;
    }
    if local_key_ready(g) && kout_m(g).localon != 0 {
        return true;
    }
    false
}

// 247e:04d9
pub fn kin_get_key(g: &mut Sre) -> u8 {
    g.dev.term.waiting = true;
    let c = get_key_waiting(g);
    g.dev.term.waiting = false;
    c
}

fn get_key_waiting(g: &mut Sre) -> u8 {
    let cursor: u8 = if g.dev.ibm != 0 { 0xdb } else { b'_' };
    let back: &[u8] = if g.dev.ansi != 0 { b"\x1b[D" } else { b"\x08" };
    let savedraw = g.ost(KOUT).raw;
    let savedmode = kout_m(g).mode;
    let savefg = g.ost(KOUT).newfg as i16;
    let savebg = g.ost(KOUT).newbg as i16;
    if kin_disabled(g) != 0 {
        return 0;
    }
    g.dev.linecount = 0;
    if g.dev.pushback != 0 {
        let c = g.dev.pushback as u8;
        g.dev.pushback = 0;
        return c;
    }
    set_mode(g, 1);
    if savedraw == 0 {
        g.ost(KOUT).raw = 1;
        kout!(g; col2(savefg, 0), cursor, back);
        g.ost(KOUT).raw = 0;
    }
    let mut warned = false;
    let mut start = b_time();
    let mut last = b_time();
    while start.wrapping_add(g.dev.timeout as i32 + 30) > last {
        if g.dev.remote != 0 && !carrier(g) {
            out!(g, KLOCAL; col(14), "Hangup?\n");
            g.exit(1);
        }
        if g.dev.console && g.dev.remote == 0 && !carrier(g) {
            g.exit(1); // the end of the host's input
        }
        if kin_key_ready(g) && g.dev.remote != 0 && remote_char_ready(g) {
            g.dev.lastremote = 1;
            let mut c = remote_getc(g) as i8 as i16;
            if savedraw == 0 && c == 0x7f {
                c = 8;
            }
            if savedraw == 0 {
                kout!(g; " \x08");
            }
            let userbreak = g.res_bool_userbreak();
            let prompt: Option<&str> = if c == g.dev.breakch as i8 as i16 && userbreak {
                Some("\nBreak out of Program?  [y/N]")
            } else if c == g.dev.stopch as i8 as i16 && userbreak {
                Some("\nStop Program?  [y/N]")
            } else {
                None
            };
            let prompt = match prompt {
                None => {
                    set_mode(g, savedmode);
                    kout!(g; col2(savefg, savebg));
                    return c as u8;
                }
                Some(p) => p,
            };
            let saved = g.ost(KOUT).line.clone();
            let fg2 = g.ost(KOUT).newfg as i16;
            let bg2 = g.ost(KOUT).newbg as i16;
            kout!(g; col2(7, 0), prompt);
            loop {
                let k = in_char(g, KIN);
                if k == b'Y' || k == b'y' {
                    if c == g.dev.breakch as i8 as i16 {
                        interrupt(g, 2);
                    } else {
                        g.exit(0);
                    }
                }
                if k == b'N' || k == b'n' || k == b'\r' {
                    break;
                }
            }
            if savedraw == 0 {
                kout!(g; col(7), &saved, col2(fg2, bg2));
            }
            if savedraw == 0 {
                g.ost(KOUT).raw = 1;
                kout!(g; cursor, back);
                g.ost(KOUT).raw = 0;
            }
            continue;
        }
        if local_key_ready(g) {
            // The local keyboard: no break prompt, and the host's DEL is
            // the PC's backspace.  (Extended keys only reached the
            // sysop's hot keys.)
            g.dev.lastremote = 0;
            let mut c = remote_getc(g) as i8 as i16;
            if c == 0x7f {
                c = 8;
            }
            if savedraw == 0 {
                kout!(g; " \x08");
            }
            if c == g.dev.breakch as i8 as i16 {
                interrupt(g, 2);
            } else if c == g.dev.stopch as i8 as i16 {
                g.exit(0);
            } else if kout_m(g).localon != 0 {
                set_mode(g, savedmode);
                kout!(g; col2(savefg, savebg));
                return c as u8;
            }
            if savedraw == 0 {
                g.ost(KOUT).raw = 1;
                kout!(g; cursor, back);
                g.ost(KOUT).raw = 0;
            }
            continue;
        }
        let now = b_time();
        if now.wrapping_sub(last) > 5 {
            start = start.wrapping_add(now.wrapping_sub(last));
        }
        last = now;
        if (g.dev.timeout as i32) <= last.wrapping_sub(start) && !warned {
            let saved = g.ost(KOUT).line.clone();
            let fg2 = g.ost(KOUT).newfg as i16;
            let bg2 = g.ost(KOUT).newbg as i16;
            if savedraw == 0 {
                kout!(g; col2(14, 0), " \n\x07Warning!  Program will abort in 30 seconds if you do not respond...\n");
                kout!(g; col(7), &saved, col2(fg2, bg2));
            } else {
                kout!(g; "\x07\x07\x07");
            }
            warned = true;
        }
        idle(g);
        term_flush(g);
        if !term_ready(g) {
            crate::brtl::b_sleep(15);
        }
    }
    let port = g.dev.remote as i32;
    g.dev.remote = 0;
    kout!(g; col2(14, 0), b" \x07\n\xaeTIMEOUT:  COM", port, ": no input for ", col(15),
          last.wrapping_sub(start), col(14), b" seconds\xaf\n");
    g.exit(2);
}

// ---------------------------------------------------------------------
// 247e:11d7: the "PAUSED" prompt; waits for a key.
pub fn pause_prompt(g: &mut Sre) {
    let mut spin: i16 = 0;
    if kin_disabled(g) != 0 {
        return;
    }
    let savedlog = kout_m(g).logon;
    kout!(g; setw(0));
    if kout_m(g).logopen != 0 {
        kout_m(g).logon = 0;
    }
    let savedraw = g.ost(KOUT).raw;
    g.ost(KOUT).raw = 1;
    if key_ready(g, KIN) {
        in_char(g, KIN);
    } else {
        let savefg = g.ost(KOUT).newfg as i16;
        let savebg = g.ost(KOUT).newbg as i16;
        let (sbback, sbmid, sbfore) = (g.dev.sbback as i16, g.dev.sbmid as i16, g.dev.sbfore as i16);
        if g.dev.ibm != 0 {
            kout!(g; col(sbback), b"\xb0\xb1\xb2\xdb", col2(sbfore, sbback), "PAUSED");
            kout!(g; col2(sbback, 0), b"\xdb\xb2\xb1\xb0", col(13), "  ");
        } else {
            kout!(g; col(sbback), b"\xc4\xc4", col(sbmid), b"\xc4\xc4", col(sbfore), "Paused", col(sbmid),
                  b"\xc4\xc4", col(sbback), b"\xc4\xc4", col(13), "  ");
        }
        while g.dev.effects != 0 && !key_ready(g, KIN) && spin < 100 {
            if g.dev.ibm != 0 {
                kout!(g; col(11), "\x08/");
                b_delay(50);
                kout!(g; col(1), "\x08X");
                b_delay(50);
                kout!(g; col(14), "\x08\\");
                b_delay(50);
                kout!(g; col(4), "\x08X");
            } else {
                kout!(g; col(14), "\x08.");
                b_delay(0x24);
                kout!(g; col(12), "\x08o");
                b_delay(50);
                kout!(g; col(4), "\x08O");
            }
            b_delay(50);
            spin += 1;
        }
        kout!(g; "\x08 ");
        if !key_ready(g, KIN) {
            kout!(g; col(15));
        }
        in_char(g, KIN);
        kout!(g; col2(savefg, savebg));
        for _ in 0..16 {
            kout!(g; "\x08 \x08");
        }
    }
    g.dev.linecount = 0;
    if savedraw == 0 {
        g.ost(KOUT).raw = 0;
    }
    if kout_m(g).logopen != 0 {
        kout_m(g).logon = savedlog;
    }
}

// 247e:1627: interrupt (Ctrl-Break) handler.
pub fn interrupt(g: &mut Sre, code: i16) {
    g.dev.pagelen = 0;
    kout!(g; b'\n', col2(12, 0), b'[', col(14), "INTERRUPT", col(12), b']', col(7), b'\n');
    if code != 999 {
        g.exit(1);
    }
}
