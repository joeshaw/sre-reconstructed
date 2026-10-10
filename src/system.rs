// The game "system" (segment 6c38): data files, main menu, rankings.

use crate::prelude::*;

const SYSTEM_FILE: &[u8] = b"Data/System.II"; // DS:a1a8
const GALAXY_FILE: &[u8] = b"Data/Galaxy.II"; // DS:a1ac
const EMPIRE_FILE: &[u8] = b"Data/Empire.II"; // DS:a1b0
const PIRATE_FILE: &[u8] = b"Data/Pirate.II"; // DS:a1b4

// 6c38:0000
pub fn system_load(g: &mut Sre) {
    g.system.t120 = b_time();
    g.system.t124 = b_time();
    g.system.galaxy.clear();
    let mut f = DataFile::new(g, SYSTEM_FILE, 0x142, 0, 1);
    let mut b = g.system.to_bytes();
    let ok = f.read(0, &mut b, 1);
    g.system = System::from_bytes(&b);
    if !ok {
        system_init(g);
    }
    crate::register::check_registration(g);
}

// 6c38:00f6
pub fn system_clear_in_use(g: &mut Sre) {
    g.system.t124 = 0;
}

// 6c38:011d
pub fn system_init(g: &mut Sre) {
    g.system.t120 = g.time.now;
    g.system.t124 = g.time.now;
    g.system.galaxy.clear();
    for i in 0..0x40 {
        g.system.bbs[i] = 0;
        g.system.sysop[i] = 0;
    }
    g.system.regstate = 0;
}

// 6c38:0188
pub fn system_set_sysop(g: &mut Sre, s: &[u8]) {
    set_name(&mut g.system.sysop, s);
}

// 6c38:01be
pub fn system_set_bbs(g: &mut Sre, s: &[u8]) {
    set_name(&mut g.system.bbs, s);
}

// 6c38:01f8
pub fn system_show_info(g: &mut Sre) {
    kout!(g; col(14), "\nSystem Information\n", cn(g));
    kout!(g; "BBS:   ", cnum(g), cs(&g.system.bbs), cn(g), b'\n');
    kout!(g; "Sysop: ", cnum(g), cs(&g.system.sysop), cn(g), b'\n');
}

// 6c38:02b8
pub fn system_maintenance(g: &mut Sre) {
    crate::maintenance::maintenance(g);
}

// 6c38:02c9
pub fn system_valid(g: &Sre) -> bool {
    let s = &g.system;
    if !(s.t124 > 0x12c684c0 && s.t124 < 0x38538e40) {
        return false;
    }
    if !(s.t120 > 0x12c684c0 && s.t120 < 0x38538e40) {
        return false;
    }
    if s.regstate < 0 || s.regstate > 5 {
        return false;
    }
    true
}

// Read a record of n bytes (n may differ from the structure's size) over
// the structure's bytes.
fn read_over(f: &mut DataFile, rec: i32, cur: Vec<u8>, n: usize) -> (bool, Vec<u8>) {
    let mut b = cur;
    if b.len() < n {
        b.resize(n, 0);
    }
    let ok = f.read(rec, &mut b, 1);
    (ok, b)
}

// 6c38:0349: read Galaxy.II, Empire.II and Pirate.II.
pub fn system_load_data(g: &mut Sre) -> i16 {
    let mut n: i32 = {
        let mut f = DataFile::new(g, GALAXY_FILE, 1, 0, 1);
        f.numrecs()
    };
    let mut gf = DataFile::new(g, GALAXY_FILE, n as i16, 0, 1);
    galaxy_init(g);
    let nn = n as i16 as u16 as usize;
    let (ok, mut b) = read_over(&mut gf, 0, g.galaxy.to_bytes(), nn);
    let gc = g.system.galaxy;
    let check = check_checksum(&mut b[..nn], &gc);
    g.galaxy = Galaxy::from_bytes(&b);
    drop(gf);
    if ok && !check && g.galaxy.name[0] != 0 {
        let now = g.time.now;
        out!(g, NEWS; time_str(g, now), ": It appears that the data files for this game have been tampered with.\n");
        galaxy_init(g);
        out!(g, ERRLOG; time_str(g, now), ":  Game data files corrupt!\n");
    }
    n = {
        let mut f = DataFile::new(g, EMPIRE_FILE, 1, 0, 1);
        f.numrecs() / MAXEMPIRES as i32
    };
    if n > 0x2fc {
        g.open = 0;
        let now = g.time.now;
        out!(g, ERRLOG; time_str(g, now),
             ":  FATAL ERROR!  Empire file format has changed!  This version of SRE cannot handle files created by newer versions!\n");
        g.exit(5);
    }
    if n > 0 && n < 0x2fc {
        let now = g.time.now;
        out!(g, ERRLOG; time_str(g, now),
             ":  NOTICE!  Empire file format has changed.  SRE is assuming you have recently installed a newer version.\n");
    }
    if ok && n > 0 {
        let mut f = DataFile::new(g, EMPIRE_FILE, n as i16, 0, 1);
        let nn = n as usize;
        for i in 0..MAXEMPIRES {
            let (_, mut b) = read_over(&mut f, i as i32, g.emp[i].to_bytes(), nn);
            let c = g.galaxy.empire[i];
            let good = check_checksum(&mut b[..nn], &c);
            g.emp[i] = Empire::from_bytes(&b);
            if !good && g.galaxy.name[0] != 0 {
                empire_init(g, i);
                if g.emp[i].exists() && g.galaxy.name[0] != 0 {
                    let now = g.time.now;
                    out!(g, NEWS; time_str(g, now), ":  It appears that the data files for ");
                    put_untranslated(g, NEWS, &ename(g, i));
                    out!(g, NEWS; " have been tampered with.\n");
                    out!(g, ERRLOG; time_str(g, now), ": Resetting EMPIRE ", i as i16, "!\n");
                }
            }
            g.emp[i].flags |= g.galaxy.w386;
        }
    } else {
        for i in 0..MAXEMPIRES {
            empire_init(g, i);
        }
    }
    n = {
        let mut f = DataFile::new(g, PIRATE_FILE, 1, 0, 1);
        f.numrecs() / MAXPIRATES as i32
    };
    if ok && n > 0 && n <= 0x106 {
        let mut f = DataFile::new(g, PIRATE_FILE, n as i16, 0, 1);
        let nn = n as usize;
        for i in 0..MAXPIRATES {
            let (_, mut b) = read_over(&mut f, i as i32, g.pir[i].to_bytes(), nn);
            let c = g.galaxy.pirate[i];
            let good = check_checksum(&mut b[..nn], &c);
            g.pir[i] = Pirate::from_bytes(&b);
            if !good && g.galaxy.name[0] != 0 {
                crate::pirates::pirate_init(g, i);
            }
        }
    } else {
        for i in 0..MAXPIRATES {
            crate::pirates::pirate_init(g, i);
        }
    }
    ok as i16
}

// 6c38:0a34
pub fn system_save(g: &mut Sre) {
    if system_valid(g) {
        let mut f = DataFile::new(g, SYSTEM_FILE, 0x142, 0, 1);
        let b = g.system.to_bytes();
        f.write(g, 0, &b, 1);
    }
}

// 6c38:0a9a: checksum, encrypt and write everything (the structures in
// memory stay as they are).
pub fn system_save_all(g: &mut Sre) {
    let mut eb: Vec<u8> = Vec::with_capacity(0x2fc * MAXEMPIRES);
    for i in 0..MAXEMPIRES {
        let mut b = g.emp[i].to_bytes();
        g.galaxy.empire[i] = make_checksum(&mut b);
        eb.extend_from_slice(&b);
    }
    let mut pb: Vec<u8> = Vec::with_capacity(0x106 * MAXPIRATES);
    for i in 0..MAXPIRATES {
        let mut b = g.pir[i].to_bytes();
        g.galaxy.pirate[i] = make_checksum(&mut b);
        pb.extend_from_slice(&b);
    }
    let mut gb = g.galaxy.to_bytes();
    g.system.galaxy = make_checksum(&mut gb);
    system_save(g);
    let mut gf = DataFile::new(g, GALAXY_FILE, 0x3c6, 0, 1);
    gf.write(g, 0, &gb, 1);
    let mut ef = DataFile::new(g, EMPIRE_FILE, 0x2fc, 0, 1);
    ef.write(g, 0, &eb, MAXEMPIRES as i16);
    let mut pf = DataFile::new(g, PIRATE_FILE, 0x106, 0, 1);
    pf.write(g, 0, &pb, MAXPIRATES as i16);
}

// 6c38:110a
fn menu_galaxy(g: &mut Sre) {
    galaxy_open(g);
    if g.open == 0 {
        kout!(g; "The game has not been reset.  Please inform the sysop.\n");
        return;
    }
    crate::maintenance::maintenance(g);
    galaxy_menu(g);
    galaxy_close(g);
}

// 6c38:1149
fn menu_who_am_i(g: &mut Sre, pl: i16) {
    let mut p = Player::default();
    crate::player::player_load(g, &mut p, pl);
    crate::player::player_who_am_i(g, &mut p);
    crate::player::player_save(g, &mut p);
}

// 6c38:1180
fn menu_settings(g: &mut Sre, pl: i16) {
    let mut p = Player::default();
    crate::player::player_load(g, &mut p, pl);
    crate::operations::preferences(g, &mut p);
    crate::player::player_save(g, &mut p);
}

// 6c38:0e04
pub fn main_menu(g: &mut Sre) {
    let mut def: i16 = 2;
    loop {
        crate::play::time_check(g);
        let dn = cs(&g.door.name);
        let pl = crate::player::player_find(g, &dn);
        status_bar(g, &dn, b"Main Menu", b' ');
        let mut m = Menu::new(g, b"System", b"System", [0xe, 8, 7, 0xf, 7, 8, 8, 7, 7, 7]);
        m.add(g, b"Status", b'1', 1);
        m.add(g, b"Visit the Galaxy", b'2', 2);
        if pl != -1 {
            m.add(g, b"Who am I?", b'3', 3);
        }
        m.add(g, b"Bulletins", b'4', 4);
        m.add(g, b"Rankings", b'8', 8);
        if pl != -1 {
            m.add(g, b"Change your settings", b'P', 13);
        }
        m.add(g, b"More Help", b'H', 15);
        m.add(g, b"Quit", b'Q', 99);
        m.add(g, b"", b'0', 99);
        let mut r = m.run(g, true, def);
        match r {
            1 => system_show_info(g),
            2 => {
                menu_galaxy(g);
                def = 99;
            }
            3 => menu_who_am_i(g, pl),
            4 => crate::help::bulletins(g),
            8 => rankings(g),
            13 => menu_settings(g, pl),
            15 => crate::help::more_help(g),
            99 => r = -1,
            _ => {}
        }
        if r == -1 {
            break;
        }
    }
    show_screen(g, None, b"Return", false);
}

// 6c38:11b7: order idx[] by val[idx[]] descending.
fn sort_ranks(idx: &mut [i16], val: &[i32], n: i16) {
    for i in 0..(n - 1).max(0) as usize {
        for j in i + 1..n as usize {
            if val[idx[i] as usize] < val[idx[j] as usize] {
                idx.swap(i, j);
            }
        }
    }
}

// 6c38:126b
fn rank_menu(g: &mut Sre, idx: &[i16], emp: &[i16], val: &[i32], n: i16) {
    let mut m = Menu::new(g, b"Rankings", b"", [0xf, 8, 0, 0xa, 7, 3, 0xb, 7, 7, 7]);
    for i in 0..n as usize {
        let e = emp[idx[i] as usize] as usize;
        let name = ename(g, e);
        let t = cat(&[b"#", &int_str(i as i32 + 1)]);
        m.add(g, &t, (e as u8).wrapping_add(b'A'), i as i16 + 1);
        let v = val[idx[i] as usize];
        let it = m.find(i as i16 + 1).unwrap();
        it.add_column_str(&name);
        it.add_column_num(v);
    }
    m.show(g);
}

// 6c38:1486
pub fn rankings(g: &mut Sre) {
    let mut idx = [0i16; 25];
    let mut emp = [0i16; 25];
    let mut networth = [0i32; 25];
    let mut planets = [0i32; 25];
    let mut score = [0i32; 25];
    let mut points = [0i32; 25];
    galaxy_open(g);
    let mut n: i16 = 0;
    for i in 0..MAXEMPIRES {
        if !g.emp[i].exists() {
            continue;
        }
        let k = n as usize;
        idx[k] = n;
        emp[k] = i as i16;
        networth[k] = net_worth(g, i);
        planets[k] = total_planets(g, i);
        score[k] = g.emp[i].score;
        points[k] = g.emp[i].d178;
        n += 1;
    }
    if n == 0 {
        kout!(g; "There are no empires in this galaxy.\n");
    } else {
        sort_ranks(&mut idx, &planets, n);
        kout!(g; col(14), "\nEmpires Sorted by Planets");
        rank_menu(g, &idx, &emp, &planets, n);
        pause_prompt(g);
        sort_ranks(&mut idx, &networth, n);
        kout!(g; col(14), "\nEmpires Sorted by Net Worth");
        rank_menu(g, &idx, &emp, &networth, n);
        pause_prompt(g);
        sort_ranks(&mut idx, &score, n);
        kout!(g; col(14), "\nEmpires Sorted by Score");
        rank_menu(g, &idx, &emp, &score, n);
        pause_prompt(g);
        if g.galaxy.enddate < never_date() {
            sort_ranks(&mut idx, &points, n);
            kout!(g; col(14), "\nEmpires Sorted by Tournament Points");
            rank_menu(g, &idx, &emp, &points, n);
            pause_prompt(g);
        }
    }
    galaxy_close(g);
}
