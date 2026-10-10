// The sysop's empire viewer, player editor and pirate editor (segment
// 473a).  The empire and player lists are drawn on the local screen.

use crate::pirateraid::choose_pirates;
use crate::pirates::*;
use crate::player::*;
use crate::prelude::*;

// 473a:0000: one line of the empire list.
#[allow(clippy::too_many_arguments)]
fn empire_line_text(g: &mut Sre, row: i16, letter: u8, name: Str, planets: i32, score: i32, networth: i32, attr: i16) {
    win_at(g, 1, row);
    out!(g, KWIN; col2(2, 0), b'<', col2(15, 2), letter, col2(2, 0), b'>');
    set_color(g, KWIN, attr >> 4, attr & 0xf);
    out!(g, KWIN; b' ', setw(0x1e), name, setw(0), b' ');
    out!(g, KWIN; col(11), setw(10), planets, setw(0));
    out!(g, KWIN; col(14), setw(11), score, setw(0));
    out!(g, KWIN; col(13), setw(13), networth, setw(0));
    out!(g, KWIN; col2(7, 0));
    win_at(g, 2, row);
}

fn empire_line(g: &mut Sre, row: i16, id: i16, attr: i16) {
    let e = id as usize;
    let (name, tp, sc, nw) = (ename(g, e), total_planets(g, e), g.emp[e].score, net_worth(g, e));
    empire_line_text(g, row, b'A' + id as u8, name, tp, sc, nw, attr);
}

// 473a:01f8: pick an empire from a list on the local screen; its id, or
// -1.  (The galaxy is left open.)
fn choose_empire_local(g: &mut Sre) -> i16 {
    let highlight: i16 = (g.scr.sbcolor << 4) | 0xf;
    let normal: i16 = 7;
    let mut n: i16 = 0;
    let mut ids = [0i16; MAXEMPIRES];
    galaxy_open(g);
    full_window(g);
    set_color(g, KWIN, 0, 7);
    win_clear(g);
    for i in 0..MAXEMPIRES {
        if g.emp[i].name[0] == 0 {
            continue;
        }
        ids[n as usize] = i as i16;
        n += 1;
        let attr = if n == g.m.chosen { highlight } else { normal };
        empire_line(g, n, i as i16, attr);
    }
    if g.m.chosen > n {
        g.m.chosen = n;
    }
    if g.m.chosen < 1 {
        g.m.chosen = 1;
    }
    if n == 0 {
        out!(g, KWIN; "Sorry, there are no users here!\nPress a key...");
        bioskey0(g);
        return -1;
    }
    win_at(g, 2, 1);
    let mut c: u8;
    loop {
        let prev = g.m.chosen;
        g.ost(KWIN).raw = 1;
        c = in_char(g, KWININ);
        if c == 0 {
            c = in_char(g, KWININ);
            if c == 0x2d {
                // Alt-X
                break;
            }
            let ch = &mut g.m.chosen;
            if c == 0x50 {
                *ch += 1;
            }
            if c == 0x48 {
                *ch -= 1;
            }
            if c == 0x4f {
                *ch = n;
            }
            if c == 0x47 {
                *ch = 1;
            }
            if c == 0x51 {
                *ch += 5;
            }
            if c == 0x49 {
                *ch -= 5;
            }
            if *ch < 1 {
                *ch = 1;
            }
            if *ch > n {
                *ch = n;
            }
            c = 0;
        }
        g.ost(KWIN).raw = 0;
        if c == b'\r' || c == 0x1b {
            break;
        }
        c = b_toupper(c);
        if c.is_ascii_uppercase() {
            for i in 0..n {
                if c as i16 == ids[i as usize] + b'A' as i16 {
                    g.m.chosen = i + 1;
                }
            }
        }
        if prev != g.m.chosen {
            empire_line(g, prev, ids[(prev - 1) as usize], normal);
            let ch = g.m.chosen;
            empire_line(g, ch, ids[(ch - 1) as usize], highlight);
        }
    }
    g.ost(KWIN).raw = 0;
    win_at(g, 1, 1);
    set_color(g, KWIN, 2, 15);
    win_clear_eol(g);
    set_color(g, KWIN, 0, 7);
    win_clear(g);
    if c == b'\r' {
        return ids[(g.m.chosen - 1) as usize];
    }
    -1
}

// 473a:05d4: view and change an empire; false when the sysop is done.
fn view_empire(g: &mut Sre) -> bool {
    let idle = set_idle(g, None);
    let id = choose_empire_local(g);
    if id == -1 {
        return false;
    }
    let e = id as usize;
    if g.emp[e].name[0] == 0 {
        kout!(g; cn(g), "Empire doesn't exist.\n");
        set_idle(g, idle);
        return false;
    }
    let mut key: u8;
    loop {
        kout!(g; "\nStatus of empire:  ", cemp(g));
        put_untranslated(g, KOUT, &ename(g, e));
        kout!(g; cn(g), b'\n');
        kout!(g; "Controlled by player #", cnum(g), g.emp[e].player, cn(g));
        let mut p = Player::default();
        let pl = g.emp[e].player;
        player_load(g, &mut p, pl);
        kout!(g; b'(');
        put_untranslated(g, KOUT, cstr(&p.name));
        kout!(g; ") Username is ", cs(&p.user), b'\n');
        player_save(g, &mut p);
        show_status(g, e);
        kout!(g; "Change ", cbr(g), b'[', cbo(g), b'P', cbr(g), b']', cn(g), "layer, ");
        kout!(g; cbr(g), b'[', cbo(g), b'E', cbr(g), b']', cn(g), "mpire Name, ");
        kout!(g; cbr(g), b'[', cbo(g), b'D', cbr(g), b']', cn(g), "elete Empire, or ");
        kout!(g; cbr(g), b'[', cbo(g), b'Q', cbr(g), b']', cn(g), "uit?  ", cin(g));
        key = get_choice(g, b"QPDE", true);
        if key == b'P' {
            kout!(g; "Change Player", cn(g), "\nNew username:  ");
            set_maxlen(g, KIN, 0x1e);
            let mut buf = in_line(g, KIN);
            set_maxlen(g, KIN, 0);
            b_strupr(&mut buf);
            let who = player_find(g, &buf);
            if who == -1 && !buf.is_empty() {
                kout!(g; "Player record not found.  If you wish to change the handle of a user, use the Player Editor; if you wish to transfer control, the second player must create a player record by Joining the Game but not entering an Empire name; then the player will have a record and you can return here to change the empire's controlling player.\n");
            } else {
                kout!(g; "Change player to #", who, " (", cemp(g), &buf, cn(g), b')');
                if yes_no(g, 0, b"") {
                    let mut q = Player::default();
                    player_load(g, &mut q, who);
                    g.emp[e].player = q.index;
                    player_save(g, &mut q);
                }
            }
            key = 0;
            pause_prompt(g);
        } else if key == b'E' {
            kout!(g; "Change Empire Name", cn(g), "\nNew empire name:  ");
            set_maxlen(g, KIN, 0x1e);
            let mut buf = ename(g, e);
            get_line(g, KIN, &mut buf);
            set_maxlen(g, KIN, 0);
            kout!(g; "Change empire name to ", cemp(g), &buf, cn(g));
            if yes_no(g, 0, b"") {
                out!(g, NEWS; "Empire ", cemp(g));
                put_untranslated(g, NEWS, &ename(g, e));
                out!(g, NEWS; cn(g), " changed name to ", col(14), &buf, cn(g), ".\n");
                empire_set_name(g, e, &buf);
            }
            key = 0;
            pause_prompt(g);
        } else if key == b'D' {
            kout!(g; "Delete Player", cn(g), b'\n');
            if !yes_no(g, 0, b"Delete this player") {
                kout!(g; col(14), "Aborted\n");
                key = 0;
            }
            pause_prompt(g);
        }
        if key == b'D' || key == b'Q' {
            break;
        }
    }
    if key == b'D' {
        kout!(g; col(14), "Deleting player...\n");
        out!(g, NEWS; "Empire ", cemp(g));
        put_untranslated(g, NEWS, &ename(g, e));
        out!(g, NEWS; cn(g), " was deleted by the sysop.\n");
        crate::sysop::delete_empire(g, e);
        empire_init(g, e);
    } else {
        kout!(g; "Quit & Save", cn(g), b'\n');
    }
    galaxy_close(g);
    set_idle(g, idle);
    true
}

// 473a:0e9a
pub fn empire_viewer(g: &mut Sre) {
    g.dev.pagelen = 0;
    while view_empire(g) {}
}

// The player chosen from a list under a heading (the Delete and Edit
// commands of 473a:0eaf); -1 if none.
fn pick_player(g: &mut Sre, heading: &str) -> i16 {
    win_clear(g);
    out!(g, KWIN; col2(14, 1), heading);
    win_clear_eol(g);
    let t = gettextinfo(g);
    set_window(g, t.winleft as i16, t.wintop as i16 + 1, t.winright as i16, t.winbottom as i16);
    let mut l = PickList::new(g, 0x17, 0x7f);
    player_list_names(g, &mut l);
    let k = l.run(g);
    let name: Str = l.item(k).cloned().unwrap_or_default();
    set_color(g, KWIN, 0, 7);
    set_window(g, t.winleft as i16, t.wintop as i16, t.winright as i16, t.winbottom as i16);
    win_clear(g);
    let mut who = player_find(g, &name);
    if who == -1 {
        who = (b_atol(&name) - 1) as i16;
    }
    who
}

// 473a:0eaf
pub fn player_editor(g: &mut Sre) {
    kout!(g; col(14), "Player Editor\n\n");
    loop {
        kout!(g; cbr(g), b'[', cbo(g), b'L', cbr(g), b']', cn(g), "ist players, ");
        kout!(g; cbr(g), b'[', cbo(g), b'E', cbr(g), b']', cn(g), "dit a player, ");
        kout!(g; cbr(g), b'[', cbo(g), b'D', cbr(g), b']', cn(g), "elete a player, or ");
        kout!(g; cbr(g), b'[', cbo(g), b'Q', cbr(g), b']', cn(g), "uit? ");
        let key = get_choice(g, b"QLED", true);
        kout!(g; cin(g), key, cn(g), b'\n');
        if key == b'Q' {
            return;
        }
        if key == b'L' {
            let mut l = PickList::new(g, 0x17, 0x7f);
            player_list_names(g, &mut l);
            win_clear(g);
            l.run(g);
            set_color(g, KWIN, 0, 7);
            win_clear(g);
        } else if key == b'D' {
            let who = pick_player(g, " Delete Player");
            if who == -1 {
                kout!(g; "Player not found!\n");
                continue;
            }
            galaxy_open(g);
            let mut i = 0;
            while i < MAXEMPIRES {
                if g.emp[i].name[0] != 0 && g.emp[i].player == who {
                    kout!(g; "This player is currently controlling empire ", b'A' + i as u8,
                          ".  Please delete the empire first if you wish to delete this player.\n");
                    break;
                }
                i += 1;
            }
            galaxy_close(g);
            if i < MAXEMPIRES {
                continue;
            }
            let mut p = Player::default();
            player_load(g, &mut p, who);
            p.name[0] = 0;
            player_save(g, &mut p);
        } else if key == b'E' {
            let who = pick_player(g, " Edit Player");
            if who == -1 {
                kout!(g; "Player not found!\n");
                continue;
            }
            let mut p = Player::default();
            player_load(g, &mut p, who);
            kout!(g; cn(g), "Player #", who, "\n");
            kout!(g; "User:  ", cs(&p.user), b'\n');
            kout!(g; "Name:  ", cs(&p.name), b'\n');
            if yes_no(g, 0, b"Change the username") {
                kout!(g; "To?  ");
                set_maxlen(g, KIN, 0x50);
                let mut buf = cs(&p.user);
                get_line(g, KIN, &mut buf);
                if !buf.is_empty() {
                    if player_find(g, &buf) != -1 {
                        kout!(g; "There is already another player with that username.\n");
                    } else {
                        player_set_user(&mut p, &buf);
                    }
                } else {
                    kout!(g; "(not changed)\n");
                }
            }
            if yes_no(g, 0, b"Change the player name") {
                kout!(g; "To?  ");
                set_maxlen(g, KIN, 0x50);
                let mut buf = cs(&p.name);
                get_line(g, KIN, &mut buf);
                if !buf.is_empty() {
                    player_set_name(&mut p, &buf);
                } else {
                    kout!(g; "(not changed)\n");
                }
            }
            player_save(g, &mut p);
        }
    }
}

// 473a:16eb
pub fn pirate_editor(g: &mut Sre) {
    kout!(g; col(14), "\nEnemy Name Editor", cn(g), b'\n');
    galaxy_open(g);
    create_pirates(g);
    loop {
        let team = choose_pirates(g);
        if team == -1 {
            break;
        }
        let pt = (team - 1) as usize;
        kout!(g; cn(g), "Change the name to:  ");
        let mut name = pirate_name(g, pt);
        set_maxlen(g, KIN, 0x20);
        get_line(g, KIN, &mut name);
        set_maxlen(g, KIN, 0);
        kout!(g; cn(g), "Change the color number to:  ");
        let mut color = pirate_color(g, pt) as i16;
        set_maxval(g, KIN, 0xf);
        set_defval(g, KIN, color as i32);
        color = in_i16(g, KIN);
        if color <= 0 {
            color = 8;
        }
        kout!(g; cn(g), "The pirate team will be called ", col(color));
        if !name.is_empty() {
            kout!(g; &name);
        } else {
            let n = pirate_name(g, pt);
            kout!(g; n);
        }
        kout!(g; cn(g));
        if yes_no(g, 1, b".  Is this OK") {
            if !name.is_empty() {
                pirate_set_name(g, pt, &name);
            }
            g.pir[pt].color = color as u8;
        }
    }
    galaxy_close(g);
}
