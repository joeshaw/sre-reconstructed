// Dates and times (segment 206a).  Times are DOS-local seconds since
// 1970 as returned by Borland's time().

use crate::brtl::{b_dostounix, b_ltoa, b_time, b_unixtodos, BDate, BTime};
use crate::sre::Sre;
use crate::sstring::Str;
use crate::stream::*;

pub struct TimeState {
    pub now: i32,     // DS:d35c time at start-up
    pub today: i32,   // DS:bbb9 midnight today
    pub timesep: u8,  // DS:bb82 Output.Time.Separator
    pub time24: i16,  // DS:bb83 Output.Time.24Hour
}

impl Default for TimeState {
    fn default() -> Self {
        TimeState { now: 0, today: 0, timesep: b':', time24: 0 }
    }
}

// 206a:0bce (static constructors)
pub fn init_time(g: &mut Sre) {
    g.time.now = b_time();
    g.time.today = midnight(g.time.now);
}

#[allow(dead_code)] // unused, as in the original
pub fn time_second(t: i32) -> i16 {
    b_unixtodos(t).1.ti_sec as i16 // 206a:008c
}
#[allow(dead_code)] // unused, as in the original
pub fn time_minute(t: i32) -> i16 {
    b_unixtodos(t).1.ti_min as i16 // 206a:00a8
}
#[allow(dead_code)] // unused, as in the original
pub fn time_hour(t: i32) -> i16 {
    b_unixtodos(t).1.ti_hour as i16 // 206a:00c2
}
pub fn time_day(t: i32) -> i16 {
    b_unixtodos(t).0.da_day as i16 // 206a:00de
}
pub fn time_month(t: i32) -> i16 {
    b_unixtodos(t).0.da_mon as i16 // 206a:00f9
}
pub fn time_year(t: i32) -> i16 {
    b_unixtodos(t).0.da_year // 206a:0114
}

// 206a:01b1
pub fn midnight(t: i32) -> i32 {
    let (d, mut m) = b_unixtodos(t);
    m.ti_hund = 0;
    m.ti_hour = 0;
    m.ti_min = 0;
    m.ti_sec = 0;
    b_dostounix(&d, &m)
}

// 206a:0251
pub fn make_date(year: i16, month: i16, day: i16) -> i32 {
    let m = BTime::default();
    let d = BDate { da_year: year, da_mon: month as i8, da_day: day as i8 };
    b_dostounix(&d, &m)
}

// Midnight of the latest representable date: "no end date".
pub fn never_date() -> i32 {
    midnight(0x7fffffff)
}

const MONTHS: [&[u8]; 13] = [b"", b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun", b"Jul", b"Aug", b"Sep", b"Oct", b"Nov", b"Dec"];

// 206a:02e7: "8-Mar-1995 12:00pm"; the time part only if not midnight.
pub fn time_str(g: &Sre, t: i32) -> Str {
    let (d, m) = b_unixtodos(t);
    let mut buf = Str::new();
    buf.extend(b_ltoa(d.da_day as i32, 10));
    buf.push(b'-');
    buf.extend_from_slice(MONTHS[d.da_mon as usize]);
    buf.push(b'-');
    buf.extend(b_ltoa(d.da_year as i32, 10));
    if m.ti_hour == 0 && m.ti_min == 0 && m.ti_hund == 0 && m.ti_sec == 0 {
        return buf;
    }
    buf.push(b' ');
    let mut h = m.ti_hour as i16;
    if g.time.time24 == 0 {
        if h >= 12 {
            h -= 12;
        }
        if h == 0 {
            h = 12;
        }
    }
    buf.extend(b_ltoa(h as i32, 10));
    buf.push(g.time.timesep);
    buf.push(m.ti_min / 10 + b'0');
    buf.push(m.ti_min % 10 + b'0');
    if g.time.time24 == 0 {
        buf.push(if m.ti_hour < 12 { b'a' } else { b'p' });
        buf.push(b'm');
    }
    buf
}

// 206a:02bd
pub fn put_time(g: &mut Sre, s: Sid, t: i32) {
    let ts = time_str(g, t);
    put_str(g, s, &ts);
}

// 206a:0498: enter a date as mm/dd/yyyy (or mm/dd/yy); Enter on an empty
// field gives 0.
pub fn input_date(g: &mut Sre) -> i32 {
    const TEMPLATE: &[u8] = b"mm/dd/yyyy"; // DS:bc03
    let i = KIN;
    let mut buf = [0u8; 0xf];
    let mut pos: i16 = 0;
    let echo_s = |g: &mut Sre, s: &[u8]| {
        for &c in s {
            echo(g, i, c);
        }
    };
    let echo_back = |g: &mut Sre| {
        echo(g, i, 8);
        echo(g, i, b' ');
        echo(g, i, 8);
    };
    crate::devices::kin_begin_input(g);
    echo_s(g, TEMPLATE);
    let t: i32;
    loop {
        let c = get_key(g, i);
        let digit = |k: usize| buf[k] as i16 - b'0' as i16;
        if (c == b'\r' || c == b'\n' || c == 0) && (pos == 6 || pos == 8) {
            let mut year: i16;
            if pos == 6 {
                year = digit(4) * 10 + digit(5);
                year += if year > 90 { 1900 } else { 2000 };
            } else {
                year = digit(4) * 1000 + digit(5) * 100 + digit(6) * 10 + digit(7);
            }
            let month = digit(0) * 10 + digit(1);
            let day = digit(2) * 10 + digit(3);
            let d = make_date(year, month, day);
            if time_year(d) == year && time_month(d) == month && time_day(d) == day {
                echo(g, i, b'\n');
                t = d;
                break;
            }
            // Not a date: start over.
            for _ in 0..pos + 2 {
                echo_back(g);
            }
            pos = 0;
            echo_s(g, TEMPLATE);
        }
        if c == 0 {
            echo(g, i, b'\n');
            t = 0;
            break;
        }
        if c == b'/' {
            if pos == 1 {
                buf[1] = buf[0];
                buf[0] = b'0';
                echo(g, i, 8);
                echo(g, i, b'0');
                echo(g, i, buf[1]);
                echo_s(g, b"/dd");
                pos = 2;
                continue;
            }
            if pos == 3 {
                buf[3] = buf[2];
                buf[2] = b'0';
                echo(g, i, 8);
                echo(g, i, b'0');
                echo(g, i, buf[3]);
                echo_s(g, b"/yyyy");
                pos = 4;
                continue;
            }
        }
        if c.is_ascii_digit() && (pos as u16) <= 7 {
            match pos {
                0 => {
                    for _ in 0..TEMPLATE.len() {
                        echo_back(g);
                    }
                    buf[0] = c;
                    echo(g, i, c);
                }
                1 => {
                    buf[1] = c;
                    echo(g, i, c);
                    echo_s(g, b"/dd");
                }
                2 => {
                    echo(g, i, 8);
                    echo(g, i, 8);
                    buf[2] = c;
                    echo(g, i, c);
                }
                3 => {
                    buf[3] = c;
                    echo(g, i, c);
                    echo_s(g, b"/yyyy");
                }
                4 => {
                    echo_s(g, b"\x08\x08  \x08\x08\x08\x08");
                    buf[4] = c;
                    echo(g, i, c);
                }
                _ => {
                    buf[pos as usize] = c;
                    echo(g, i, c);
                }
            }
            pos += 1;
        }
        if c == 8 && pos > 0 {
            let n = if pos == 2 {
                4
            } else if pos == 4 {
                6
            } else {
                1
            };
            for _ in 0..n {
                echo_back(g);
            }
            pos -= 1;
            if pos == 0 {
                echo_s(g, TEMPLATE);
            }
            if pos == 2 {
                echo_s(g, b"dd");
            }
            if pos == 4 {
                echo_s(g, b"yyyy");
            }
        }
    }
    crate::devices::kin_end_input(g);
    t
}
