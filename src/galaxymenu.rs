// The galaxy menu and joining a game (segment 62b1), and the comparison
// of empire names (segment 7458).

use crate::player::*;
use crate::prelude::*;

// ---------------------------------------------------------------------
// 7458

// 7458:0000: trim spaces and convert to upper case.
fn name_trim(s: &mut Str) -> i16 {
    while !s.is_empty() && ctype_signed(at(s, 0)) & CT_SP != 0 {
        erase(s, 0, 1);
    }
    while !s.is_empty() && ctype_signed(at(s, s.len() as i16 - 1)) & CT_SP != 0 {
        let n = s.len() as i16 - 1;
        erase(s, n, 1);
    }
    for c in s.iter_mut() {
        *c = b_toupper(*c);
    }
    s.len() as i16
}

// 7458:00c0: 0xff becomes a space; runs of spaces become one.
fn name_spaces(s: &mut Str) {
    loop {
        let i = find(s, b"\xff", 0);
        if i < 0 {
            break;
        }
        s[i as usize] = b' ';
    }
    loop {
        let i = find(s, b"  ", 0);
        if i < 0 {
            break;
        }
        erase(s, i, 1);
    }
}

// 7458:014c: letters and digits only.
fn name_alnum(s: &mut Str) {
    let t: Str = s
        .iter()
        .cloned()
        .filter(|&c| ctype_signed(c) & (CT_DIG | CT_UPP | CT_LOW) != 0 && c < 0xa8)
        .collect();
    *s = t;
}

// 7458:01de
pub fn name_normalize(s: &mut Str, level: i16) {
    if level < 1 {
        return;
    }
    name_trim(s);
    if level < 2 {
        return;
    }
    name_spaces(s);
    if level < 3 {
        return;
    }
    name_alnum(s);
}

// 7458:021f: how alike two names are (0 = different, 2 = the same but
// for spacing, 3 = the same or similar).
pub fn name_similarity(mut a: Str, mut b: Str) -> i16 {
    name_trim(&mut a);
    name_trim(&mut b);
    if str_eq_i2(&b, &a) {
        return 3;
    }
    name_spaces(&mut a);
    name_spaces(&mut b);
    if str_eq_i2(&b, &a) {
        return 2;
    }
    name_alnum(&mut a);
    name_alnum(&mut b);
    let la = a.len() as i16;
    let lb = b.len() as i16;
    let mut diff: i16 = 0;
    if la - lb > 2 || lb - la > 2 {
        return 0;
    }
    let mut i: i16 = 0;
    while i < la && i < lb {
        if b_toupper(at(&a, i)) != b_toupper(at(&b, i)) {
            diff += 1;
        }
        i += 1;
    }
    if (diff as i32) * 10 < la as i32 + lb as i32 {
        return 3;
    }
    0
}

// ---------------------------------------------------------------------

fn key(g: &mut Sre, c: u8) {
    kout!(g; cbr(g), b'[', cbo(g), c, cbr(g), b']', cn(g));
}

// 62b1:05b9: the settings of the game.
fn galaxy_info(g: &mut Sre) {
    kout!(g; cn(g), "\nThis is ", col(15), cs(&g.galaxy.name), cn(g), ".\n");
    kout!(g; "There are ", cnum(g), g.galaxy.grantturns, cn(g), " turns per day.\n");
    if g.galaxy.grantdays > 1 {
        kout!(g; "Each \"day\" (maintenance interval) lasts for ", cnum(g), g.galaxy.grantdays, cn(g), " real days.\n");
    }
    kout!(g; "Each empire receives ", cnum(g), g.galaxy.protection, cn(g), " turns of protection.\n");
    kout!(g; cnum(g), g.galaxy.d392, cn(g), " planets are available.\n");
    kout!(g; cnum(g), g.galaxy.d388, cn(g), " megatons of food are available.\n");
    if g.galaxy.d38e >= 0 {
        kout!(g; cnum(g), g.galaxy.d38e, cn(g), " planets can be bought each turn.\n");
    }
    if g.system.regstate != 0 {
        kout!(g; "The interest rate is ", g.galaxy.interest, "%.\n");
    }
    kout!(g; "This game is set up with ");
    if g.galaxy.w384 != 0 {
        kout!(g; "traditional inflationary economics.\n");
    } else {
        kout!(g; "non-inflationary prices.\n");
    }
    if g.galaxy.enddate < midnight(0x7fffffff) {
        kout!(g; "This is a tournament game that ");
        if g.galaxy.t44 <= g.time.today {
            kout!(g; "started ");
        } else {
            kout!(g; "starts ");
        }
        let t = g.galaxy.t44;
        put_time(g, KOUT, t);
        kout!(g; ", and ends ");
        let t = g.galaxy.enddate;
        put_time(g, KOUT, t);
        kout!(g; ".\n");
    }
}

// 62b1:0964
fn new_news(g: &mut Sre, id: i16) {
    let e = id as usize;
    if !g.emp[e].exists() {
        return;
    }
    kout!(g; "New news since ");
    let t = g.emp[e].d2ec;
    put_time(g, KOUT, t);
    kout!(g; ":\n");
    let mut c = b'9';
    loop {
        let mut name = b"News/NEWS0.MSG".to_vec();
        name[9] = c;
        let since = g.emp[e].d2ec;
        show_news(g, &name, since);
        if c == b'0' {
            break;
        }
        c -= 1;
    }
    g.emp[e].d2ec = b_time();
}

// 62b1:0a82
fn old_news(g: &mut Sre) {
    kout!(g; "See news from how many days ago?  ", cbr(g), b'[', cbo(g), "0=Today, or 1-9", cbr(g), b']', cn(g), "  ");
    loop {
        let c = in_char(g, KIN);
        if c == b'\r' || c == 27 || c == b'Q' || c == b'q' {
            kout!(g; "Quit\n");
            return;
        }
        if (c as i8) < b'0' as i8 || (c as i8) > b'9' as i8 {
            continue;
        }
        kout!(g; c, " day");
        if c != b'1' {
            kout!(g; b's');
        }
        kout!(g; b'\n');
        let mut name = b"News/NEWS_.MSG".to_vec();
        name[9] = c;
        if !show_news(g, &name, 0) {
            kout!(g; "The news file from ", c, " days ago is missing.\n");
        }
        return;
    }
}

fn emperor(male: i16) -> &'static [u8] {
    if male != 0 {
        b"Emperor"
    } else {
        b"Emperess"
    }
}

// 62b1:0c07
fn join_game(g: &mut Sre) {
    if g.time.today >= g.galaxy.t4c {
        kout!(g; "No new players can join this game.\n\n");
        return;
    }
    if !yes_no(g, 0, b"Are you sure you would like to join this game") {
        return;
    }
    let dn = cs(&g.door.name);
    let mut pl = player_find(g, &dn);
    let mut isnew = false;
    if pl == -1 {
        pl = player_free(g);
        if pl == -1 {
            kout!(g; "Sorry, SRE is unable to add you to the system.\n");
            return;
        }
        let mut np = Player::default();
        player_load(g, &mut np, pl);
        player_init_index(g, &mut np, pl);
        status_bar(g, &dn, b"Creating New Player", b'*');
        loop {
            crate::help::show_help_topic(g, b"reset-new-player");
            kout!(g; "Enter the name of your ruler:\n> ");
            set_maxlen(g, KIN, 0x1f);
            let name = in_line(g, KIN);
            set_maxlen(g, KIN, 0);
            if name.is_empty() {
                kout!(g; "Ruler not created.\n");
                player_save(g, &mut np);
                return;
            }
            player_set_name(&mut np, &name);
            player_set_user(&mut np, &dn);
            kout!(g; cn(g), "Are you ");
            key(g, b'F');
            kout!(g; "emale or ");
            key(g, b'M');
            kout!(g; "ale?  ", col(15));
            let mut c;
            loop {
                c = b_toupper(in_char(g, KIN));
                if c == b'M' || c == b'F' {
                    break;
                }
            }
            kout!(g; if c == b'M' { "Male" } else { "Female" }, cn(g), b'\n');
            np.male = if c == b'M' { 1 } else { 0 };
            kout!(g; "You will be:  ", emperor(np.male), cemp(g), b' ', Untr(cs(&np.name)), cn(g), b'\n');
            if yes_no(g, 1, b"Is this correct") {
                break;
            }
        }
        show_screen(g, None, b"NewPlaye", false);
        isnew = true;
        player_save(g, &mut np);
    }
    let mut p = Player::default();
    player_load(g, &mut p, pl);
    if p.name[0] == 0 {
        kout!(g; "There is a problem with SRE:  error #<galaxy:joingame>.\n");
        player_save(g, &mut p);
        return;
    }
    if isnew {
        kout!(g; "Welcome, ", col(14), emperor(p.male), b' ', Untr(cs(&p.name)), cn(g), "!\n\n");
        if yes_no(g, 1, b"Do you wish to view the instructions") {
            display_file(g, b"Text/Player.Msg", 0);
        }
    }
    let mut slot: i16 = -1;
    let mut n: i16 = 0;
    let mut open: i32 = 0;
    galaxy_open(g);
    for i in (0..=24).rev() {
        if !g.emp[i].exists() {
            mask_set(&mut open, i as i16);
            n += 1;
            slot = i as i16;
        }
    }
    if n == 0 {
        if !show_screen(g, None, b"GameFull", false) {
            kout!(g; "Sorry - this game is full.\n");
        }
        player_save(g, &mut p);
        return;
    }
    if n > 1 {
        kout!(g; "There are ", cnum(g), n, cn(g), " open slots for new players:\n    ", cemp(g));
        for i in 0..MAXEMPIRES {
            if mask_test(open, i as i16) {
                kout!(g; i as u8 + b'A', b' ');
            }
        }
        kout!(g; cn(g), "\nWhich one would you like?  [ENTER to pick the first slot]  ", cin(g));
        let mut c;
        loop {
            c = in_char(g, KIN);
            if c == b'\r' {
                break;
            }
            if c == 27 {
                kout!(g; cn(g), "Aborted!", cn(g), b'\n');
                player_save(g, &mut p);
                return;
            }
            c = b_toupper(c);
            if !(b'A'..=b'Y').contains(&c) {
                continue;
            }
            if mask_test(open, (c - b'A') as i16) {
                break;
            }
        }
        if (b'A'..=b'Y').contains(&c) {
            slot = (c - b'A') as i16;
            kout!(g; c);
        } else {
            kout!(g; "[first slot]");
        }
        kout!(g; cn(g), b'\n');
    }
    galaxy_close(g);
    let e = slot as usize;
    if !g.emp[e].exists() {
        empire_init_id(g, e, slot);
        crate::sysop::delete_empire(g, e);
        empire_init_id(g, e, slot);
        status_bar(g, &dn, b"Creating New Empire", b'*');
        let mut name: Str;
        loop {
            kout!(g; "Choose a name for your new empire:\n> ");
            set_maxlen(g, KIN, 0x3f);
            name = in_line(g, KIN);
            set_maxlen(g, KIN, 0);
            if name.is_empty() {
                kout!(g; "An empire was not created.\n");
                player_save(g, &mut p);
                return;
            }
            let mut letters: i16 = 0;
            let mut nonspace: i16 = 0;
            for &ch in name.iter() {
                if ctype_signed(ch) & (CT_DIG | CT_UPP | CT_LOW) != 0 && ch < 0xa8 {
                    letters += 1;
                }
                if ctype_signed(ch) & CT_SP == 0 {
                    nonspace += 1;
                }
            }
            if nonspace / 2 > letters && nonspace > 3 {
                kout!(g; "Please use a name rather than symbols.\n");
                continue;
            }
            if letters < 3 {
                kout!(g; "Please use a longer name.\n");
                continue;
            }
            let mut matched: i16 = -1;
            let mut how: i16 = 0;
            for i in 0..MAXEMPIRES {
                if !g.emp[i].exists() {
                    continue;
                }
                let r = name_similarity(name.clone(), ename(g, i));
                if r > 0 {
                    how = r;
                    matched = i as i16;
                }
            }
            if matched != -1 {
                let m = matched as usize;
                if how == 1 {
                    kout!(g; "That name is already being used by empire ", cemp(g), m as u8 + b'A', cn(g), b'\n');
                }
                if how == 2 {
                    kout!(g; "Empire ", m as u8 + b'A', " is named '", cemp(g), uname(g, m), cn(g),
                          "'.  Please choose another name.\n");
                }
                if how == 3 {
                    kout!(g; "That name is similar to '", cemp(g), uname(g, m), cn(g),
                          "'.  To avoid confusion, please choose a different name.\n");
                }
                continue;
            }
            kout!(g; "Name your empire ", cemp(g), &name, cn(g));
            if yes_no(g, 1, b"") {
                break;
            }
        }
        empire_set_name(g, e, &name);
    }
    g.emp[e].flags = g.galaxy.w386;
    g.emp[e].player = p.index;
    kout!(g; "You are:  ", emperor(p.male), col(14), b' ', Untr(cs(&p.name)), cn(g));
    kout!(g; ", and you now rule ", cemp(g), uname(g, e), cn(g), ".\n");
    player_save(g, &mut p);
}

// 62b1:1bd7
fn tournament_results(g: &mut Sre) {
    kout!(g; col(14), "\nTournament Results\n");
    hline(g, col(4), 0x4b);
    kout!(g; cn(g), "Tournament started:  ", col(12));
    let t = midnight(g.galaxy.t44);
    put_time(g, KOUT, t);
    kout!(g; b'\n');
    kout!(g; cn(g), "Tournament ended:    ", col(12));
    let t = midnight(g.galaxy.enddate);
    put_time(g, KOUT, t);
    kout!(g; b'\n');
    kout!(g; cn(g));
    let mut idx = [0i16; 25];
    let mut pts = [0i32; 26];
    for i in 0..MAXEMPIRES {
        idx[i] = i as i16;
        pts[i] = 0;
        if !g.emp[i].exists() {
            continue;
        }
        if g.emp[i].exists() && total_planets(g, i) > 0 && g.emp[i].population > 0 {
            pts[i] = g.emp[i].d178;
        }
    }
    pts[25] = 0;
    for i in 0..24 {
        for j in i + 1..25 {
            if pts[i] < pts[j] {
                pts.swap(i, j);
                idx.swap(i, j);
            }
        }
    }
    let mut last: i16 = -1;
    for i in 0..MAXEMPIRES {
        if pts[i] > 0 {
            last = i as i16;
        }
    }
    if last == -1 {
        kout!(g; "There were no winners.\n");
    } else {
        kout!(g; "\nEmpires are listed by rank:\n");
        kout!(g; " #      Empire name                          Points\n");
        let mut rank: i16 = 0;
        for i in 0..=last as usize {
            kout!(g; setw(2), col(14), rank + 1, setw(0), col(6), ":  ");
            kout!(g; col(12), idx[i] as u8 + b'A', "  ");
            kout!(g; setw(0x23), col(15), uname(g, idx[i] as usize));
            kout!(g; setw(8), col(10), pts[i], setw(0), b'\n');
            if pts[i + 1] < pts[rank as usize] {
                rank = i as i16 + 1;
            }
        }
    }
    hline(g, col(4), 0x4b);
}

// 62b1:0000
pub fn galaxy_menu(g: &mut Sre) {
    let mut def: i16 = 6;
    crate::pirates::create_pirates(g);
    loop {
        crate::play::time_check(g);
        let dn = cs(&g.door.name);
        let gn = cs(&g.galaxy.name);
        status_bar(g, &dn, &gn, b'*');
        let pl = player_find(g, &dn);
        let id = empire_of_player(g, pl);
        if id != -1 {
            g.current = id;
        }
        let alive = |g: &Sre| {
            let e = id as usize;
            g.emp[e].exists() && total_planets(g, e) > 0 && g.emp[e].population > 0
        };
        let mut m = Menu::new(g, b"Galaxy", b"Galaxy Menu", [0xe, 4, 0xc, 0xf, 7, 0xd, 0xe, 7, 7, 7]);
        m.add(g, b"Status", b'1', 1);
        if id != -1 {
            m.add(g, b"New News", b'2', 2);
        }
        m.add(g, b"Daily News", b'3', 3);
        if g.time.today > g.galaxy.enddate {
            m.add(g, b"See Tournament Results", b'4', 11);
        } else if id == -1 {
            m.add(g, b"Join this Game", b'4', 10);
            if def == 6 {
                def = 8;
            }
        } else {
            m.add(g, b"Play SRE!", b'4', 4);
            let e = id as usize;
            if g.emp[e].turnsused < g.emp[e].turnsperday && alive(g) {
                def = 4;
            } else if def == 4 {
                def = -1;
            }
        }
        if id != -1 {
            m.add(g, b"See Your Status", b'5', 5);
        }
        if crate::messages::r_messages(g) {
            m.add(g, b"Read Messages", b'6', 6);
            m.add(g, b"Send Messages", b'7', 7);
        } else if def == 6 {
            def = -1;
        }
        m.add(g, b"Scores", b'8', 8);
        if id != -1 && alive(g) && g.time.today <= g.galaxy.enddate && crate::diplomacy::r_diplomacy(g) {
            m.add(g, b"Diplomacy", b'9', 9);
        }
        let r = m.run(g, true, def);
        match r {
            1 => galaxy_info(g),
            2 => new_news(g, id),
            3 => old_news(g),
            4 => {
                let mut p = Player::default();
                player_load(g, &mut p, pl);
                crate::play::play_game(g, &mut p);
                player_save(g, &mut p);
            }
            5 => {
                if g.emp[id as usize].exists() {
                    show_status(g, id as usize);
                }
            }
            6 => {
                if pl != -1 && id != -1 {
                    let mut p = Player::default();
                    player_load(g, &mut p, pl);
                    crate::messages::read_messages(g, &mut p, id as usize);
                    player_save(g, &mut p);
                } else {
                    crate::messages::read_public_messages(g);
                }
                if def == 6 {
                    def = -1;
                }
            }
            7 => {
                if pl != -1 && id != -1 {
                    let n = ename(g, id as usize);
                    crate::messages::send_message(g, id, &n);
                } else {
                    let n = cat(&[b"Visitor: ", &dn]);
                    crate::messages::send_message(g, -1, &n);
                }
            }
            8 => {
                score_list_all(g);
                show_masters(g);
                if def == 8 {
                    def = -1;
                }
            }
            9 => {
                let mut p = Player::default();
                player_load(g, &mut p, pl);
                crate::diplomacy::treaty_messages(g, &mut p, id as usize);
                crate::diplomacy::diplomacy_menu(g, &mut p, id as usize);
                player_save(g, &mut p);
            }
            10 => join_game(g),
            11 => tournament_results(g),
            _ => {}
        }
        if r == -1 {
            break;
        }
    }
}
