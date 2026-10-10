// Solar Realms Elite 0.994b, reconstructed: program entry (segment 1848).

mod advisors;
mod attack;
mod bank;
mod battle;
mod brtl;
mod civilwar;
mod conio;
mod covert;
mod crypt;
mod devices;
mod diplomacy;
mod door;
mod doubleagents;
mod editors;
mod empire;
mod events;
mod files;
mod foodmarket;
mod galaxy;
mod galaxymenu;
mod game;
mod help;
mod income;
mod itemlist;
mod layout;
mod lottery;
mod maintenance;
mod menu;
mod messages;
mod modstate;
mod news;
mod operations;
mod pirateraid;
mod pirates;
mod play;
mod player;
mod population;
mod prelude;
mod register;
mod report;
mod research;
mod reset;
mod resource;
mod screens;
mod spending;
mod sre;
mod sstring;
mod stream;
mod streamfile;
mod sysop;
mod system;
mod timeutil;
mod trading;
mod translate;
mod turnevents;
mod types;
mod upkeep;

use prelude::*;

// 1848:0067: file error handler (logs to SYSOP.LOG).
fn file_error_log(g: &mut Sre, msg: &[u8]) {
    let now = g.time.now;
    out!(g, ERRLOG; time_str(g, now), ":  ", msg.to_vec());
}

// 1848:00a2 (atexit)
pub fn main_at_exit(g: &mut Sre) {
    while g.open != 0 {
        galaxy_close(g);
    }
    host_remove(b"InUse.SR");
}

// 1848:0bde: true if another node is playing.
fn in_use(g: &mut Sre) -> bool {
    let mut stale = false;
    let now = b_time();
    {
        let f = ifstream(g, b"InUse.SR", false);
        if !ifstream_fail(g, f) {
            set_maxlen(g, f, 0x50);
            in_line(g, f);
            set_maxlen(g, f, 0);
            let t = in_i32(g, f);
            if now.wrapping_sub(t) > 3600 {
                stale = true;
            }
        } else {
            stale = true;
        }
        ifstream_done(g, f);
    }
    if !stale {
        return true;
    }
    let o = ofstream(g, b"InUse.SR", false, 0);
    out!(g, o; cs(&g.door.name), b'\n');
    out!(g, o; now, b'\n');
    out!(g, o; "\nThis file tells SRE that this user is still playing SRE.");
    out!(g, o; "\nIf you delete this file, another user will be able to play.");
    out!(g, o; "\nThis is to prevent multiple nodes from playing simultaneously.\n");
    ofstream_done(g, o);
    false
}

fn write_user_list(g: &mut Sre) {
    let o = ofstream(g, b"Text/Userlist.TXT", true, 0);
    galaxy_open(g);
    for i in 0..MAXEMPIRES {
        if !g.emp[i].exists() {
            continue;
        }
        let mut p = Player::default();
        let pl = g.emp[i].player;
        player::player_load(g, &mut p, pl);
        out!(g, o; cs(&p.user), b'\n');
        player::player_save(g, &mut p);
    }
    galaxy_close(g);
    ofstream_done(g, o);
}

// 1848:00c1
fn real_main(g: &mut Sre) -> i32 {
    b_srand(b_time() as u16);
    kout!(g; col(7), b'\n');
    let flavor = g.grs(&R_FLAVOR);
    if !flavor.is_empty() {
        g.scr.flavordir = cat(&[b"flavors\\", &flavor, b"\\"]);
        let f = cat(&[&g.scr.flavordir, b"resource.dat"]);
        g.resdb.load_file(&f);
    }
    {
        let mut sb = Resource::num(b"StatusBar.Background", g.scr.sbcolor as i32);
        g.scr.sbcolor = sb.long(&mut g.resdb) as i16;
        g.dev.sbback = g.scr.sbcolor as u8;
        g.dev.sbmid = (g.scr.sbcolor + 8) as u8;
        g.dev.sbfore = 15;
    }
    set_file_error_handler(g, Some(file_error_log));
    door::door_read(g);
    // The terminal is the sysop's screen for the local functions and for
    // local play; otherwise it is the remote user's.
    g.dev.console = g.door.port == 0;
    for a in [&b"Reset"[..], b"Register", b"Sysop", b"Edit", b"Userlist", b"Maintenance"] {
        if g.has_command_arg(a) {
            g.dev.console = true;
        }
    }
    door::comm_init(g);
    let mut statusbars = Resource::num(b"System.StatusBars", 1);
    if statusbars.bool(&mut g.resdb) {
        status_bar(g, b"SRE:II", b"", b'*');
    } else {
        g.scr.statusbars = 0;
    }
    // The original reports coreleft() and its overlay buffer; these are
    // the figures it gives in a 640K DOS session.
    kout!(g; cnum(g), 285520i32, cn(g), " bytes free");
    let mut expanded = Resource::num(b"ExpandedSwapping", 0);
    let mut extended = Resource::num(b"ExtendedSwapping", 0);
    kout!(g; "; ", cnum(g), 86016i32, cn(g), " bytes (conventional) used for swap space.\n");
    if extended.bool(&mut g.resdb) {
        kout!(g; "Extended memory swapping\n");
    } else if expanded.bool(&mut g.resdb) {
        kout!(g; "EMS swapping\n");
    } else {
        kout!(g; "Disk swapping\n");
    }
    play::time_start(g);
    if in_use(g) {
        out!(g, KLOCAL; "Error:  Game in use on another node.\n");
        door::comm_ports(g);
        door::command_line(g);
        door::door_open(g);
        kout!(g; "Sorry, it appears that another user is playing this game of SRE at this time.\n");
        system_clear_in_use(g);
        g.exit(2);
    }
    g.atexit.push(AtExit::Game);
    set_idle(g, Some(play::idle_hook));
    if g.has_command_arg(b"Reset") {
        if !system_valid(g) || g.system.sysop[0] == 0 {
            reset::reset_game(g);
        }
        sysop::sysop_menu(g);
        return 0;
    }
    if !system_valid(g) || g.system.bbs[0] == 0 {
        kout!(g; "Sorry, this game has not been reset.\n");
        out!(g, KMODEM; "Please inform your sysop.\n");
        return 1;
    }
    if g.has_command_arg(b"Register") {
        if !register::register(g) {
            kout!(g; cn(g), "Registration has not been completed.\n");
        }
        return 0;
    }
    if g.has_command_arg(b"Sysop") || g.has_command_arg(b"Edit") {
        sysop::sysop_menu(g);
        return 0;
    }
    if g.has_command_arg(b"Userlist") {
        write_user_list(g);
        return 0;
    }
    if g.has_command_arg(b"Maintenance") {
        kout!(g; "Running system maintenance:\n");
        system_maintenance(g);
        return 0;
    }
    research::research_init(g);
    door::comm_ports(g);
    door::command_line(g);
    door::door_open(g);
    if g.door.port == 0 {
        g.dev.pagelen -= 1;
    }
    if g.door.port != 0 && g.door.baud <= 1200 {
        g.dev.effects = 0;
    }
    door::lockout_check(g);
    let mut blank = Resource::num(b"System.BlankLocalScreen", 0);
    if blank.bool(&mut g.resdb) && g.door.port != 0 {
        kout_m(g).localon = 0;
        out!(g, KLOCAL; "Someone is playing SRE:II!\n");
    }
    title_screen(g);
    let dn = cs(&g.door.name);
    status_bar(g, &dn, b"SRE", b'*');
    if kout_m(g).logon != 0 {
        g.ost(KLOG).noansi = 1;
        let mut line = b"\n\xda".to_vec();
        line.extend(std::iter::repeat(0xc4u8).take(77));
        line.push(b'\n');
        out!(g, KLOG; col(8), line);
        out!(g, KLOG; 0xb3u8, col(15), " Log File for '", dn, "':  \n", cn(g));
    }
    load_translation(g);
    main_menu(g);
    score_files(g);
    if g.system.regstate != 0 {
        if random(3) == 0 {
            put_untranslated(g, KOUT, b"\nThis game is registered to ");
            kout!(g; col(14));
            let r = cs(&g.system.regto);
            put_untranslated(g, KOUT, &r);
            kout!(g; cn(g), b'\n');
        }
    } else if !g.grb(&R_UNREGISTERED) && random(32) == 0 {
        put_untranslated(g, KOUT, b"\nThis game is currently unregistered!\n");
    }
    news_files(g, 0);
    0
}

// Testing aid: SRE_DUMP=<file> writes the empire array at exit (before
// it is saved), to compare with the original's memory.
pub fn dump_empires(g: &mut Sre) {
    if let Some(f) = std::env::var_os("SRE_DUMP") {
        let mut b = Vec::new();
        for e in g.emp.iter() {
            b.extend(e.to_bytes());
        }
        let _ = std::fs::write(f, b);
    }
}

// 1848:00c1 (end): write the score files named by Scores.File.*.
fn score_files(g: &mut Sre) {
    galaxy_open(g);
    if g.open != 0 {
        let mut r = Resource::text(b"Scores.File.ANSI", Some(b"SREScore.ANS"));
        let name = r.str(&mut g.resdb);
        if !name.is_empty() {
            let f = ofstream(g, &name, false, 0);
            g.ost(f).noansi = 0;
            let mask = empire_mask(g);
            score_list(g, mask, Some(f));
            ofstream_done(g, f);
        }
    }
    if g.open != 0 {
        let mut r = Resource::text(b"Scores.File.Text", Some(b"SREScore.TXT"));
        let name = r.str(&mut g.resdb);
        if !name.is_empty() {
            let f = ofstream(g, &name, false, 0);
            g.ost(f).noansi = 1;
            let mask = empire_mask(g);
            score_list(g, mask, Some(f));
            ofstream_done(g, f);
        }
    }
    galaxy_close(g);
}

// 1848 main + Borland start-up: static data, then main.
fn main() {
    use std::os::unix::ffi::OsStringExt;
    let argv: Vec<Vec<u8>> = std::env::args_os().map(|a| a.into_vec()).collect();
    // Static initialisation: the door's start time (play.rs) is taken
    // before anything else, from the host's clock.
    let start = b_time();
    if let Some(dir) = std::env::var_os("SRE_DIR") {
        if std::env::set_current_dir(&dir).is_err() {
            eprintln!("sre: cannot change to {}", dir.to_string_lossy());
            std::process::exit(1);
        }
    }
    if let Some(clk) = std::env::var_os("SRE_CLOCK") {
        b_setclock(b_atol(&clk.into_vec()), 0);
    }
    let mut sre = Sre::new(argv);
    let g = &mut sre;
    g.m.start = start;
    term_init(g);
    init_time(g);
    for i in 0..MAXEMPIRES {
        g.emp[i] = Empire::default();
        empire_init(g, i);
    }
    for i in 0..MAXPIRATES {
        pirates::pirate_new(g, i);
    }
    galaxy_init(g);
    system_load(g);
    g.atexit.push(AtExit::DumpEmpires);
    let r = real_main(g);
    g.exit(r);
}
