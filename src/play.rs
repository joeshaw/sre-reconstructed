// Time limits (segment 1cf7) and the turn sequence (segment 1e7b).

use crate::prelude::*;

static R_STRICT: GRes = GRes::num(b"StrictTime", 0); // a712
static R_SAVEOFTEN: GRes = GRes::num(b"System.SaveOften", 0); // b71e
pub static R_LOTTERY: GRes = GRes::num(b"Lottery", 1); // b72a

// Stand-in for the original's start-up of its time functions.
pub fn time_start(_g: &mut Sre) {
    let _ = &R_STRICT;
}

// 1cf7:0cfa: minutes left in the door.
pub fn minutes_left(g: &Sre) -> i32 {
    let now = b_time();
    if g.door.minutes >= 1000 || g.door.minutes < 0 {
        return 999;
    }
    g.door.minutes as i32 - now.wrapping_sub(g.m.start) / 60
}

// 1cf7:0d69
fn time_is_up(g: &Sre) -> bool {
    let now = b_time();
    let d = &g.door;
    if d.minutes >= 1000 {
        return false;
    }
    now.wrapping_sub(g.m.start) > 60 * d.minutes as i32 && d.minutes > -1
}

// 1cf7:179e: planets regenerate while the player thinks.
fn idle_planets(g: &mut Sre) {
    let last = match g.m.idle_last {
        Some(l) => l,
        None => {
            let t = b_time(); // DS:d2e4
            g.m.idle_last = Some(t);
            t
        }
    };
    let now = b_time();
    if now.wrapping_sub(last) > 1 {
        g.m.idle_last = Some(now);
        if g.open != 0 {
            crate::maintenance::regenerate_planets(g);
        }
    }
}

// 1cf7:15b8: the keyboard idle function (the sysop's status bars are
// not reconstructed).
pub fn idle_hook(g: &mut Sre) {
    idle_planets(g);
}

// 1e7b:000f
fn time_up_check(g: &mut Sre) {
    if !time_is_up(g) {
        return;
    }
    kout!(g; b'\n');
    show_screen(g, None, b"TimeUp", false);
    kout!(g; col(14), "Sorry, your time is up.  Please play again tomorrow!\n");
    g.exit(1);
}

// 1e7b:008d
pub fn time_check(g: &mut Sre) {
    time_up_check(g);
    if minutes_left(g) <= 1 && g.m.warned == 0 {
        kout!(g; col(14), "Warning:  ", cn(g), "You have less than 1 minute remaining in this door!\n");
        g.m.warned = 1;
    }
}

// 1e7b:00fa
fn collapsed(g: &mut Sre, e: usize) {
    if total_planets(g, e) <= 0 && !show_screen(g, None, b"Collapse", false) {
        kout!(g; "Your empire has collapsed because you have no planets left.\n");
    }
    if g.emp[e].population <= 0 && !show_screen(g, None, b"Abandone", false) {
        kout!(g; "Your people have abandoned you, and your empire has collapsed.\n");
    }
}

fn alive(g: &Sre, e: usize) -> bool {
    g.emp[e].exists() && total_planets(g, e) > 0 && g.emp[e].population > 0
}

fn key(g: &mut Sre, c: u8) {
    kout!(g; cbr(g), b'[', cbo(g), c, cbr(g), b']', cn(g));
}

// The start of a turn's phases (inlined in 1e7b:0731 and 1e7b:0f61).
fn start_turn(g: &mut Sre, e: usize) {
    let x = &mut g.emp[e];
    x.phase = 0;
    x.m1bc.set(0);
    x.b1c4 &= !1;
    x.b1c4 &= !2;
    x.b1c4 &= !4;
    x.b1c4 &= !8;
    clear_spending(&mut x.u_1c7);
    x.b1c6 = 0;
    x.b1c5 = 0;
}

// Phase 14: attacks, trades and messages.
fn action_phase(g: &mut Sre, p: &mut Player, e: usize) {
    let cantrade = g.grb(&crate::trading::R_TRADING) && g.emp[e].year >= 0;
    let mut canattack = true;
    let canmessage = g.grb(&crate::messages::R_MESSAGES);
    loop {
        let n = cantrade as i16 + canattack as i16 + canmessage as i16;
        kout!(g; "\nDo you wish to");
        let mut c = b'N';
        if n == 1 {
            if canattack {
                c = if yes_no(g, 0, b" send an attack") { b'A' } else { b'N' };
            }
            if cantrade {
                c = if yes_no(g, 0, b" trade") { b'T' } else { b'N' };
            }
            if canmessage {
                c = if yes_no(g, 0, b" send a message") { b'M' } else { b'N' };
            }
        } else {
            if canattack {
                kout!(g; b' ');
                key(g, b'A');
                kout!(g; "ttack,");
            }
            if cantrade {
                kout!(g; b' ');
                key(g, b'T');
                kout!(g; "rade,");
            }
            if canmessage {
                kout!(g; " send a ");
                key(g, b'M');
                kout!(g; "essage,");
            }
            kout!(g; " or ");
            key(g, b'N');
            kout!(g; "one? ");
            loop {
                c = b_toupper(in_char(g, KIN));
                if (b'1'..=b'9').contains(&c) && canattack {
                    break;
                }
                if c == b'A' && canattack {
                    break;
                }
                if c == b'T' && cantrade {
                    break;
                }
                if c == b'M' && canmessage {
                    break;
                }
                if c == b'\r' || c == 27 || c == b'0' {
                    c = b'N';
                }
                if c == b'N' {
                    break;
                }
            }
            kout!(g; cin(g));
            match c {
                b'1'..=b'9' | b'A' => kout!(g; "Attack"),
                b'T' => kout!(g; "Trade"),
                b'M' => kout!(g; "Message"),
                b'N' => kout!(g; "None"),
                _ => {}
            }
            kout!(g; cn(g), b'\n');
        }
        if c == b'T' {
            crate::trading::trade(g, p, e);
        } else if c == b'M' {
            let (id, name) = (g.emp[e].id, ename(g, e));
            crate::messages::send_message(g, id, &name);
        } else if c != b'N' {
            canattack = !crate::attack::attack(g, p, e, c);
        }
        if c == b'N' {
            break;
        }
        if !(cantrade || canattack || canmessage) {
            break;
        }
    }
}

// 1e7b:0731: one phase of a turn; false when the player is done.
fn phase(g: &mut Sre, p: &mut Player, ph: i16, e: usize) -> bool {
    match ph {
        0 => start_turn(g, e),
        1 => {
            g.emp[e].w192 = 0;
            build_cmd_ship(g, e);
            bank_interest(g, e);
        }
        2 => {
            crate::trading::trade_arrivals(g, p, e);
            crate::events::process_events(g);
            prev_phase(g, e);
            show_new_reports(g, p, e);
            hline(g, col(8), 0x4b);
        }
        3 => crate::income::turn_report(g, p, e),
        4 => {
            prev_phase(g, e);
            crate::turnevents::advisor(g, p, e);
            pause_prompt(g);
        }
        5 => {
            prev_phase(g, e);
            show_status(g, e);
        }
        6 => {
            prev_phase(g, e);
            crate::upkeep::pay_upkeep(g, p, e);
        }
        7 => {
            prev_phase(g, e);
            crate::foodmarket::food_market(g, p, e);
        }
        8 => {
            g.emp[e].d22f[0] = 0;
            g.emp[e].d22f[1] = 0;
            g.emp[e].d22f[2] = 0;
        }
        9 => {
            prev_phase(g, e);
            crate::covert::covert_ops(g, p, e);
        }
        10 => {
            prev_phase(g, e);
            if g.system.regstate != 0 {
                crate::bank::bank(g, p, e);
            }
        }
        11 => {
            prev_phase(g, e);
            crate::spending::spending(g, p, e);
        }
        12 => crate::doubleagents::double_agents(g, p, e),
        13 => {}
        14 => action_phase(g, p, e),
        15 => {}
        16 => crate::population::end_of_turn(g, p, e),
        17 => crate::population::population(g, p, e),
        18 => {
            update_score(g, e);
            update_d1b0(g, e);
        }
        19 => {
            crate::civilwar::civil_war(g, p, e);
            crate::civilwar::desertion(g, p, e);
        }
        20 => planets_move_toward(g, e),
        21 => {
            let mut done = g.emp[e].turnsused >= g.emp[e].turnsperday && g.m.local == 0;
            if done {
                pause_prompt(g);
            }
            if !done && !yes_no(g, 1, b"Continue") {
                done = true;
            }
            if done {
                score_list_all(g);
                show_masters(g);
                pause_prompt(g);
                return false;
            }
        }
        _ => kout!(g; "Not yet installed\n"),
    }
    true
}

// 1e7b:0f61: the first turn of a day.
fn first_turn(g: &mut Sre, p: &mut Player, e: usize) {
    if g.grb(&crate::diplomacy::R_DIPLOMACY) {
        crate::diplomacy::treaty_messages(g, p, e);
        crate::diplomacy::diplomacy_menu(g, p, e);
    }
    let bonus = protection_bonus(g, e);
    start_turn(g, e);
    next_phase(g, e);
    if bonus > 0 {
        kout!(g; "You receive ", cnum(g), bonus, cn(g), " credits from the Galactic Coordinator!\n");
    }
    if g.grb(&R_LOTTERY) {
        crate::lottery::lottery(g, p, e);
    }
}

// 1e7b:0197
pub fn play_game(g: &mut Sre, p: &mut Player) {
    if !galaxy_playable(g) {
        return;
    }
    galaxy_open(g);
    if g.open == 0 {
        return;
    }
    let id = empire_of_player(g, p.index);
    if id == -1 {
        kout!(g; "You do not have an empire.\n");
        return;
    }
    let e = id as usize;
    if !alive(g, e) {
        collapsed(g, e);
        galaxy_close(g);
        return;
    }
    g.current = id;
    g.emp[e].flags |= g.galaxy.w386;
    g.m.local = g.emp[e].flags;
    {
        let dn = cs(&g.door.name);
        let name = ename(g, e);
        let c = (g.emp[e].id as u8).wrapping_add(b'A');
        status_bar(g, &dn, &name, c);
    }
    if g.emp[e].flags != 0 && g.door.port != 0 {
        kout!(g; "This game is configured to be a local-only game.  Remote players are not allowed to play.\n");
        galaxy_close(g);
        return;
    }
    let today = "\nYou are today's ";
    let mut n: i16 = 0;
    if g.emp[e].id == g.galaxy.solarmaster {
        n += 1;
        kout!(g; today, col(14), "Solar Master", cn(g), b'!');
    }
    if g.emp[e].id == g.galaxy.planetmaster {
        n += 1;
        kout!(g; today, col(11), "Planet Master", cn(g), b'!');
    }
    if g.emp[e].id == g.galaxy.weaponsmaster {
        n += 1;
        kout!(g; today, col(13), "Weapons Master", cn(g), b'!');
    }
    if n != 0 {
        kout!(g; b'\n');
    }
    if n == 3 {
        kout!(g; "The Galactic Coordinator extends Its ", col(14), "Personal ");
    }
    if n == 2 {
        kout!(g; "The Galactic Coordinator sends the message:  ", col(3));
    }
    if n != 0 {
        kout!(g; "Congratulations", cn(g), "!\n");
    }
    crate::events::show_covert_reports(g, p, e);
    crate::battle::show_battle_reports(g, p, e);
    if g.emp[e].phase == 0 && g.emp[e].turnsused == 0 {
        first_turn(g, p, e);
    }
    g.emp[e].lastplay = g.time.now;
    p.lastplay = g.time.now;
    g.emp[e].w2a1 = g.grl(&R_INACTIVETIMEOUT) as i16;
    if g.grb(&crate::messages::R_MESSAGES) {
        crate::messages::read_messages(g, p, e);
    }
    if g.emp[e].turnsused >= g.emp[e].turnsperday && g.m.local == 0 {
        kout!(g; "You have used all of your turns for today.\n\n");
        if g.emp[e].year < 0 && g.galaxy.grantdays > 1 {
            kout!(g; "(You receive ", g.galaxy.grantturns, " turns every ", g.galaxy.grantdays, " days.\n");
        }
        return;
    }
    crate::diplomacy::treaty_messages(g, p, e);
    if g.emp[e].phase > 1 {
        kout!(g; "Continuing previously interrupted turn...\n");
    }
    while g.emp[e].turnsused < g.emp[e].turnsperday || g.m.local != 0 {
        let ph = g.emp[e].phase;
        next_phase(g, e);
        let r = phase(g, p, ph, e);
        if g.emp[e].phase == ph {
            next_phase(g, e);
        }
        if g.grb(&R_SAVEOFTEN) {
            system_save_all(g);
        }
        if !alive(g, e) {
            collapsed(g, e);
            break;
        }
        if !r {
            break;
        }
        time_check(g);
    }
    let dn = cs(&g.door.name);
    let pn = cs(&p.name);
    status_bar(g, &dn, &pn, b' ');
    galaxy_close(g);
}
