// Borland C++ 3.1 runtime behaviour the game depends on.

use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

// 1000:459d stricmp
pub fn b_stricmp(a: &[u8], b: &[u8]) -> i16 {
    let at = |s: &[u8], i: usize| if i < s.len() { s[i] } else { 0 };
    let mut i = 0;
    let mut j = 0;
    loop {
        let mut ca = at(a, i);
        i += 1;
        let mut cb = at(b, j);
        if ca == 0 {
            return ca as i16 - cb as i16;
        }
        j += 1;
        if ca == cb {
            continue;
        }
        if ca.is_ascii_lowercase() {
            ca -= 0x20;
        }
        if cb.is_ascii_lowercase() {
            cb -= 0x20;
        }
        if ca != cb {
            return ca as i16 - cb as i16;
        }
    }
}

// 1000:462e strnicmp
pub fn b_strnicmp(a: &[u8], b: &[u8], mut n: i16) -> i16 {
    let at = |s: &[u8], i: usize| if i < s.len() { s[i] } else { 0 };
    let (mut ca, mut cb) = (0u8, 0u8);
    let mut i = 0;
    let mut j = 0;
    while n != 0 {
        ca = at(a, i);
        i += 1;
        cb = at(b, j);
        if ca == 0 {
            break;
        }
        j += 1;
        n -= 1;
        if ca == cb {
            continue;
        }
        if ca.is_ascii_lowercase() {
            ca -= 0x20;
        }
        if cb.is_ascii_lowercase() {
            cb -= 0x20;
        }
        if ca != cb {
            break;
        }
        if n == 0 {
            return 0;
        }
    }
    if n == 0 && ca == cb {
        return 0;
    }
    ca as i16 - cb as i16
}

// 1000:4788 strupr
pub fn b_strupr(s: &mut [u8]) {
    for c in s.iter_mut() {
        if *c == 0 {
            break;
        }
        if c.is_ascii_lowercase() {
            *c -= 0x20;
        }
    }
}

pub fn b_ltoa(v: i32, radix: i16) -> Vec<u8> {
    let mut out = Vec::new();
    let mut u: u32;
    if radix == 10 && v < 0 {
        out.push(b'-');
        u = (-(v as i64)) as u32;
    } else {
        u = v as u32;
    }
    let mut tmp = Vec::new();
    loop {
        let d = (u % radix as u32) as u8;
        tmp.push(if d < 10 { b'0' + d } else { b'a' + d - 10 });
        u /= radix as u32;
        if u == 0 {
            break;
        }
    }
    while let Some(c) = tmp.pop() {
        out.push(c);
    }
    out
}

pub fn b_atol(s: &[u8]) -> i32 {
    let at = |i: usize| if i < s.len() { s[i] } else { 0 };
    let mut i = 0;
    while matches!(at(i), b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c) {
        i += 1;
    }
    let mut neg = false;
    if at(i) == b'-' {
        neg = true;
        i += 1;
    } else if at(i) == b'+' {
        i += 1;
    }
    let mut v: u32 = 0;
    while at(i).is_ascii_digit() {
        v = v.wrapping_mul(10).wrapping_add((at(i) - b'0') as u32);
        i += 1;
    }
    if neg {
        (v as i32).wrapping_neg()
    } else {
        v as i32
    }
}

// ---------------------------------------------------------------------
// rand()/srand() (1000:0b9d, 1000:0b8c).

static SEED: AtomicU32 = AtomicU32::new(1);

pub fn b_rand() -> i16 {
    let s = SEED.load(Ordering::Relaxed).wrapping_mul(0x015A4E35).wrapping_add(1);
    SEED.store(s, Ordering::Relaxed);
    ((s >> 16) & 0x7fff) as i16
}

pub fn b_srand(seed: u16) {
    SEED.store(seed as u32, Ordering::Relaxed);
}

// Borland's random(n), as the game computes it inline.
pub fn random(n: i16) -> i16 {
    ((b_rand() as i32 * n as i32) / 0x8000) as i16
}

// ---------------------------------------------------------------------
// DOS date/time structures and conversions (1000:47ac dostounix,
// 1000:48d6 unixtodos, 1000:0f03 time) with Borland's default
// TZ = EST5EDT and its US daylight rule: timezone = 5 hours (EST),
// daylight = 1, as Borland's tzset() sets them when TZ is not defined.

#[derive(Clone, Copy, Default)]
pub struct BDate {
    pub da_year: i16,
    pub da_day: i8,
    pub da_mon: i8,
}

#[derive(Clone, Copy, Default)]
pub struct BTime {
    pub ti_min: u8,
    pub ti_hour: u8,
    pub ti_hund: u8,
    pub ti_sec: u8,
}

const TIMEZONE: i32 = 18000;
const DAYLIGHT: bool = true;
const DAYS: [i8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
// DS:c72c; index 0 is the word before the cumulative table.
const YDAYS_TAB: [u16; 14] = [7966, 0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];

fn ydays(i: i32) -> u16 {
    YDAYS_TAB[(i + 1) as usize]
}

// 1000:4cc8 __isDST(year - 1970, month (0 = use yday), yday, hour)
fn is_dst(year: u16, mut month: u16, mut yday: u16, hour: u8) -> bool {
    let mut cx: u16;
    if month == 0 {
        let mut si = yday;
        if yday >= 59 && (year.wrapping_add(70) & 3) == 0 {
            si = si.wrapping_sub(1);
        }
        month = 0;
        while ydays(month as i32) <= si {
            month += 1;
            // A day past the end of the table (a time before 1980) reads
            // on past it; whatever month that gives is past October.
            if month > 12 {
                return false;
            }
        }
    } else {
        yday = yday.wrapping_add(ydays(month as i32 - 1));
        if month > 3 && (year.wrapping_add(70) & 3) == 0 {
            yday = yday.wrapping_add(1);
        }
    }
    if month < 4 {
        return false;
    }
    if month > 10 {
        return false;
    }
    if month != 4 && month != 10 {
        return true;
    }
    if (year as i16) > 16 && month == 4 {
        cx = ydays(month as i32 - 1).wrapping_add(7);
    } else {
        cx = ydays(month as i32);
    }
    if (year.wrapping_add(0x7b2) & 3) != 0 {
        cx = cx.wrapping_sub(1);
    }
    let bx = ((year.wrapping_add(1) as i16 >> 2) as u16).wrapping_add(cx);
    let ax = 0x16du16.wrapping_mul(year).wrapping_add(bx).wrapping_add(4);
    cx = cx.wrapping_sub(ax % 7);
    if month == 4 {
        if yday > cx {
            return true;
        }
        if yday != cx {
            return false;
        }
        return hour >= 2;
    }
    if yday < cx {
        return true;
    }
    if yday != cx {
        return false;
    }
    hour <= 1
}

pub fn b_dostounix(d: &BDate, t: &BTime) -> i32 {
    let mut x: i32 = TIMEZONE + 315532800;
    let si: i16 = d.da_year.wrapping_sub(1980);
    x = x.wrapping_add(((si >> 2) as i32).wrapping_mul(126230400));
    x = x.wrapping_add(((si & 3) as i32) * 31536000);
    if si & 3 != 0 {
        x = x.wrapping_add(86400);
    }
    let mut days: i16 = 0;
    let mut i: i16 = d.da_mon as i16 - 1;
    while i > 0 {
        i -= 1;
        // A month past December (a mistyped date) reads on past the
        // table, into data that depends on where DOS loaded the program;
        // the caller rejects such a date whatever it comes to.
        days = days.wrapping_add(DAYS.get(i as usize).map_or(0, |&n| n as i16));
    }
    days = days.wrapping_add(d.da_day as i16 - 1);
    if d.da_mon > 2 && (d.da_year & 3) == 0 {
        days = days.wrapping_add(1);
    }
    let mut hours: i16 = ((days as i32) * 24 + t.ti_hour as i32) as i16;
    if DAYLIGHT && is_dst(d.da_year.wrapping_sub(1970) as u16, 0, days as u16, t.ti_hour) {
        hours = hours.wrapping_sub(1);
    }
    x = x.wrapping_add((hours as i32).wrapping_mul(3600));
    x = x.wrapping_add(t.ti_min as i32 * 60 + t.ti_sec as i32);
    x
}

pub fn b_unixtodos(mut time: i32) -> (BDate, BTime) {
    let mut d = BDate::default();
    let mut t = BTime::default();
    time = time.wrapping_sub(TIMEZONE + 315532800);
    t.ti_hund = 0;
    t.ti_sec = (time % 60) as u8;
    time /= 60;
    t.ti_min = (time % 60) as u8;
    time /= 60;
    d.da_year = (1980 + 4 * (time / 35064) as i16 as i32) as i16;
    time %= 35064;
    if time >= 8784 {
        time -= 8784;
        d.da_year = d.da_year.wrapping_add(1);
        d.da_year = d.da_year.wrapping_add((time / 8760) as i16);
        time %= 8760;
    }
    if DAYLIGHT && is_dst(d.da_year.wrapping_sub(1970) as u16, 0, (time / 24) as u16, (time % 24) as u8) {
        time += 1;
    }
    t.ti_hour = (time % 24) as u8;
    time /= 24;
    time += 1;
    if (d.da_year & 3) == 0 {
        if time > 60 {
            time -= 1;
        } else if time == 60 {
            d.da_mon = 2;
            d.da_day = 29;
            return (d, t);
        }
    }
    d.da_mon = 0;
    while (DAYS[d.da_mon as usize] as i32) < time {
        time -= DAYS[d.da_mon as usize] as i32;
        d.da_mon += 1;
    }
    d.da_mon += 1;
    d.da_day = time as i8;
    (d, t)
}

// ---------------------------------------------------------------------
// Clock source: the host's local time, or a fixed one for tests.

static FIXED: AtomicBool = AtomicBool::new(false);
static FIXED_BASE: AtomicI32 = AtomicI32::new(0); // DOS local wall clock, as seconds since 1970 (UTC fields)
static RATE: AtomicI32 = AtomicI32::new(0); // 0 = frozen; 1 = real time
static START: OnceLock<Instant> = OnceLock::new();

// Milliseconds elapsed (for delay() and timeouts).
pub fn b_millis() -> u32 {
    START.get_or_init(Instant::now).elapsed().as_millis() as u32
}

pub fn b_setclock(base: i32, rate: i32) {
    FIXED.store(true, Ordering::Relaxed);
    FIXED_BASE.store(base, Ordering::Relaxed);
    RATE.store(rate, Ordering::Relaxed);
    b_millis();
}

fn date_shift() -> i32 {
    static SHIFT: OnceLock<i32> = OnceLock::new();
    *SHIFT.get_or_init(|| match std::env::var("SRE_DATE_SHIFT") {
        Ok(s) if !s.is_empty() => b_atol(s.as_bytes()),
        _ => 28,
    })
}

fn wallclock() -> libc::tm {
    // SAFETY: tm is plain data; gmtime_r/localtime_r fill it in.
    let mut out: libc::tm = unsafe { std::mem::zeroed() };
    if FIXED.load(Ordering::Relaxed) {
        let mut t = FIXED_BASE.load(Ordering::Relaxed) as libc::time_t;
        if RATE.load(Ordering::Relaxed) != 0 {
            t += (b_millis() / 1000) as libc::time_t;
        }
        unsafe { libc::gmtime_r(&t, &mut out) };
    } else {
        // The original only accepts dates from 1980 to November 1999
        // (6c38:0261), so the host's date is moved back by
        // SRE_DATE_SHIFT years: 28 by default, which keeps the calendar
        // (weekdays and leap years) the same and works through 2027.
        let t = unsafe { libc::time(std::ptr::null_mut()) };
        unsafe { libc::localtime_r(&t, &mut out) };
        out.tm_year -= date_shift();
    }
    out
}

pub fn b_getdate() -> BDate {
    let tm = wallclock();
    BDate { da_year: (tm.tm_year + 1900) as i16, da_mon: (tm.tm_mon + 1) as i8, da_day: tm.tm_mday as i8 }
}

pub fn b_gettime() -> BTime {
    let tm = wallclock();
    BTime { ti_hour: tm.tm_hour as u8, ti_min: tm.tm_min as u8, ti_sec: tm.tm_sec as u8, ti_hund: 0 }
}

// 1000:0f03
pub fn b_time() -> i32 {
    let d = b_getdate();
    let t = b_gettime();
    b_dostounix(&d, &t)
}

pub fn b_sleep(ms: u32) {
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
}

pub fn b_delay(ms: u16) {
    if FIXED.load(Ordering::Relaxed) && RATE.load(Ordering::Relaxed) == 0 {
        return;
    }
    b_sleep(ms as u32);
}
