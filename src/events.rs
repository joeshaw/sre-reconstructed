// Scheduled events: covert operations and bank investments (4c48).

use crate::battle::*;
use crate::player::random_pronoun;
use crate::prelude::*;
use std::any::Any;

pub const EVENT_FILE: &[u8] = b"Data/Event"; // DS:303a
pub const COVERT_FILE: &[u8] = b"Data/CovertOp"; // DS:308e

pub const CLS_DISSENSION: &[u8] = b"EventCovertDissension"; // DS:303e
pub const CLS_DEMORALIZE: &[u8] = b"EventCovertDemoralize"; // DS:304e
pub const CLS_HOSTAGES: &[u8] = b"EventCovertHostages"; // DS:305e
pub const CLS_SETUP: &[u8] = b"EventCovertSetup"; // DS:306e
pub const CLS_BANK: &[u8] = b"EventBank"; // DS:307e

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EvKind {
    Dissension, // vtable 37fa ...
    Demoralize,
    Hostages,
    Setup,
    Bank, // vtable 375e
}

// An event (base vtable 381a): the data of all the kinds; each kind
// writes its own part.
#[derive(Clone)]
pub struct Event {
    pub kind: EvKind,
    pub empire: i16,  // 02 owner, -1 when cancelled
    pub created: i32, // 04
    pub expires: i32, // 08
    pub due: i32,     // 0c
    // covert operations
    pub target: i16,  // 10
    pub success: i16, // 12
    pub victim: i16,  // 14 the empire made to believe it was attacked
    // bank investments
    pub amount: Money,      // 10
    pub m14: Money,         // 14
    pub name: [u8; 0x4e],   // 18
    pub year: i16,          // 66 matures in this year of the empire
    pub issuer: i16,        // 68 empire that issued the bonds, -1 = bank
}

impl Event {
    fn blank(kind: EvKind) -> Event {
        Event {
            kind,
            empire: 0,
            created: 0,
            expires: 0,
            due: 0,
            target: 0,
            success: 0,
            victim: 0,
            amount: Money::default(),
            m14: Money::default(),
            name: [0; 0x4e],
            year: 0,
            issuer: 0,
        }
    }

    // 4c48:0000: expires after two weeks; due now.
    fn init(&mut self, g: &Sre, id: i16) {
        self.empire = id;
        self.created = g.time.now;
        self.expires = g.time.now.wrapping_add(0x127500);
        self.due = g.time.now;
    }

    // 4c48:0392
    pub fn covert(g: &Sre, kind: EvKind, id: i16, target: i16, success: i16) -> Event {
        let mut e = Event::blank(kind);
        e.init(g, id);
        e.target = target;
        e.success = success;
        e
    }

    // 4c48:1a60
    pub fn setup(g: &Sre, id: i16, target: i16, victim: i16, success: i16) -> Event {
        let mut e = Event::covert(g, EvKind::Setup, id, target, success);
        e.victim = victim;
        e
    }

    // 4c48:2773
    pub fn bank(g: &Sre, id: i16, name: &[u8], amount: i32, year: i16, issuer: i16) -> Event {
        let mut e = Event::blank(EvKind::Bank);
        e.init(g, id);
        e.amount.v = 0;
        e.m14.v = 0;
        e.amount.set(0);
        e.name = [0; 0x4e];
        strcpy(&mut e.name, name);
        e.year = year;
        e.amount.set(amount);
        e.issuer = issuer;
        e
    }

    // 4c48:011d
    pub fn set_due(&mut self, g: &Sre, days: i16, hours: i16, minutes: i16) {
        self.due = g
            .time
            .now
            .wrapping_add(86400 * days as i32)
            .wrapping_add(3600 * hours as i32)
            .wrapping_add(60 * minutes as i32);
    }

    // 4c48:0218
    fn base_due(&self) -> bool {
        b_time() >= self.due
    }

    // [14]
    pub fn is_due(&self, g: &Sre) -> bool {
        match self.kind {
            // 4c48:1b57: only while the target is playing.
            EvKind::Setup => {
                if g.current == -1 {
                    return false;
                }
                self.base_due() && self.target == g.emp[g.current as usize].id
            }
            // 4c48:298d
            EvKind::Bank => self.base_due() && g.emp[self.empire as usize].year >= self.year,
            _ => self.base_due(),
        }
    }

    // [1c]
    fn display(&self, g: &mut Sre) {
        let s: &str = match self.kind {
            EvKind::Dissension => "Dissension", // 4c48:0502
            EvKind::Demoralize => "Demoralize", // 4c48:0bc7
            EvKind::Hostages => "Hostages",     // 4c48:1266
            EvKind::Setup => "Setup",           // 4c48:1b9d
            EvKind::Bank => return,
        };
        kout!(g; setw(20), s, setw(0));
    }

    // [18]
    fn execute(&mut self, g: &mut Sre) -> Option<Event> {
        match self.kind {
            EvKind::Dissension => self.exec_dissension(g),
            EvKind::Demoralize => self.exec_demoralize(g),
            EvKind::Hostages => self.exec_hostages(g),
            EvKind::Setup => self.exec_setup(g),
            EvKind::Bank => return self.exec_bank(g),
        }
        None
    }

    // 4c48:052e
    fn exec_dissension(&mut self, g: &mut Sre) {
        let a = self.empire as usize;
        let t = self.target as usize;
        if self.success != 0 {
            let ra = report_out_to(g, g.emp[a].id, None);
            let rt = report_out_to(g, g.emp[t].id, None);
            let n = g.emp[t].soldiers / (((7 * b_rand() as i32) / 0x8000) + 9);
            out!(g, rt; cnum(g), n, cn(g), " soldiers dissented from your army due to propaganda.");
            report_set_time(g, rt, self.due);
            out!(g, ra; "You successfully supported dissension of ", cnum(g), n, cn(g), " soldiers of the army of ",
                 cemp(g), uname(g, t), cn(g), b'.');
            report_set_time(g, ra, self.due);
            g.emp[t].soldiers -= n;
            g.emp[a].agents += 1;
            report_done(g, rt);
            report_done(g, ra);
        } else if g.emp[t].exists() {
            let ra = report_out_to(g, g.emp[a].id, None);
            let rt = report_out_to(g, g.emp[t].id, None);
            out!(g, ra; "You failed to cause dissension in ", cemp(g), uname(g, t), cn(g), b'.');
            report_set_time(g, ra, self.due);
            out!(g, rt; "A covert agent from ", cemp(g), uname(g, a), cn(g),
                 " was caught trying to support dissension and was executed!");
            report_set_time(g, rt, self.due);
            if g.grb(&R_FAILEDCOVERT) {
                out!(g, NEWS; cemp(g), uname(g, a), cn(g), " was caught trying to support dissension in ", col(13),
                     uname(g, t), cn(g), ".\n");
            }
            report_done(g, rt);
            report_done(g, ra);
        }
    }

    // 4c48:0bf3
    fn exec_demoralize(&mut self, g: &mut Sre) {
        let a = self.empire as usize;
        let t = self.target as usize;
        if self.success != 0 {
            let ra = report_out_to(g, g.emp[a].id, None);
            let rt = report_out_to(g, g.emp[t].id, None);
            out!(g, ra; "You succeeded in demoralizing troops of ", cemp(g), uname(g, t), cn(g), b'.');
            report_set_time(g, ra, self.due);
            out!(g, rt; "Mysterious shipments of alcohol demoralized your troops.");
            report_set_time(g, rt, self.due);
            let drop = (g.emp[t].effectiveness as i32 / (((7 * b_rand() as i32) / 0x8000) + 7)) as i8;
            let x = &mut g.emp[t];
            x.effectiveness = (x.effectiveness as i32 - drop as i32) as i8;
            if x.effectiveness < 0 {
                x.effectiveness = 0;
            }
            if x.effectiveness > 100 {
                x.effectiveness = 100;
            }
            g.emp[a].agents += 1;
            report_done(g, rt);
            report_done(g, ra);
        } else if g.emp[t].exists() {
            let ra = report_out_to(g, g.emp[a].id, None);
            let rt = report_out_to(g, g.emp[t].id, None);
            out!(g, ra; "You failed to demoralize troops in ", cemp(g), uname(g, t), cn(g), b'.');
            report_set_time(g, ra, self.due);
            out!(g, rt; "A covert agent from ", cemp(g), uname(g, a), cn(g),
                 " was caught trying to demoralize troops and was executed!");
            report_set_time(g, rt, self.due);
            if g.grb(&R_FAILEDCOVERT) {
                out!(g, NEWS; cemp(g), uname(g, a), cn(g), " was caught trying to demoralize troops of ", col(13),
                     uname(g, t), cn(g), ".\n");
            }
            report_done(g, rt);
            report_done(g, ra);
        }
    }

    // 4c48:1292
    fn exec_hostages(&mut self, g: &mut Sre) {
        let a = self.empire as usize;
        let t = self.target as usize;
        if self.success != 0 {
            let mut m = g.emp[t].money.get() / (((3 * b_rand() as i32) / 0x8000) + 5);
            let mut b = g.emp[t].bank.get() / (((5 * b_rand() as i32) / 0x8000) + 7);
            let ra = report_out_to(g, g.emp[a].id, None);
            let rt = report_out_to(g, g.emp[t].id, None);
            if b < 0 {
                b = 0;
            }
            out!(g, rt; "Terrorists took hostages and you had to pay ", cnum(g), m.wrapping_add(b), cn(g),
                 " credits as ransom.");
            report_set_time(g, rt, self.due);
            out!(g, ra; "Your covert agent succeeded in taking ", cnum(g), m.wrapping_add(b), cn(g), " credits from ",
                 cemp(g), uname(g, t), cn(g), " as ransom for hostages.");
            report_set_time(g, ra, self.due);
            g.emp[t].money.sub(m);
            g.emp[t].bank.sub(b);
            if g.emp[a].money.get().wrapping_add(b).wrapping_add(m) > 0 {
                g.emp[a].money.add(b.wrapping_add(m));
            } else {
                // Fill the treasury, then the bank, in steps.
                while m.wrapping_add(b) > 10000 {
                    if g.emp[a].money.get() < 2000000000 {
                        g.emp[a].money.add(10000);
                    } else {
                        g.emp[a].bank.add(10000);
                    }
                    if m > 10000 {
                        m -= 10000;
                    } else if b > 10000 {
                        b -= 10000;
                    } else {
                        m += b;
                        b = 0;
                    }
                }
                g.emp[a].money.add(m.wrapping_add(b));
            }
            g.emp[a].agents += 1;
            report_done(g, rt);
            report_done(g, ra);
        } else if g.emp[t].exists() {
            let ra = report_out_to(g, g.emp[a].id, None);
            let rt = report_out_to(g, g.emp[t].id, None);
            out!(g, ra; "You failed to take hostages in ", cemp(g), uname(g, t), cn(g), ".\n");
            report_set_time(g, ra, self.due);
            out!(g, rt; "A covert agent from ", cemp(g), uname(g, a), cn(g), " was caught trying to take hostages and ");
            let pr = random_pronoun(g, 0);
            out!(g, rt; pr, " was executed!");
            report_set_time(g, rt, self.due);
            if g.grb(&R_FAILEDCOVERT) {
                out!(g, NEWS; cemp(g), uname(g, a), cn(g), " was caught trying to take hostages from ", col(13),
                     uname(g, t), cn(g), ".\n");
            }
            report_done(g, rt);
            report_done(g, ra);
        }
    }

    // 4c48:1bc9: make the victim believe the target attacked it.
    fn exec_setup(&mut self, g: &mut Sre) {
        let a = self.empire as usize;
        let t = self.target as usize;
        let v = self.victim as usize;
        if self.success != 0 {
            let ra = covert_report(g, g.emp[a].id);
            out!(g, ra; "Your covert agent was successful in setting up ", cemp(g), uname(g, t), cn(g), " and ", cemp(g),
                 uname(g, v), cn(g), b'.');
            let rv = battle_report_to(g, g.emp[v].id);
            let sev = (-(((5 * b_rand() as i32) / 0x8000) as i16) - 1) as i16;
            let tn = ename(g, t);
            attack_message(g, rv, &tn, sev);
            attack_news(g, t, v, false, sev);
            let (tid, vid) = (g.emp[t].id as usize, g.emp[v].id as usize);
            set_treaty(&mut g.emp[t].treaty[vid], 0, 0);
            g.emp[t].treaty[vid].setc(0);
            set_treaty(&mut g.emp[v].treaty[tid], 0, 0);
            g.emp[v].treaty[tid].setc(0);
            g.emp[a].agents += 1;
            report_done(g, rv);
            report_done(g, ra);
        } else if g.emp[t].exists() && g.emp[v].exists() {
            let ra = covert_report(g, g.emp[a].id);
            out!(g, ra; "Your covert agent was caught setting up ", cemp(g), uname(g, t), cn(g), " and ", cemp(g),
                 uname(g, v), cn(g), ", and ");
            let pr = random_pronoun(g, 0);
            out!(g, ra; pr, " was executed.");
            if g.emp[t].id != g.emp[a].id {
                let r = covert_report(g, g.emp[t].id);
                out!(g, r; "A covert agent from ", cemp(g), uname(g, a), cn(g), " was caught trying to set you up.");
                report_done(g, r);
            }
            if g.emp[v].id != g.emp[a].id {
                let r = covert_report(g, g.emp[v].id);
                out!(g, r; "A covert agent from ", cemp(g), uname(g, a), cn(g), " was caught trying to make you think ",
                     uname(g, t), " attacked you.");
                report_done(g, r);
            }
            if g.grb(&R_FAILEDCOVERT) {
                out!(g, NEWS; cemp(g), uname(g, a), cn(g), " attempted to set up ", col(13), uname(g, t), cn(g), " and ",
                     col(13), uname(g, v), cn(g), ".\n");
            }
            report_done(g, ra);
        }
    }

    // 4c48:29cb
    pub fn display_bank(&self, g: &mut Sre, n: i16) {
        if n == 1 {
            kout!(g; col(10), "###", cnum(g), setwr(14), "Credits", setwr(7), col(15), "Turn", setwr(7), col(15), "Hours",
                  setw(0), b'\n');
            hline(g, col(3), 0x4b);
        }
        kout!(g; col(10), setw(3), n, setw(0), cnum(g), setw(14), self.amount.get(), setw(0), col(15), setw(7), self.year);
        let mut hours = (self.due.wrapping_sub(b_time()).wrapping_add(1800) / 3600) as i16;
        if hours < 0 {
            hours = 0;
        }
        kout!(g; setw(7), hours, setw(0), cn(g), "  ", cs(&self.name), b'\n');
    }

    // 4c48:2c42: the investment matures into the bank (or the treasury
    // when the bank is full); bonds are paid by their issuer.
    fn exec_bank(&mut self, g: &mut Sre) -> Option<Event> {
        let e = self.empire as usize;
        let original = self.amount.get();
        let mut paid: i32 = 0;
        if g.emp[e].bank.get() < 2000000000 && g.emp[e].bank.get().wrapping_add(self.amount.get()) > 0 {
            g.emp[e].bank.add(self.amount.get());
            paid = self.amount.get();
            self.amount.set(0);
        } else {
            while g.emp[e].bank.get() < 2000000000 && self.amount.get() > 10000000 {
                g.emp[e].bank.add(10000000);
                self.amount.sub(10000000);
                paid += 10000000;
            }
            while g.emp[e].bank.get() < 2000000000 && self.amount.get() > 10000 {
                g.emp[e].bank.add(10000);
                self.amount.sub(10000);
                paid += 10000;
            }
            if g.emp[e].money.get() < 2000000000 && g.emp[e].money.get().wrapping_add(self.amount.get()) > 0 {
                g.emp[e].money.add(self.amount.get());
                paid = paid.wrapping_add(self.amount.get());
                self.amount.set(0);
            }
        }
        let mut next = None;
        let actual = bond_payoff(g, self.issuer, paid);
        let r = report_out_to(g, self.empire, None);
        out!(g, r; "Your ", cin(g), cs(&self.name), cn(g));
        if original == 0 {
            out!(g, r; " have crashed!");
        } else if actual < paid {
            out!(g, r; " have crashed, giving you only ", cnum(g), actual, cn(g), " credits in the bank.");
            if g.emp[e].bank.get() >= paid {
                g.emp[e].bank.sub(paid.wrapping_sub(actual));
            } else if g.emp[e].money.get() >= paid.wrapping_sub(actual) {
                g.emp[e].money.sub(paid.wrapping_sub(actual));
            }
        } else {
            out!(g, r; " have returned your investment, giving you ", cnum(g), paid, cn(g), " credits in the bank.");
        }
        if self.amount.get() > 0 {
            out!(g, r; "  ", cnum(g), self.amount.get(), cn(g), " credits have been re-invested because your bank was full.");
            let mut b = Event::bank(g, self.empire, &self.name, self.amount.get(), self.year, self.issuer);
            b.set_due(g, 0, 24, 0);
            next = Some(b);
        } else if self.issuer > -1 && g.emp[self.issuer as usize].exists() {
            g.emp[self.issuer as usize].bonds -= 1;
        }
        report_set_time(g, r, self.due);
        if paid <= 0 {
            rep_of(g, r).rep.clear_recipients();
        }
        report_done(g, r);
        next
    }

    // The bytes of the object's data (all but the original's vtable
    // pointer).
    fn bytes(&self) -> Vec<u8> {
        let mut b = vec![0u8; 0x68];
        self.empire.wr(&mut b[0..]);
        self.created.wr(&mut b[2..]);
        self.expires.wr(&mut b[6..]);
        self.due.wr(&mut b[10..]);
        match self.kind {
            EvKind::Bank => {
                self.amount.wr(&mut b[14..]);
                self.m14.wr(&mut b[18..]);
                b[22..22 + 0x4e].copy_from_slice(&self.name);
                self.year.wr(&mut b[100..]);
                self.issuer.wr(&mut b[102..]);
            }
            _ => {
                self.target.wr(&mut b[14..]);
                self.success.wr(&mut b[16..]);
                self.victim.wr(&mut b[18..]);
                b.truncate(if self.kind == EvKind::Setup { 0x14 } else { 0x12 });
            }
        }
        b
    }

    fn set_bytes(&mut self, b: &[u8]) {
        self.empire = i16::rd(&b[0..]);
        self.created = i32::rd(&b[2..]);
        self.expires = i32::rd(&b[6..]);
        self.due = i32::rd(&b[10..]);
        match self.kind {
            EvKind::Bank => {
                self.amount = Money::rd(&b[14..]);
                self.m14 = Money::rd(&b[18..]);
                self.name.copy_from_slice(&b[22..22 + 0x4e]);
                self.year = i16::rd(&b[100..]);
                self.issuer = i16::rd(&b[102..]);
            }
            _ => {
                self.target = i16::rd(&b[14..]);
                self.success = i16::rd(&b[16..]);
                if self.kind == EvKind::Setup {
                    self.victim = i16::rd(&b[18..]);
                }
            }
        }
    }
}

impl Streamable for Event {
    fn class_name(&self) -> &'static [u8] {
        match self.kind {
            EvKind::Dissension => CLS_DISSENSION, // 4c48:0492
            EvKind::Demoralize => CLS_DEMORALIZE, // 4c48:0b57
            EvKind::Hostages => CLS_HOSTAGES,     // 4c48:11f6
            EvKind::Setup => CLS_SETUP,           // 4c48:1ae3
            EvKind::Bank => CLS_BANK,             // 4c48:28b3
        }
    }

    fn size(&self) -> i16 {
        match self.kind {
            EvKind::Bank => 0x6a,
            EvKind::Setup => 0x16,
            _ => 0x14,
        }
    }

    fn expired(&mut self, g: &mut Sre) -> bool {
        match self.kind {
            // 4c48:2962
            EvKind::Bank => self.amount.get() <= 0 || self.empire == -1,
            // 4c48:01b2
            _ => g.time.now > self.expires || self.empire == -1,
        }
    }

    fn on_empire_deleted(&mut self, g: &mut Sre, id: i16) {
        match self.kind {
            // 4c48:284e
            EvKind::Bank => {
                if self.empire == id {
                    if self.issuer != -1 {
                        g.emp[self.issuer as usize].bonds -= 1;
                    }
                    self.empire = -1;
                }
                if self.issuer == id {
                    self.issuer = -1;
                    self.amount.set(0);
                }
            }
            // 4c48:1ab1
            EvKind::Setup => {
                if self.empire == id || self.target == id || self.victim == id {
                    self.empire = -1;
                }
            }
            // 4c48:046c
            _ => {
                if self.empire == id || self.target == id {
                    self.empire = -1;
                }
            }
        }
    }

    fn data(&self) -> Vec<u8> {
        self.bytes()
    }

    fn set_data(&mut self, b: &[u8]) {
        self.set_bytes(b);
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

// The registry's constructors (4c48:04aa, 0b6f, 120e, 1afb, 28cb).
pub fn new_dissension() -> Box<dyn Streamable> {
    Box::new(Event::blank(EvKind::Dissension))
}
pub fn new_demoralize() -> Box<dyn Streamable> {
    Box::new(Event::blank(EvKind::Demoralize))
}
pub fn new_hostages() -> Box<dyn Streamable> {
    Box::new(Event::blank(EvKind::Hostages))
}
pub fn new_setup() -> Box<dyn Streamable> {
    Box::new(Event::blank(EvKind::Setup))
}
pub fn new_bank() -> Box<dyn Streamable> {
    Box::new(Event::blank(EvKind::Bank))
}

// A report about covert operations (vtable 3742).
pub fn covert_report(g: &mut Sre, id: i16) -> Sid {
    report_out_to(g, id, Some(COVERT_FILE))
}

// ---------------------------------------------------------------------

// 4c48:0274
pub fn events_empire_deleted(g: &mut Sre, id: i16) -> i16 {
    let mut n = 0;
    let mut f = StreamFile::new(g, EVENT_FILE);
    while let Some(mut o) = f.read(g) {
        o.on_empire_deleted(g, id);
        n += 1;
        f.keep(g, o.as_mut());
    }
    f.close(g);
    n
}

// 4c48:0358
pub fn clear_events(g: &mut Sre) {
    let f = StreamFile::new(g, EVENT_FILE);
    f.close(g);
}

// 4c48:32a9
pub fn clear_covert_reports(g: &mut Sre) {
    clear_stream_file(g, Some(COVERT_FILE));
}

// 4c48:32bd
pub fn show_covert_reports(g: &mut Sre, p: &Player, e: usize) -> i16 {
    let id = g.emp[e].id;
    show_reports(g, id, COVERT_FILE, (p.prefs2 >> 3) & 1 != 0, 5, 13)
}

fn as_event(o: &mut Box<dyn Streamable>) -> &mut Event {
    o.as_any().downcast_mut::<Event>().expect("not an event")
}

// 4c48:32f1: run the events that are due.
pub fn process_events(g: &mut Sre) -> i16 {
    if b_time() < g.m.nextevent {
        return 0;
    }
    galaxy_open(g);
    let mut n: i16 = 0;
    let mut f = StreamFile::new(g, EVENT_FILE);
    g.m.nextevent = 0x7fffffff;
    while let Some(mut o) = f.read(g) {
        let ev = as_event(&mut o);
        if ev.due < g.m.nextevent {
            g.m.nextevent = ev.due;
        }
        if ev.is_due(g) {
            n += 1;
            if let Some(mut nx) = ev.execute(g) {
                if nx.due < g.m.nextevent {
                    g.m.nextevent = nx.due;
                }
                f.keep(g, &mut nx);
            }
        } else {
            if ev.due < g.m.nextevent {
                g.m.nextevent = ev.due;
            }
            f.keep(g, ev);
        }
    }
    galaxy_close(g);
    f.close(g);
    n
}

// 4c48:35ce
pub fn list_investments(g: &mut Sre, id: i16) -> i16 {
    let mut n: i16 = 0;
    let mut f = StreamFile::new(g, EVENT_FILE);
    while let Some(mut o) = f.read(g) {
        let cls = o.class_name();
        let ev = as_event(&mut o);
        if str_eq_i(CLS_BANK, cls) && ev.empire == id {
            n += 1;
            ev.display_bank(g, n);
        }
        f.keep(g, o.as_mut());
    }
    f.close(g);
    if n > 0 {
        hline(g, col(3), 0x4b);
    } else {
        kout!(g; cn(g), "You have no investments.\n");
    }
    n
}

// 4c48:375d
pub fn list_covert_ops(g: &mut Sre, id: i16) -> i16 {
    let mut n: i16 = 0;
    let mut f = StreamFile::new(g, EVENT_FILE);
    while let Some(mut o) = f.read(g) {
        let cls = o.class_name();
        let ev = as_event(&mut o);
        if ev.empire == id
            && (str_eq_i(CLS_DISSENSION, cls)
                || str_eq_i(CLS_DEMORALIZE, cls)
                || str_eq_i(CLS_SETUP, cls)
                || str_eq_i(CLS_HOSTAGES, cls))
        {
            n += 1;
            if n == 1 {
                hline(g, col(5), 0x4b);
            }
            kout!(g; col(13), setw(5), n, setw(0), col(15), b' ');
            ev.display(g);
            if ev.target != -1 {
                kout!(g; cn(g), " to ", uname(g, ev.target as usize));
            }
            kout!(g; b'\n');
        }
        f.keep(g, o.as_mut());
    }
    f.close(g);
    if n > 0 {
        hline(g, col(5), 0x4b);
    } else {
        kout!(g; cn(g), "You have no covert agents out on missions.\n");
    }
    n
}

// 4c48:3b34
pub fn add_event(g: &mut Sre, e: &mut Event) {
    if e.due < g.m.nextevent {
        g.m.nextevent = e.due;
    }
    append_object(g, EVENT_FILE, e);
}

// 4c48:3bc9: the issuer of bonds pays for them, selling assets if it
// has to; returns what was actually paid.
pub fn bond_payoff(g: &mut Sre, issuer: i16, amount: i32) -> i32 {
    use crate::spending::*;
    let mut owed = Money { v: amount };
    let mut sold = false;
    if issuer == -1 {
        return amount;
    }
    let e = issuer as usize;
    if g.emp[e].exists() {
        if g.emp[e].money.get() >= owed.get() {
            let o = owed.get();
            g.emp[e].money.sub(o);
            owed.set(0);
        } else {
            owed.sub(g.emp[e].money.get());
            g.emp[e].money.set(0);
        }
        if g.emp[e].bank.get() >= owed.get() {
            let o = owed.get();
            g.emp[e].bank.sub(o);
            owed.set(0);
        } else {
            owed.sub(g.emp[e].bank.get());
            g.emp[e].bank.set(0);
        }
        while owed.get() > 0 {
            let before = owed.get();
            sold = true;
            if owed.get() > 0 && g.emp[e].soldiers > 100 {
                owed.sub(100i32.wrapping_mul(price_soldiers(g, e) / 3));
                g.emp[e].soldiers -= 100;
            }
            if owed.get() > 0 && g.emp[e].fighters > 100 {
                owed.sub(100i32.wrapping_mul(price_fighters(g, e) / 3));
                g.emp[e].fighters -= 100;
            }
            if owed.get() > 0 && g.emp[e].stations > 100 {
                owed.sub(100i32.wrapping_mul(price_stations(g, e) / 3));
                g.emp[e].stations -= 100;
            }
            if owed.get() > 0 && g.emp[e].generals > 10 {
                owed.sub(10i32.wrapping_mul(price_generals(g, e) / 3));
                g.emp[e].generals -= 10;
            }
            if owed.get() > 0 && g.emp[e].heavycruisers > 100 {
                owed.sub(100i32.wrapping_mul(price_heavy_cruisers(g, e) / 3));
                g.emp[e].heavycruisers -= 100;
            }
            if owed.get() > 0 && g.emp[e].carriers > 10 {
                owed.sub(10i32.wrapping_mul(price_carriers(g, e) / 3));
                g.emp[e].carriers -= 10;
            }
            if owed.get() > 0 && g.emp[e].agents > 10 {
                owed.sub(10i32.wrapping_mul(price_agents(g, e) / 3));
                g.emp[e].agents -= 10;
            }
            if owed.get() > 0 && g.emp[e].food > 100 {
                owed.sub(200);
                g.emp[e].food -= 100;
            }
            if owed.get() > 0 && g.emp[e].population > 100 {
                owed.sub(100);
                g.emp[e].population -= 10;
            }
            for i in 0..10 {
                if owed.get() > 0 && g.emp[e].planets[i].count > 10 {
                    owed.sub(price_planet(g, e) / 3);
                    g.emp[e].planets[i].count -= 1;
                }
            }
            if before == owed.get() {
                break;
            }
        }
        let r = report_out_to(g, issuer, None);
        out!(g, r; cnum(g), amount, cn(g), " credits were taken from you to pay for for bond payoffs.");
        if sold {
            out!(g, r; "  Some of your assets had to be sold to get cash for this expense.");
        }
        report_done(g, r);
    }
    if owed.get() > 0 {
        return amount.wrapping_sub(owed.get());
    }
    amount
}
