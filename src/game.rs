// Game data structures.  These are the exact binary layouts of the
// original data files (Borland C++, byte-packed, little-endian), so the
// reconstruction reads and writes the original's files.

use crate::layout::{record, Le};

pub const MAXEMPIRES: usize = 25;
pub const MAXPIRATES: usize = 9;

const MONEY_MAX: i32 = 2000000000;
const MONEY_MIN: i32 = -2000000000;

// 28ec: a money amount, kept within +-2,000,000,000.
#[derive(Clone, Copy, Default)]
pub struct Money {
    pub v: i32,
}

impl Le for Money {
    const SIZE: usize = 4;
    fn rd(b: &[u8]) -> Self {
        Money { v: i32::rd(b) }
    }
    fn wr(&self, b: &mut [u8]) {
        self.v.wr(b)
    }
}

impl Money {
    pub fn get(&self) -> i32 {
        self.v
    }

    // 28ec:003d: above the maximum clamps; below the minimum becomes 0.
    pub fn set(&mut self, x: i32) {
        self.v = if x > MONEY_MAX {
            MONEY_MAX
        } else if x < MONEY_MIN {
            0
        } else {
            x
        };
    }

    // 28ec:00b3: saturating add, done in decreasing power-of-two steps.
    pub fn add(&mut self, mut x: i32) {
        if x < 0 {
            self.sub(x.wrapping_neg());
            return;
        }
        let mut cur = self.v;
        let mut step: i32 = 0x10000000;
        while step > 0 {
            while cur < MONEY_MAX && x >= step && ((cur as u32).wrapping_add(step as u32) as i32) > cur {
                cur = cur.wrapping_add(step);
                x -= step;
            }
            step /= 2;
        }
        self.set(cur);
    }

    // 28ec:0199
    pub fn sub(&mut self, mut x: i32) {
        if x < 0 {
            self.add(x.wrapping_neg());
            return;
        }
        let mut cur = self.v;
        let mut step: i32 = 0x10000000;
        while step > 0 {
            while cur > MONEY_MIN && x >= step && ((cur as u32).wrapping_sub(step as u32) as i32) < cur {
                cur = cur.wrapping_sub(step);
                x -= step;
            }
            step /= 2;
        }
        self.set(cur);
    }
}

// 28ec:029a: would a + b leave the money range?
pub fn money_overflow(a: i32, b: i32) -> bool {
    let s = (a as u32).wrapping_add(b as u32) as i32;
    (a >= 0 && b >= 0 && s < 0)
        || (a >= 0 && b >= 0 && s > MONEY_MAX)
        || (a <= 0 && b <= 0 && s > 0)
        || (a <= 0 && b <= 0 && s < MONEY_MIN)
}

// A diplomatic relation with another empire (4 bytes of bit fields).
#[derive(Clone, Copy, Default)]
pub struct Treaty {
    pub a: u8,   // bits 0-7
    pub b1: u8,  // bits 8-15: type (low 5 bits) ...
    pub w2: u16, // bits 16-31
}

impl Le for Treaty {
    const SIZE: usize = 4;
    fn rd(b: &[u8]) -> Self {
        Treaty { a: b[0], b1: b[1], w2: u16::rd(&b[2..]) }
    }
    fn wr(&self, b: &mut [u8]) {
        b[0] = self.a;
        b[1] = self.b1;
        self.w2.wr(&mut b[2..]);
    }
}

impl Treaty {
    pub fn typ(&self) -> i16 {
        (self.b1 & 0x1f) as i16
    }
    pub fn settype(&mut self, t: i16) {
        self.b1 = (self.b1 & 0xe0) | (t as u8 & 0x1f);
    }
    // bits 13-17
    pub fn c(&self) -> i16 {
        ((((self.b1 as u16) | (self.w2 << 8)) >> 5) & 0x1f) as i16
    }
    pub fn setc(&mut self, v: i16) {
        let mut x: u16 = (self.b1 as u16) | (self.w2 << 8);
        x = (x & 0xfc1f) | (((v & 0x1f) as u16) << 5);
        self.b1 = x as u8;
        self.w2 = (self.w2 & 0xff00) | (x >> 8);
    }
    // bits 18-24
    pub fn d(&self) -> i16 {
        ((self.w2 >> 2) & 0x7f) as i16
    }
    pub fn setd(&mut self, v: i16) {
        self.w2 = (self.w2 & 0xfe03) | (((v & 0x7f) as u16) << 2);
    }
    // bits 25-31
    pub fn e(&self) -> i16 {
        ((self.w2 >> 9) & 0x7f) as i16
    }
    pub fn sete(&mut self, v: i16) {
        self.w2 = (self.w2 & 0x01ff) | (((v & 0x7f) as u16) << 9);
    }
}

record! {
    // Planets of one type.
    #[derive(Copy)]
    pub struct PlanetSet {
        pub count: i32, // +0
        pub a: i32,     // +4
        pub b: i32,     // +8
    }
}

pub const P_PETROLEUM: usize = 0;
pub const P_EDUCATION: usize = 1;
pub const P_TOURISM: usize = 2;
pub const P_FOOD: usize = 3;
pub const P_RESEARCH: usize = 4;
pub const P_ORE: usize = 5;
pub const P_GOVERNMENT: usize = 6;
pub const P_URBAN: usize = 7;
pub const P_SUPPLY: usize = 8;
pub const P_ANTIPOLLUTION: usize = 9;

record! {
    pub struct Empire {
        pub id: i16,                    // 000
        pub name: [u8; 0x20],           // 002
        pub player: i16,                // 022
        pub created: i32,               // 024
        pub lastplay: i32,              // 028
        pub planets: [PlanetSet; 10],   // 02c
        pub u_0a4: [u8; 0x48],          // 0a4
        pub treaty: [Treaty; 26],       // 0ec
        pub soldiers: i32,              // 154
        pub fighters: i32,              // 158
        pub stations: i32,              // 15c defense stations
        pub heavycruisers: i32,         // 160
        pub carriers: i32,              // 164
        pub generals: i32,              // 168
        pub d16c: i32,                  // 16c
        pub effectiveness: i8,          // 170
        pub cmdship: i8,                // 171 command ship % completed
        pub w172: i16,                  // 172
        pub score: i32,                 // 174
        pub d178: i32,                  // 178
        pub money: Money,               // 17c
        pub m180: Money,                // 180
        pub population: i32,            // 184 millions
        pub d188: i32,                  // 188
        pub food: i32,                  // 18c megatons
        pub insurgency: i16,            // 190 0..7
        pub w192: i16,                  // 192
        pub year: i16,                  // 194 < 0: turns of protection left
        pub turnsused: i16,             // 196 today
        pub turnsperday: i16,           // 198
        pub w19a: i16,                  // 19a
        pub bank: Money,                // 19c savings (< 0: loan)
        pub m1a0: Money,                // 1a0
        pub d1a4: i32,                  // 1a4
        pub w1a8: i16,                  // 1a8
        pub taxrate: i8,                // 1aa
        pub w1ab: i16,                  // 1ab
        pub draftrate: i8,              // 1ad
        pub w1ae: i16,                  // 1ae
        pub d1b0: i32,                  // 1b0
        pub d1b4: i32,                  // 1b4
        pub flags: i16,                 // 1b8
        pub phase: i16,                 // 1ba turn phase
        pub m1bc: Money,                // 1bc
        pub m1c0: Money,                // 1c0
        pub b1c4: u8,                   // 1c4
        pub b1c5: i8,                   // 1c5
        pub b1c6: i8,                   // 1c6
        pub u_1c7: [u8; 0x60],          // 1c7
        pub weapons: u16,               // 227 bits: 1 nuclear, 2 chemical, 4 psionic, 8 mothership
        pub w229: i16,                  // 229
        pub agents: i32,                // 22b covert agents
        pub d22f: [i32; 7],             // 22f
        pub u_24b: [u8; 0x24],          // 24b
        pub w26f: [i16; 9],             // 26f
        pub u_281: [u8; 0x12],          // 281
        pub d293: i32,                  // 293
        pub b297: [i8; 6],              // 297
        pub d29d: i32,                  // 29d
        pub w2a1: i16,                  // 2a1
        pub bondname: [u8; 0x20],       // 2a3 the empire's own bonds
        pub bonds: i16,                 // 2c3 bonds outstanding
        pub maxbonds: i16,              // 2c5
        pub bondprice: i32,             // 2c7
        pub bondpayoff: i32,            // 2cb
        pub bondhours: i16,             // 2cf
        pub bondturns: i16,             // 2d1
        pub b2d3: [i8; 25],             // 2d3
        pub d2ec: i32,                  // 2ec
        pub lightcruisers: i32,         // 2f0
        pub troubles: i32,              // 2f4
        pub bonuses: i32,               // 2f8
    }
}

record! {
    // 14f6:000f: integrity check of a record (also its encryption key).
    #[derive(Copy)]
    pub struct Checksum {
        pub sum: i32,       // 7420:0130 (16-bit sum, zero-extended)
        pub crc: i32,       // 7420:009a CRC-32
        pub salt: [u8; 8],  // initial key, random
        pub key: [u8; 8],   // final key after encryption
    }
}

impl Checksum {
    pub fn clear(&mut self) {
        *self = Checksum::default();
    }
}

record! {
    // Galaxy.II (DS:cd6a, 0x3c6 bytes).
    pub struct Galaxy {
        pub name: [u8; 0x40],           // 000 (name[0] == 0: no game)
        pub created: i32,               // 040
        pub t44: i32,                   // 044
        pub t48: i32,                   // 048
        pub t4c: i32,                   // 04c
        pub enddate: i32,               // 050
        pub empire: [Checksum; 25],     // 054
        pub pirate: [Checksum; 9],      // 2ac
        pub w384: i16,                  // 384
        pub w386: i16,                  // 386 OR-ed into every empire's flags
        pub d388: i32,                  // 388
        pub w38c: i16,                  // 38c
        pub d38e: i32,                  // 38e
        pub d392: i32,                  // 392
        pub solarmaster: i16,           // 396
        pub planetmaster: i16,          // 398
        pub weaponsmaster: i16,         // 39a
        pub grantdays: i16,             // 39c turns are granted every n days
        pub grantturns: i16,            // 39e turns granted
        pub protection: i16,            // 3a0 turns of protection
        pub w3a2: i16,                  // 3a2
        pub w3a4: i16,                  // 3a4
        pub w3a6: i16,                  // 3a6
        pub d3a8: i32,                  // 3a8
        pub d3ac: i32,                  // 3ac
        pub w3b0: i16,                  // 3b0
        pub w3b2: i16,                  // 3b2
        pub w3b4: i16,                  // 3b4
        pub d3b6: i32,                  // 3b6
        pub t3ba: i32,                  // 3ba
        pub interest: i16,              // 3be
        pub w3c0: i16,                  // 3c0
        pub d3c2: i32,                  // 3c2
    }
}

record! {
    // System.II (DS:d196, 0x142 bytes, not encrypted).
    pub struct System {
        pub sysop: [u8; 0x40],          // 000
        pub bbs: [u8; 0x40],            // 040
        pub regto: [u8; 0x50],          // 080 registered to
        pub bank: [u8; 0x50],           // 0d0 name of the bank
        pub t120: i32,                  // 120
        pub t124: i32,                  // 124
        pub regstate: i16,              // 128 0..5
        pub galaxy: Checksum,           // 12a
    }
}

record! {
    // Player.II record (0x72 bytes).
    pub struct Player {
        pub index: i16,                 // 00
        pub user: [u8; 0x20],           // 02 BBS user name
        pub name: [u8; 0x20],           // 22 name in the game
        pub b42: [u8; 0x20],            // 42
        pub created: i32,               // 62
        pub lastplay: i32,              // 66
        pub male: i16,                  // 6a
        pub prefs1: u8,                 // 6c
        pub prefs2: u8,                 // 6d
        pub w6e: i16,                   // 6e
        pub w70: i16,                   // 70
    }
}

record! {
    // Pirate.II record (0x106 bytes).
    pub struct Pirate {
        pub name: [u8; 0x20],           // 00
        pub level: i16,                 // 20 1..9
        pub lasthour: i32,              // 22
        pub u_026: [u8; 0x69],          // 26
        pub color: u8,                  // 8f
        pub money: i32,                 // 90
        pub d94: i32,                   // 94
        pub d98: i32,                   // 98
        pub d9c: i32,                   // 9c
        pub da0: i32,                   // a0
        pub da4: i32,                   // a4
        pub da8: i32,                   // a8
        pub dac: i32,                   // ac
        pub db0: i32,                   // b0
        pub wb4: [i16; 16],             // b4
        pub wd4: [i16; 25],             // d4
    }
}

record! {
    // DOORFILE.SR (7389, DS:d360, 0x30 bytes).
    pub struct DoorInfo {
        pub name: [u8; 0x20],           // 00
        pub ansi: i16,                  // 20
        pub ibm: i16,                   // 22
        pub pagelen: i16,               // 24
        pub port: i16,                  // 26 0 = local
        pub baud: u16,                  // 28
        pub lockbaud: u16,              // 2a
        pub minutes: i16,               // 2c time limit, -1 = none
        pub loaded: i16,                // 2e
    }
}

const _: () = assert!(<Empire as Le>::SIZE == 0x2fc);
const _: () = assert!(<Checksum as Le>::SIZE == 0x18);
const _: () = assert!(<Galaxy as Le>::SIZE == 0x3c6);
const _: () = assert!(<System as Le>::SIZE == 0x142);
const _: () = assert!(<Player as Le>::SIZE == 0x72);
const _: () = assert!(<DoorInfo as Le>::SIZE == 0x30);
const _: () = assert!(<Pirate as Le>::SIZE == 0x106);

impl Empire {
    pub fn exists(&self) -> bool {
        self.name[0] != 0
    }
}
