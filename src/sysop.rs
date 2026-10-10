// The sysop menu (segment 6bd1) and deleting empires (segment 6924).

use crate::prelude::*;
use crate::reset::*;

// 6bd1:0389: name and reset the galaxy; true if it was reset.
fn reset_the_game(g: &mut Sre) -> bool {
    let mut name: Str;
    loop {
        kout!(g; "Enter the name of your galaxy:\n> ");
        set_maxlen(g, KIN, 0x3f);
        name = in_line(g, KIN);
        set_maxlen(g, KIN, 0);
        if name.is_empty() {
            kout!(g; "The galaxy was not created.\n");
            return false;
        }
        kout!(g; "Name your galaxy ", cemp(g), &name, cn(g));
        if yes_no(g, 1, b"") {
            break;
        }
    }
    if !yes_no(g, 0, b"Are you sure you want to reset the game") {
        return false;
    }
    galaxy_reset(g);
    galaxy_open(g);
    galaxy_set_name(g, &name);
    reset_galaxy(g);
    galaxy_close(g);
    true
}

// 6bd1:048d
fn tournament(g: &mut Sre) {
    kout!(g; "Tournament setup\n");
    if reset_the_game(g) {
        galaxy_open(g);
        ask_game_dates(g);
        galaxy_close(g);
    }
}

// 6bd1:04c7
fn change_names(g: &mut Sre) {
    kout!(g; "Change names\n");
    kout!(g; col(10), "Sysop: ");
    set_maxlen(g, KIN, 0x40);
    let sysop = in_line(g, KIN);
    set_maxlen(g, KIN, 0);
    if !sysop.is_empty() {
        system_set_sysop(g, &sysop);
    }
    kout!(g; col(10), "BBS:   ");
    set_maxlen(g, KIN, 0x40);
    let bbs = in_line(g, KIN);
    set_maxlen(g, KIN, 0);
    if !bbs.is_empty() {
        system_set_bbs(g, &bbs);
    }
    kout!(g; "Enter the name of your galaxy:\n> ");
    set_maxlen(g, KIN, 0x3f);
    let name = in_line(g, KIN);
    set_maxlen(g, KIN, 0);
    if !name.is_empty() {
        galaxy_open(g);
        galaxy_set_name(g, &name);
        reset_galaxy(g);
        galaxy_close(g);
    } else {
        kout!(g; "The name was not changed.\n");
    }
}

// 6bd1:0000
pub fn sysop_menu(g: &mut Sre) {
    loop {
        let mut m = Menu::new(g, b"Sysop", b"Sysop", [0xe, 8, 7, 0xf, 7, 8, 8, 7, 7, 7]);
        m.add(g, b"Status", b'1', 1);
        m.add(g, b"Reset the Game", b'2', 2);
        m.add(g, b"Reset and set up a Tournament", b'3', 3);
        m.add(g, b"Change System Names", b'4', 4);
        m.add(g, b"Read Messages", b'6', 6);
        m.add(g, b"Send a Message", b'7', 7);
        m.add(g, b"Empire Viewer", b'E', 10);
        m.add(g, b"Player Editor", b'P', 11);
        m.add(g, b"Pirate Editor", b'T', 12);
        if g.system.regstate == 0 {
            m.add(g, b"What happens if I register", b'$', 13);
        }
        let r = m.run(g, true, -1);
        match r {
            1 => system_show_info(g),
            2 => {
                reset_the_game(g);
            }
            3 => tournament(g),
            4 => change_names(g),
            6 => {
                galaxy_open(g);
                crate::messages::sysop_messages(g);
                galaxy_close(g);
            }
            7 => {
                galaxy_open(g);
                crate::messages::send_message(g, -1, b"");
                galaxy_close(g);
            }
            10 => crate::editors::empire_viewer(g),
            11 => crate::editors::player_editor(g),
            12 => crate::editors::pirate_editor(g),
            13 => {
                crate::help::show_help_topic(g, b"info-register");
                kout!(g; cn(g), "It will show up as:\n\nThis game is registered to ", col(14), "(your real name here)");
                kout!(g; cn(g), "\n\nYour bank name will show up as:\n\nWelcome to ", col(14), "(your bank name here)");
                kout!(g; cn(g), "\n\nWe only register to your real name, so if you do not wish your real name to appear to your users, please do not register.\n");
            }
            _ => {}
        }
        if r == -1 {
            break;
        }
    }
}

// 6924:057d: remove an empire from the game.
pub fn delete_empire(g: &mut Sre, e: usize) {
    let id = g.emp[e].id;
    if id == -1 {
        return;
    }
    for i in 0..MAXEMPIRES {
        let o = &mut g.emp[i];
        for k in [5, 0, 1, 2, 3, 4, 6] {
            mask_clear(&mut o.d22f[k], id);
        }
        set_treaty(&mut o.treaty[id as usize], 0, 0);
        o.treaty[id as usize].setc(0);
        o.b2d3[id as usize] = 0;
    }
    for j in 0..10 {
        g.emp[e].planets[j].count = 0;
    }
    g.emp[e].population = -1;
    crate::trading::trades_empire_deleted(g, id);
    crate::messages::messages_empire_deleted(g, id);
    reports_empire_deleted(g, id, Some(crate::battle::BATTLE_FILE));
    reports_empire_deleted(g, id, None);
    reports_empire_deleted(g, id, Some(crate::events::COVERT_FILE));
    crate::events::events_empire_deleted(g, id);
}
