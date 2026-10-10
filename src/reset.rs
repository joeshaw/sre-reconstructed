// Resetting the game (segment 6924).

use crate::prelude::*;

// 6924:0000: ask for a number in [min, max].
pub fn ask_number(g: &mut Sre, prompt: &[u8], min: i16, max: i16, def: i16) -> i16 {
    let mut v: i16;
    loop {
        kout!(g; prompt, "?  ", cbr(g), b'[', cbo(g));
        kout!(g; col(9), "MAX=", col(3), max, cbr(g), b']', cn(g), b' ');
        set_defval(g, KIN, def as i32);
        set_maxval(g, KIN, max as i32);
        v = in_i16(g, KIN);
        if !(v < min || v > max) {
            break;
        }
    }
    v
}

// 6924:0128: set up a new game.
pub fn reset_galaxy(g: &mut Sre) {
    let mut localonly = Resource::num(b"System.LocalOnly", 0);
    g.galaxy.w386 = localonly.bool(&mut g.resdb) as i16;
    host_mkdir(b"DATA");
    host_mkdir(b"BACKUP");
    if g.galaxy.w386 != 0 {
        kout!(g; "This game is configured for LOCAL PLAY ONLY.\n\n");
        kout!(g; "You will be able to play an unlimited number of turns.\n");
    }
    let help = crate::help::show_help_topic;
    help(g, b"reset-inflation");
    g.galaxy.w384 = yes_no(g, 1, b"Do you want Inflation") as i16;
    help(g, b"reset-maintenance-interval");
    g.galaxy.grantdays = ask_number(g, b"How many days before maintenance runs", 1, 14, 1);
    if g.system.regstate >= 2 {
        g.galaxy.grantturns = ask_number(g, b"How many turns should be allowed per maintenance interval", 1, 12, 5);
        g.galaxy.protection = ask_number(g, b"How many turns of protection", 0, 30, 20);
    }
    if g.system.regstate >= 2 {
        help(g, b"reset-galactic-coordinator-taxes");
        g.galaxy.w3b0 = ask_number(g, b"What is the galactic tax rate [per 1000 credits]", 0, 100, 15);
    }
    if g.system.regstate >= 2 {
        help(g, b"reset-planet-market");
        g.galaxy.d392 = ask_number(g, b"How many planets are initially available", 0, 25000, 100) as i32;
    }
    help(g, b"reset-planet-limit");
    g.galaxy.d38e = ask_number(g, b"How many planets per turn can be bought", -1, 25000, -1) as i32;
    if g.system.regstate >= 2 {
        help(g, b"reset-planet-regeneration");
        g.galaxy.w38c = ask_number(g, b"How many seconds", 1, 32000, 60);
    }
    help(g, b"reset-food-market");
    g.galaxy.d388 = 1000 * ask_number(g, b"How many thousands of megatons of food", 0, 1000, 70) as i32;
    for i in 0..=9u8 {
        let mut name = b"NEWS/NEWS_.MSG".to_vec();
        name[9] = b'0' + i;
        host_remove(&name);
    }
    out!(g, NEWS; col(14), "\\   The Galactic Times                                        ", col(7));
    let today = g.time.today;
    put_time(g, NEWS, today);
    out!(g, NEWS; "\n");
    out!(g, NEWS; col(9), "\\___________________________________________________________________________\n\n", cn(g));
    out!(g, NEWS; "The game was reset.\n");
    crate::events::clear_events(g);
    clear_stream_file(g, None);
    crate::messages::clear_messages(g);
    crate::events::clear_covert_reports(g);
    crate::battle::clear_battle_reports(g);
    crate::trading::clear_trades(g);
    for i in 0..MAXPIRATES {
        crate::pirates::pirate_init(g, i);
    }
    if yes_no(g, 1, b"Would you like new pirate names") {
        for i in 0..MAXPIRATES {
            crate::pirates::pirate_set_name(g, i, b"");
        }
    }
}

// 6924:052c
pub fn ask_game_dates(g: &mut Sre) {
    kout!(g; "\nEnter the date that users can start playing:  ");
    g.galaxy.t44 = input_date(g);
    kout!(g; "\nEnter the last date that users can play:      ");
    g.galaxy.enddate = input_date(g);
}

// 6924:0aab
fn system_setup(g: &mut Sre, sysop: &[u8], bbs: &[u8]) {
    system_init(g);
    system_set_sysop(g, sysop);
    system_set_bbs(g, bbs);
    crate::register::check_registration(g);
}

// 6924:0993: first-time set-up (sre reset with no game).
pub fn reset_game(g: &mut Sre) {
    host_mkdir(b"DATA");
    host_mkdir(b"BACKUP");
    host_mkdir(b"NEWS");
    system_init(g);
    kout!(g; col(10), "\nSysop: ");
    set_maxlen(g, KIN, 0x40);
    let sysop = in_line(g, KIN);
    set_maxlen(g, KIN, 0);
    kout!(g; col(10), "BBS:   ");
    set_maxlen(g, KIN, 0x40);
    let bbs = in_line(g, KIN);
    set_maxlen(g, KIN, 0);
    system_setup(g, &sysop, &bbs);
}
