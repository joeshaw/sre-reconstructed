// Door file, terminal set-up and command line (segments 7389 and 4018).

use crate::prelude::*;

const DOOR_FILE: &[u8] = b"DOORFILE.SR"; // DS:bc10

// 7389:0000
fn door_defaults(d: &mut DoorInfo) {
    d.name[0] = 0;
    d.ansi = 1;
    d.ibm = 1;
    d.pagelen = 0x19;
    d.baud = 0x960;
    d.port = 0;
    d.minutes = -1;
    d.loaded = 0;
}

// 7389:006f: read DOORFILE.SR (name, ansi, ibm, page length, baud,
// port, minutes, ...), then the resources that override it.
pub fn door_read(g: &mut Sre) {
    let mut d = g.door.clone();
    door_defaults(&mut d);
    let f = ifstream(g, DOOR_FILE, false);
    if !ifstream_fail(g, f) {
        let name = in_line(g, f);
        set_name(&mut d.name, &name);
        d.ansi = b_atol(&in_line(g, f)) as i16;
        d.ibm = b_atol(&in_line(g, f)) as i16;
        d.pagelen = b_atol(&in_line(g, f)) as i16;
        let v = b_atol(&in_line(g, f));
        d.baud = if v > 0xffff { 0x9600 } else { v as u16 };
        d.port = b_atol(&in_line(g, f)) as i16;
        let v = b_atol(&in_line(g, f));
        d.minutes = if v > 0x7fff { 0x7fff } else { v as i16 };
    }
    if !ifstream_fail(g, f) {
        d.loaded = 1;
    }
    ifstream_done(g, f);
    d.lockbaud = d.baud;
    let mut ransi = Resource::num(b"Ansi", d.ansi as i32);
    let mut ribm = Resource::num(b"IBMCharacters", d.ibm as i32);
    let mut rcheck = Resource::num(b"CheckTimeLimit", 1);
    let mut rlocal = Resource::num(b"Local", 0);
    let mut rlock = Resource::num(b"LockBaudRate", 0);
    let mut rtimeout = Resource::num(b"TimeOut", g.dev.timeout as i32);
    g.dev.timeout = rtimeout.long(&mut g.resdb) as i16;
    if ransi.long(&mut g.resdb) != -1 {
        d.ansi = ransi.bool(&mut g.resdb) as i16;
    }
    if ribm.long(&mut g.resdb) != -1 {
        d.ibm = ribm.bool(&mut g.resdb) as i16;
    }
    if !rcheck.bool(&mut g.resdb) {
        d.minutes = -1;
    }
    if rlocal.bool(&mut g.resdb) {
        d.port = 0;
    }
    if rlock.long(&mut g.resdb) != 0 {
        let b = rlock.long(&mut g.resdb);
        if (300..=0xffff).contains(&b) {
            d.baud = b as u16;
            out!(g, KLOCAL; "Locking baud rate at ", col(11), d.baud, col(7), " baud.\n");
        } else {
            out!(g, KLOCAL; col(14), "Invalid resource 'LockBaudRate':  baud rate must be from 300 to 65535 baud.\n", col(7));
        }
    }
    g.dev.ansi = d.ansi;
    g.dev.ibm = d.ibm;
    g.door = d;
}

// 7389:04b1
pub fn door_open(g: &mut Sre) -> bool {
    if g.door.loaded == 0 {
        kout!(g; DOOR_FILE, " not found.  Please run the install program!\n");
        g.exit(1);
    }
    if g.door.pagelen == 0 || g.door.pagelen < 10 {
        g.door.pagelen = 0x17;
    }
    g.dev.remote = g.door.port;
    if g.door.port == 0 {
        if g.door.ansi == -1 {
            g.door.ansi = 1;
        }
    } else if g.door.ansi == -1 {
        kout!(g; "Do you have ANSI color?  [Y/n] ");
        g.ost(KOUT).raw = 1;
        let mut c;
        loop {
            c = b_toupper(in_char(g, KIN));
            if c == b'\r' {
                c = b'Y';
            }
            if c == b'Y' || c == b'N' {
                break;
            }
        }
        g.ost(KOUT).raw = 0;
        g.door.ansi = (c == b'Y') as i16;
        kout!(g; c, b'\n');
    }
    if g.door.ibm == -1 {
        g.door.ibm = g.door.ansi;
    } else {
        g.door.ibm &= 1;
    }
    g.dev.ansi = g.door.ansi;
    g.dev.ibm = g.door.ibm;
    g.dev.remote = g.door.port;
    g.ost(KOUT).noansi = if g.dev.ansi != 0 { 0 } else { 1 };
    true
}

// ---------------------------------------------------------------------
// 4018:0000: COM port addresses and interrupts (no effect here).
pub fn comm_ports(g: &mut Sre) {
    for i in 1..=8 {
        let name = cat(&[b"COM", &int_str(i)]);
        let mut r = Resource::text(&name, Some(b""));
        r.str(&mut g.resdb);
    }
}

// 4018:01dc
pub fn comm_init(g: &mut Sre) {
    let mut effects = Resource::num(b"SpecialEffects", g.dev.effects as i32);
    let _video = Resource::text(b"Display.Video", Some(b"Direct"));
    let _vga = Resource::num(b"Display.VGA", 0);
    let _multi = Resource::num(b"System.Multitasker", 0);
    let _fossil = Resource::num(b"System.Fossil", 0);
    let _bios = Resource::num(b"BIOS", 0);
    g.dev.effects = effects.bool(&mut g.resdb) as i16;
    {
        let mut comma = Resource::num(b"Output.Comma", 1);
        if !comma.bool(&mut g.resdb) {
            let s = comma.str(&mut g.resdb);
            g.comma = if s.len() == 1 { s[0] } else { 0 };
        }
    }
    {
        let mut sep = Resource::text(b"Output.Time.Separator", Some(b":"));
        let s = sep.str(&mut g.resdb);
        if s.len() == 1 {
            g.time.timesep = s[0];
        }
    }
    let mut h24 = Resource::num(b"Output.Time.24Hour", 0);
    g.time.time24 = h24.bool(&mut g.resdb) as i16;
}

// 4018:0603
pub fn command_line(g: &mut Sre) {
    if g.resdb.argv.len() > 1 && b_stricmp(&g.resdb.argv[1], b"/?") == 0 {
        crate::help::show_help_topic(g, b"SRE-command-line-options");
        pause_prompt(g);
        g.exit(0);
    }
    if g.has_command_arg(b"HELP") {
        status_bar(g, b"Help", b"", b'?');
        crate::help::more_help(g);
        pause_prompt(g);
        g.exit(0);
    }
    if g.has_command_arg(b"CreateINSTRUCTIONS") {
        status_bar(g, b"Creating Instructions Files", b"!", b'?');
        g.dev.pagelen = 0;
        kout_m(g).localon = 0;
        open_log(g, b"Text/Player.ANS");
        g.ost(KLOG).noansi = 0;
        display_file(g, b"Text/Player.MSG", 0);
        close_log(g);
        open_log(g, b"Text/Player.TXT");
        g.ost(KLOG).noansi = 1;
        display_file(g, b"Text/Player.MSG", 0);
        close_log(g);
        g.exit(0);
    }
    let mut log = Resource::text(b"Game.Log", Some(b""));
    let l = log.str(&mut g.resdb);
    if !l.is_empty() {
        open_log(g, &l);
    }
}

// 4018:078a
pub fn lockout_check(g: &mut Sre) {
    let f = ifstream(g, b"Text/Lockout.LST", false);
    while !ifstream_fail(g, f) {
        let name = in_line(g, f);
        if b_stricmp(&name, cstr(&g.door.name)) == 0 {
            if !show_screen(g, None, b"Lockout", false) {
                kout!(g; "You are not to play this game.\n");
            }
            g.exit(1);
        }
    }
    ifstream_done(g, f);
}
