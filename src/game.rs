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

// Treaty types (Treaty::typ()).
pub const TT_NONE: i16 = 0;
pub const TT_NEUTRALITY: i16 = 1;
pub const TT_FREETRADE: i16 = 2;
pub const TT_MINORALLIANCE: i16 = 3;
pub const TT_ARMEDDEFENSE: i16 = 4;
pub const TT_CRUISERPROTECTION: i16 = 5;
pub const TT_TOTALDEFENSE: i16 = 6;
pub const TT_CUSTOM: i16 = 7;

// The status of a proposal (or of a break) sent by the other empire.
pub const TS_NONE: i16 = 0;
pub const TS_PROPOSED: i16 = 1;
pub const TS_REJECTED: i16 = 2;
pub const TS_ACCEPTED: i16 = 3;
pub const TS_BROKEN: i16 = 4;

// A diplomatic relation with another empire (4 bytes of bit fields).
#[derive(Clone, Copy, Default)]
pub struct Treaty {
    pub days: u8, // bits 0-7: days left, 0 = no limit
    pub lo: u8,   // bits 8-15: type (TT_*, low 5 bits) ...
    pub hi: u16,  // bits 16-31
}

impl Le for Treaty {
    const SIZE: usize = 4;
    fn rd(b: &[u8]) -> Self {
        Treaty { days: b[0], lo: b[1], hi: u16::rd(&b[2..]) }
    }
    fn wr(&self, b: &mut [u8]) {
        b[0] = self.days;
        b[1] = self.lo;
        self.hi.wr(&mut b[2..]);
    }
}

impl Treaty {
    pub fn typ(&self) -> i16 {
        (self.lo & 0x1f) as i16
    }
    pub fn settype(&mut self, t: i16) {
        self.lo = (self.lo & 0xe0) | (t as u8 & 0x1f);
    }
    // bits 13-17: TS_*
    pub fn status(&self) -> i16 {
        ((((self.lo as u16) | (self.hi << 8)) >> 5) & 0x1f) as i16
    }
    pub fn setstatus(&mut self, v: i16) {
        let mut x: u16 = (self.lo as u16) | (self.hi << 8);
        x = (x & 0xfc1f) | (((v & 0x1f) as u16) << 5);
        self.lo = x as u8;
        self.hi = (self.hi & 0xff00) | (x >> 8);
    }
    // bits 18-24: % of the ally's cruisers that defend
    pub fn cruiserpct(&self) -> i16 {
        ((self.hi >> 2) & 0x7f) as i16
    }
    pub fn setcruiserpct(&mut self, v: i16) {
        self.hi = (self.hi & 0xfe03) | (((v & 0x7f) as u16) << 2);
    }
    // bits 25-31: % of the ally's soldiers that defend
    pub fn soldierpct(&self) -> i16 {
        ((self.hi >> 9) & 0x7f) as i16
    }
    pub fn setsoldierpct(&mut self, v: i16) {
        self.hi = (self.hi & 0x01ff) | (((v & 0x7f) as u16) << 9);
    }
}

record! {
    // Planets of one type.
    #[derive(Copy)]
    pub struct PlanetSet {
        pub count: i32,  // +0
        pub target: i32, // +4 the production per planet moves toward this
        pub output: i32, // +8 production per planet
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
        pub tradegoods: i32,            // 16c value of the goods out with trading fleets
        pub effectiveness: i8,          // 170
        pub cmdship: i8,                // 171 command ship % completed
        pub tickets: i16,               // 172 lottery tickets bought today
        pub score: i32,                 // 174
        pub points: i32,                // 178 tournament points (Solar/Planet/Weapons Master)
        pub money: Money,               // 17c
        pub m180: Money,                // 180 (unused)
        pub population: i32,            // 184 millions
        pub pollution: i32,             // 188 exatons
        pub food: i32,                  // 18c megatons
        pub insurgency: i16,            // 190 0..7
        pub planetsbought: i16,         // 192 this turn, less those released
        pub year: i16,                  // 194 < 0: turns of protection left
        pub turnsused: i16,             // 196 today
        pub turnsperday: i16,           // 198
        pub bonusyear: i16,             // 19a year of the next protection bonus
        pub bank: Money,                // 19c savings (< 0: loan)
        pub m1a0: Money,                // 1a0 (unused)
        pub loanpayment: i32,           // 1a4 owed on the loan each turn
        pub w1a8: i16,                  // 1a8 (unused)
        pub taxrate: i8,                // 1aa
        pub w1ab: i16,                  // 1ab (unused)
        pub draftrate: i8,              // 1ad
        pub w1ae: i16,                  // 1ae (unused)
        pub favor: i32,                 // 1b0 of the Galactic Coordinator
        pub efficiency: i32,            // 1b4 profit per planet
        pub localplay: i16,             // 1b8 unlimited turns (Galaxy::localonly)
        pub phase: i16,                 // 1ba of the turn in progress, PH_*
        pub income: Money,              // 1bc last turn's, in thousands
        pub m1c0: Money,                // 1c0 (unused)
        pub shortfalls: u8,             // 1c4 SF_* this turn
        pub effchange: i8,              // 1c5 change of effectiveness from the army's upkeep
        pub effnext: i8,                // 1c6 effectiveness after it; < 0: desertion
        // 1c7: what went short this turn (0x60 bytes, cleared each turn)
        pub neglected: [i32; 10],       // 1c7 planets of each type left unmaintained
        pub u_1ef: [u8; 0x34],          // 1ef
        pub starving: i32,              // 223 people not fed
        pub weapons: u16,               // 227 W_*
        pub attacks: i16,               // 229 made today
        pub agents: i32,                // 22b covert agents
        // 22f: the empires each covert operation was done to (masks); see op_mask
        pub aidmask: i32,               // 22f insurgent aid, this turn
        pub dissensionmask: i32,        // 233 this turn
        pub demoralizemask: i32,        // 237 this turn
        pub bombmask: i32,              // 23b today
        pub hostagemask: i32,           // 23f today
        pub bribemask: i32,             // 243 empires an agent of ours is bribed in
        pub setupmask: i32,             // 247 today
        pub u_24b: [u8; 0x20],          // 24b
        pub othermask: i32,             // 26b never used
        pub w26f: [i16; 9],             // 26f (cleared, never used)
        pub u_281: [u8; 0x12],          // 281
        pub researchpoints: i32,        // 293
        pub supplyrates: [i8; 6],       // 297 % of supply planets making soldiers, fighters,
                                        //     stations, cruisers, carriers, generals
        pub d29d: i32,                  // 29d (unused)
        pub inactivedays: i16,          // 2a1 days left before an inactive empire is deleted
        pub bondname: [u8; 0x20],       // 2a3 the empire's own bonds
        pub bonds: i16,                 // 2c3 bonds outstanding
        pub maxbonds: i16,              // 2c5
        pub bondprice: i32,             // 2c7
        pub bondpayoff: i32,            // 2cb
        pub bondhours: i16,             // 2cf
        pub bondturns: i16,             // 2d1
        pub intercepts: [i8; 25],       // 2d3 our messages each empire's spies will intercept
        pub lastnews: i32,              // 2ec when the news was last read
        pub lightcruisers: i32,         // 2f0
        pub troubles: i32,              // 2f4 TR_* (shown, but never set)
        pub bonuses: i32,               // 2f8 B_* (shown, but never set)
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
        pub startdate: i32,             // 044 play starts
        pub lastregen: i32,             // 048 planets last came back onto the market
        pub joindeadline: i32,          // 04c no joining from this date
        pub enddate: i32,               // 050
        pub empire: [Checksum; 25],     // 054
        pub pirate: [Checksum; 9],      // 2ac
        pub inflation: i16,             // 384 prices grow with the empire
        pub localonly: i16,             // 386 OR-ed into every empire's localplay
        pub marketfood: i32,            // 388 megatons for sale
        pub regenseconds: i16,          // 38c a planet comes back every n seconds
        pub planetsperturn: i32,        // 38e that can be bought, -1 = no limit
        pub marketplanets: i32,         // 392 for sale
        pub solarmaster: i16,           // 396
        pub planetmaster: i16,          // 398
        pub weaponsmaster: i16,         // 39a
        pub daylength: i16,             // 39c a game "day" (maintenance interval) in days
        pub grantturns: i16,            // 39e turns granted
        pub protection: i16,            // 3a0 turns of protection
        pub w3a2: i16,                  // 3a2 (unused)
        pub lotterytickets: i16,        // 3a4 sold today
        pub lotterywinner: i16,         // 3a6 empire, -1 = none
        pub jackpot: i32,               // 3a8 last won
        pub coordfund: i32,             // 3ac the Galactic Coordinator's, in thousands
        pub coordtax: i16,              // 3b0 per 1000 credits of income
        pub nukelimit: i16,             // 3b2 most planets a nuclear bomb destroys
        pub nukepercent: i16,           // 3b4 of the planets a nuclear bomb destroys
        pub oilprice: i32,              // 3b6
        pub lasthourly: i32,            // 3ba hourly maintenance last ran
        pub interest: i16,              // 3be
        pub newspaper: i16,             // 3c0 that prints the news, 0..3
        pub psionicprice: i32,          // 3c2 the smuggler's price of a psionic bomb
    }
}

record! {
    // System.II (DS:d196, 0x142 bytes, not encrypted).
    pub struct System {
        pub sysop: [u8; 0x40],          // 000
        pub bbs: [u8; 0x40],            // 040
        pub regto: [u8; 0x50],          // 080 registered to
        pub bank: [u8; 0x50],           // 0d0 name of the bank
        pub created: i32,               // 120
        pub inuse: i32,                 // 124 since when the game is in use, 0 = not
        pub regstate: i16,              // 128 RS_* (0..5 valid)
        pub galaxy: Checksum,           // 12a
    }
}

record! {
    // Player.II record (0x72 bytes).
    pub struct Player {
        pub index: i16,                 // 00
        pub user: [u8; 0x20],           // 02 BBS user name
        pub name: [u8; 0x20],           // 22 name in the game
        pub b42: [u8; 0x20],            // 42 (cleared, never used)
        pub created: i32,               // 62
        pub lastplay: i32,              // 66
        pub male: i16,                  // 6a
        pub prefs1: u8,                 // 6c PF1_*
        pub prefs2: u8,                 // 6d PF2_*
        pub w6e: i16,                   // 6e (cleared, never used)
        pub seenplanets: i16,           // 70 bits: planet types whose screen was shown
    }
}

record! {
    // Pirate.II record (0x106 bytes).
    pub struct Pirate {
        pub name: [u8; 0x20],           // 00
        pub level: i16,                 // 20 1..9
        pub lasthour: i32,              // 22
        pub u_026: [u8; 0x69],          // 26 (cleared, never used)
        pub color: u8,                  // 8f
        pub money: i32,                 // 90
        pub food: i32,                  // 94
        pub soldiers: i32,              // 98
        pub fighters: i32,              // 9c
        pub stations: i32,              // a0
        pub heavycruisers: i32,         // a4
        pub carriers: i32,              // a8
        pub generals: i32,              // ac
        pub agents: i32,                // b0
        pub planets: [i16; 16],         // b4 by type (P_*)
        pub hatred: [i16; 25],          // d4 of each empire, from its raids
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

// Empire::phase: the steps of a turn, in order (see phase in play.rs).
pub const PH_START: i16 = 0;
pub const PH_INTEREST: i16 = 1;
pub const PH_REPORTS: i16 = 2;
pub const PH_TURNREPORT: i16 = 3;
pub const PH_ADVISOR: i16 = 4;
pub const PH_STATUS: i16 = 5;
pub const PH_UPKEEP: i16 = 6;
pub const PH_FOOD: i16 = 7;
pub const PH_TURNOPS: i16 = 8;
pub const PH_COVERT: i16 = 9;
pub const PH_BANK: i16 = 10;
pub const PH_SPENDING: i16 = 11;
pub const PH_DOUBLEAGENTS: i16 = 12;
pub const PH_UNUSED13: i16 = 13;
pub const PH_ACTION: i16 = 14;
pub const PH_UNUSED15: i16 = 15;
pub const PH_ENDTURN: i16 = 16;
pub const PH_POPULATION: i16 = 17;
pub const PH_SCORE: i16 = 18;
pub const PH_CIVILWAR: i16 = 19;
pub const PH_PLANETS: i16 = 20;
pub const PH_CONTINUE: i16 = 21;

// Empire::weapons
pub const W_NUCLEAR: u16 = 1;
pub const W_CHEMICAL: u16 = 2;
pub const W_PSIONIC: u16 = 4;
pub const W_MOTHERSHIP: u16 = 8;

// Empire::troubles
pub const TR_PLAGUE: i32 = 0x01;
pub const TR_FAMINE: i32 = 0x02;
pub const TR_SPACESTORMS: i32 = 0x04;
pub const TR_DUSTCLOUD: i32 = 0x08;
pub const TR_GASCLOUD: i32 = 0x10;
pub const TR_BANKFAILURE: i32 = 0x20;

// Empire::bonuses
pub const B_CLOAKING: i32 = 0x01;
pub const B_SCANNER: i32 = 0x02;
pub const B_GENEFOOD: i32 = 0x04;
pub const B_SHIELD: i32 = 0x08;
pub const B_ACCURA: i32 = 0x10;
pub const B_SPYING: i32 = 0x20;

// Empire::shortfalls (SF_UNUSED is tested but never set.)
pub const SF_NEGLECT: u8 = 1;
pub const SF_STARVED: u8 = 2;
pub const SF_UNUSED: u8 = 4;
pub const SF_ARMY: u8 = 8;

// System::regstate
pub const RS_UNREGISTERED: i16 = 0;
pub const RS_REGISTERED: i16 = 2;
pub const RS_BETA: i16 = 4;

// Player::prefs1 (bits 0 and 1 are never used)
pub const PF1_SHORTMENUS: u8 = 0x04; // don't redisplay the spending and planet menus
pub const PF1_TERSE: u8 = 0x08; // shorter prompts (never set by the preferences menu)
pub const PF1_ASKDEPOSIT: u8 = 0x10; // ask about depositing at the end of a turn
pub const PF1_DEPOSIT: u8 = 0x20; // ... and deposit by default
pub const PF1_ASKDONATE: u8 = 0x40; // ask about donating at the end of a turn
pub const PF1_DONATE: u8 = 0x80; // ... and donate by default
// Player::prefs2
pub const PF2_ASKPAY: u8 = 0x01; // ask about paying the upkeep
pub const PF2_AUTOPAY: u8 = 0x02; // ... and pay by default
pub const PF2_BATTLEMSG: u8 = 0x04; // attach a message to a won battle's report
pub const PF2_REPORTDATES: u8 = 0x08; // show the dates of reports
pub const PF2_ENTERSTAYS: u8 = 0x10; // Enter doesn't leave the spending menu

pub fn has_pref(prefs: u8, f: u8) -> bool {
    prefs & f != 0
}
pub fn set_pref(prefs: &mut u8, f: u8, on: bool) {
    *prefs = (*prefs & !f) | if on { f } else { 0 };
}

impl Empire {
    pub fn exists(&self) -> bool {
        self.name[0] != 0
    }
    pub fn has_weapon(&self, w: u16) -> bool {
        self.weapons & w != 0
    }
}
