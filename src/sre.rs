// The game's state: the original's global variables, in one place.

use crate::conio::ConState;
use crate::devices::DevState;
use crate::game::*;
use crate::menu::MenuChars;
use crate::resource::{ResDb, Resource};
use crate::screens::ScreenState;
use crate::stream::*;
use crate::timeutil::TimeState;
use crate::translate::TransState;

// Colours that the game keeps in variables.
pub struct Colors {
    pub number: Color,  // 677e Output.Number.Color
    pub bracket: Color, // 6786 Output.Bracket.Color
    pub bold: Color,    // 678a Output.Bold.Color
    pub empire: Color,  // 678e Output.Empire.Color
    pub white: Color,   // 6782
    pub c6792: Color,
    pub c6796: Color,
    pub c679a: Color,
    pub c679e: Color,
    pub c67a2: Color,
    pub green: Color, // 0094
}

impl Default for Colors {
    fn default() -> Self {
        Colors {
            number: col(11),
            bracket: col(9),
            bold: col(11),
            empire: col(15),
            white: col(15),
            c6792: col(4),
            c6796: col(4),
            c679a: col(1),
            c679e: col(4),
            c67a2: col(8),
            green: col(10),
        }
    }
}

// A resource the original keeps in a global object: it is looked up
// once, on first use, and keeps its value.
pub struct GRes {
    pub name: &'static [u8],
    pub num: i32,
    pub text: Option<&'static [u8]>,
}

impl GRes {
    pub const fn num(name: &'static [u8], v: i32) -> GRes {
        GRes { name, num: v, text: None }
    }
    pub const fn text(name: &'static [u8], t: &'static [u8]) -> GRes {
        GRes { name, num: 0, text: Some(t) }
    }
}

pub static R_USERBREAK: GRes = GRes::num(b"UserBreak", 1); // devices
pub static R_UNREGISTERED: GRes = GRes::num(b"System.Unregistered", 0); // 50d0
pub static R_FLAVOR: GRes = GRes::num(b"Flavor", 0); // 50dc
pub static R_INTLASK: GRes = GRes::num(b"Output.InternationalAsk", 0); // 1d70
pub static R_INACTIVETIMEOUT: GRes = GRes::num(b"Game.InactiveTimeOut", 10); // 3eaa
pub static R_FAILEDCOVERT: GRes = GRes::num(b"News.FailedCovertOps", 1); // 3eb6
pub static R_ATTACKVICTIM: GRes = GRes::num(b"News.AttackVictim", 1); // 3ec2
pub static R_PIRATERAIDS: GRes = GRes::num(b"News.PirateRaids", 0); // 3ece
pub static R_CIVILWAR: GRes = GRes::num(b"News.CivilWar", 1); // 3eda

// What exit() runs, last registered first (the original's atexit()).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AtExit {
    DumpEmpires,
    Game,
}

pub struct Sre {
    pub os: Vec<Option<OStream>>,
    pub is: Vec<Option<IStream>>,
    pub dev: DevState,
    pub con: ConState,
    pub resdb: ResDb,
    pub gres: std::collections::HashMap<&'static [u8], Resource>,
    pub trans: TransState,
    pub translate_hook: bool, // DS:be63
    pub comma: u8,            // Output.Comma (DS:be62), 0 = none
    pub time: TimeState,
    pub c: Colors,
    pub menuchars: MenuChars,
    pub scr: ScreenState,
    pub fileerror: Option<crate::files::FileErrorFn>,
    pub atexit: Vec<AtExit>,

    // The game.
    pub emp: Vec<Empire>,   // DS:c92e
    pub pir: Vec<Pirate>,   // DS:c932
    pub galaxy: Galaxy,     // DS:cd6a
    pub system: System,     // DS:d196
    pub door: DoorInfo,     // DS:d360
    pub current: i16,       // DS:2b54 (-1: none)
    pub open: i16,          // DS:3ee6 galaxy open count
    pub newreports: i16,    // DS:91d2
    pub regname: crate::sstring::Str,
    pub m: crate::modstate::ModState,
}

impl Sre {
    pub fn new(argv: Vec<Vec<u8>>) -> Sre {
        let mut os: Vec<Option<OStream>> = Vec::new();
        os.push(Some(crate::devices::new_kout()));
        os.push(Some(OStream::new(0, Kind::Local)));
        os.push(Some(OStream::new(0, Kind::Modem)));
        os.push(Some(OStream::new(0, Kind::File(Default::default()))));
        os.push(Some(OStream::new(0, Kind::Window)));
        os.push(Some(crate::news::new_news_stream()));
        os.push(Some(crate::news::new_sysop_log()));
        let is = vec![Some(crate::devices::new_kin()), Some(IStream::new(IKind::Window { pending: 0 }))];
        let mut resdb = ResDb::default();
        resdb.argv = argv;
        Sre {
            os,
            is,
            dev: DevState::default(),
            con: ConState::default(),
            resdb,
            gres: std::collections::HashMap::new(),
            trans: TransState::default(),
            translate_hook: false,
            comma: b',',
            time: TimeState::default(),
            c: Colors::default(),
            menuchars: MenuChars::default(),
            scr: ScreenState::default(),
            fileerror: None,
            atexit: Vec::new(),
            emp: (0..MAXEMPIRES).map(|_| Empire::default()).collect(),
            pir: (0..MAXPIRATES).map(|_| Pirate::default()).collect(),
            galaxy: Galaxy::default(),
            system: System::default(),
            door: DoorInfo::default(),
            current: -1,
            open: 0,
            newreports: 1,
            regname: Vec::new(),
            m: Default::default(),
        }
    }

    pub fn ost(&mut self, s: Sid) -> &mut OStream {
        self.os[s].as_mut().expect("stream closed")
    }

    pub fn ist(&mut self, i: Iid) -> &mut IStream {
        self.is[i].as_mut().expect("stream closed")
    }

    pub fn alloc_os(&mut self, o: OStream) -> Sid {
        for i in NFIXED_OS..self.os.len() {
            if self.os[i].is_none() {
                self.os[i] = Some(o);
                return i;
            }
        }
        self.os.push(Some(o));
        self.os.len() - 1
    }

    pub fn free_os(&mut self, s: Sid) -> OStream {
        self.os[s].take().expect("stream closed")
    }

    pub fn alloc_is(&mut self, i: IStream) -> Iid {
        for k in NFIXED_IS..self.is.len() {
            if self.is[k].is_none() {
                self.is[k] = Some(i);
                return k;
            }
        }
        self.is.push(Some(i));
        self.is.len() - 1
    }

    pub fn free_is(&mut self, i: Iid) {
        self.is[i] = None;
    }

    pub fn has_command_arg(&self, s: &[u8]) -> bool {
        self.resdb.has_command_arg(s)
    }

    fn gres(&mut self, d: &GRes) -> &mut Resource {
        self.gres.entry(d.name).or_insert_with(|| match d.text {
            Some(t) => Resource::text(d.name, Some(t)),
            None => Resource::num(d.name, d.num),
        })
    }

    pub fn grb(&mut self, d: &GRes) -> bool {
        self.gres(d);
        let r = self.gres.get_mut(d.name).unwrap();
        r.bool(&mut self.resdb)
    }

    pub fn grl(&mut self, d: &GRes) -> i32 {
        self.gres(d);
        let r = self.gres.get_mut(d.name).unwrap();
        r.long(&mut self.resdb)
    }

    pub fn grs(&mut self, d: &GRes) -> crate::sstring::Str {
        self.gres(d);
        let r = self.gres.get_mut(d.name).unwrap();
        r.str(&mut self.resdb)
    }

    pub fn res_bool_userbreak(&mut self) -> bool {
        self.grb(&R_USERBREAK)
    }

    pub fn res_intlask(&mut self) -> bool {
        self.grb(&R_INTLASK)
    }

    // exit(): the atexit functions, the static objects' destructors (the
    // files they hold are closed), then the terminal.
    pub fn exit(&mut self, code: i32) -> ! {
        while let Some(f) = self.atexit.pop() {
            match f {
                AtExit::Game => crate::main_at_exit(self),
                AtExit::DumpEmpires => crate::dump_empires(self),
            }
        }
        if self.os[KOUT].is_some() && crate::devices::kout_m(self).logopen != 0 {
            crate::devices::close_log(self);
        }
        for s in 0..self.os.len() {
            if let Some(o) = self.os[s].as_mut() {
                if let Kind::File(f) = &mut o.kind {
                    if let Some(mut w) = f.fp.take() {
                        use std::io::Write;
                        let _ = w.flush();
                    }
                }
            }
        }
        crate::devices::term_restore(self);
        std::process::exit(code)
    }
}
