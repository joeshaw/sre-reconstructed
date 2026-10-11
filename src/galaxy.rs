// The galaxy (segment 5222).

use crate::prelude::*;

// 5222:0112
pub fn galaxy_init(g: &mut Sre) {
    let today = g.time.today;
    let now = g.time.now;
    let never = never_date();
    let x = &mut g.galaxy;
    x.name[0] = 0;
    x.created = today;
    x.lasthourly = today;
    x.startdate = today;
    x.lastregen = now;
    x.joindeadline = never;
    x.enddate = never;
    x.inflation = 0;
    x.localonly = 0;
    x.interest = 3;
    for c in x.empire.iter_mut() {
        c.clear();
    }
    for c in x.pirate.iter_mut() {
        c.clear();
    }
    x.weaponsmaster = -1;
    x.planetmaster = -1;
    x.solarmaster = -1;
    x.grantturns = 5;
    x.protection = 20;
    x.w3a2 = 0;
    x.lotterytickets = 0;
    x.lotterywinner = -1;
    x.jackpot = 0;
    x.coordfund = 10000;
    x.coordtax = 15;
    x.nukelimit = 300;
    x.nukepercent = 20;
    x.psionicprice = 10000000;
    x.newspaper = 0;
    x.daylength = 1;
    x.marketfood = 7000000;
    x.regenseconds = 3;
    x.marketplanets = 100;
    x.planetsperturn = -1;
    x.oilprice = 0;
}

// 5222:0339
pub fn galaxy_reset(g: &mut Sre) {
    galaxy_init(g);
    for i in 0..MAXEMPIRES {
        empire_init(g, i);
    }
    system_save_all(g);
}

// 5222:037e
pub fn galaxy_set_name(g: &mut Sre, s: &[u8]) {
    set_name(&mut g.galaxy.name, s);
}

// 5222:03b4: the price of petroleum follows supply and demand.
pub fn galaxy_oil_price(g: &mut Sre) {
    if g.open == 0 {
        return;
    }
    let mut planets: i32 = 0;
    let mut petroleum: i32 = 0;
    for i in 0..MAXEMPIRES {
        planets = planets.wrapping_add(total_planets(g, i));
        petroleum = petroleum.wrapping_add(g.emp[i].planets[P_PETROLEUM].count);
    }
    let mut v = (300i32.wrapping_mul(planets)) / (petroleum.wrapping_add(10));
    if v < 700 {
        v = 700;
    }
    if v > 100000 {
        v = 100000;
    }
    g.galaxy.oilprice = v;
}

// 5222:04a8
pub fn galaxy_open(g: &mut Sre) {
    let o = g.open;
    g.open = o.wrapping_add(1);
    if o == 0 {
        g.open = system_load_data(g);
    }
}

// 5222:04c2
pub fn galaxy_close(g: &mut Sre) {
    if g.open == 0 {
        return;
    }
    g.open -= 1;
    if g.open != 0 {
        return;
    }
    system_save_all(g);
    g.current = -1;
}

// 5222:04f9
pub fn score_list_all(g: &mut Sre) {
    kout!(g; b'\n');
    let m = empire_mask(g);
    score_list(g, m, None);
}

fn show_master(g: &mut Sre, what: &str, c: Color, idx: i16) {
    kout!(g; cn(g), what, c);
    let n = cs(&g.emp[idx as usize].name);
    put_untranslated(g, KOUT, &n);
    kout!(g; cn(g), ".\n");
}

// 5222:052e
pub fn show_masters(g: &mut Sre) {
    galaxy_open(g);
    if g.galaxy.planetmaster != -1 {
        let i = g.galaxy.planetmaster;
        show_master(g, "Today's Planet Master is ", col(11), i);
    }
    if g.galaxy.solarmaster != -1 {
        let i = g.galaxy.solarmaster;
        show_master(g, "Today's Solar Master is ", col(14), i);
    }
    if g.galaxy.weaponsmaster != -1 {
        let i = g.galaxy.weaponsmaster;
        show_master(g, "Today's Weapons Master is ", col(13), i);
    }
    galaxy_close(g);
}

fn alive(g: &Sre, e: usize) -> bool {
    g.emp[e].exists() && total_planets(g, e) > 0 && g.emp[e].population > 0
}

fn letter(g: &Sre, e: usize) -> u8 {
    if g.emp[e].id == -1 {
        0
    } else {
        (g.emp[e].id as u8).wrapping_add(b'A')
    }
}

// 14f6:01ed
pub fn mask_test(mask: i32, bit: i16) -> bool {
    if (bit as u16) > 0x18 {
        return false;
    }
    ((mask as u32) >> bit) & 1 != 0
}

// 14f6:0213
pub fn mask_set(mask: &mut i32, bit: i16) {
    if (bit as u16) > 0x18 {
        return;
    }
    *mask |= (1u32 << bit) as i32;
}

// 14f6:0237
pub fn mask_clear(mask: &mut i32, bit: i16) {
    if (bit as u16) > 0x18 {
        return;
    }
    *mask &= !((1u32 << bit) as i32);
}

// 5222:07c8: the list of players and scores; empires in mask are marked.
pub fn score_list(g: &mut Sre, mask: i32, o: Option<Sid>) {
    let mut ctitle: i16 = 0xe;
    let mut cline: i16 = 8;
    let mut ckey: i16 = 0xe;
    let mut cbracket: i16 = 2;
    let (mut c0, mut c1, mut c2, mut c3, mut c4): (i16, i16, i16, i16, i16) = (0xf, 0xb, 0xe, 0xd, 0xa);
    if g.open == 0 {
        out!(g, ERRLOG; "Galaxy not open.\n");
        return;
    }
    {
        let mut rt = Resource::num(b"Scores.Colors.Title", ctitle as i32);
        let mut rl = Resource::num(b"Scores.Colors.Line", cline as i32);
        let mut rk = Resource::num(b"Scores.Colors.Key", ckey as i32);
        let mut rb = Resource::num(b"Scores.Colors.Brackets", cbracket as i32);
        let mut r0 = Resource::num(b"Scores.Colors.Col0", c0 as i32);
        let mut r1 = Resource::num(b"Scores.Colors.Col1", c1 as i32);
        let mut r2 = Resource::num(b"Scores.Colors.Col2", c2 as i32);
        let mut r3 = Resource::num(b"Scores.Colors.Col3", c3 as i32);
        let mut r4 = Resource::num(b"Scores.Colors.Col4", c4 as i32);
        let db = &mut g.resdb;
        ctitle = rt.long(db) as i16;
        ckey = rk.long(db) as i16;
        cline = rl.long(db) as i16;
        cbracket = rb.long(db) as i16;
        c0 = r0.long(db) as i16;
        c1 = r1.long(db) as i16;
        c2 = r2.long(db) as i16;
        c3 = r3.long(db) as i16;
        c4 = r4.long(db) as i16;
    }
    let o = o.unwrap_or(KOUT);
    let tourney = |g: &Sre| g.galaxy.enddate < never_date();
    out!(g, o; col(ctitle), "List of Players/Scores:\n");
    out!(g, o; setw(4), col(ckey), "ID");
    out!(g, o; setw(0x20), col(c0), "Empire");
    out!(g, o; setwr(11), col(c1), "Planets");
    let t = tourney(g) as i16;
    out!(g, o; setwr(11 - t), col(c2), "Score");
    let t = tourney(g) as i16;
    out!(g, o; setwr(13 - t), col(c3), "Net Worth");
    if tourney(g) {
        out!(g, o; setwr(7), col(c4), "Points");
    }
    out!(g, o; setw(0), b'\n', col(cline));
    let n = tourney(g) as i16 * 5 + 0x47;
    for _ in 0..n {
        out!(g, o; 0xc4u8);
    }
    out!(g, o; b'\n');
    for i in 0..MAXEMPIRES {
        if !g.emp[i].exists() {
            continue;
        }
        if o == KOUT && key_ready(g, KIN) {
            return;
        }
        let l = letter(g, i);
        if mask_test(mask, i as i16) {
            out!(g, o; col(cbracket), b'<', col(ckey), l, col(cbracket), b'>');
        } else if alive(g, i) {
            out!(g, o; col(7), b' ', l, b' ');
        } else {
            out!(g, o; col(4), b' ', l, b' ');
        }
        out!(g, o; b' ', col(c0), setw(0x20));
        let nm = ename(g, i);
        put_untranslated(g, o, &nm);
        out!(g, o; setw(0));
        if !alive(g, i) {
            out!(g, o; setwr(0x16), col(12), "DEAD!", setw(0), b'\n');
            continue;
        }
        let tp = total_planets(g, i);
        out!(g, o; col(c1), setw(11), tp, setw(0));
        out!(g, o; col(c2));
        let t = tourney(g) as i16;
        let sc = g.emp[i].score;
        out!(g, o; setw(11 - t), sc, setw(0));
        out!(g, o; col(c3));
        let t = tourney(g) as i16;
        let nw = net_worth(g, i);
        out!(g, o; setw(13 - t), nw, setw(0));
        if tourney(g) {
            let p = g.emp[i].points;
            out!(g, o; col(c4), setw(7), p, setw(0));
        }
        out!(g, o; cn(g), b'\n');
    }
    out!(g, o; col(cline));
    let n = tourney(g) as i16 * 5 + 0x47;
    for _ in 0..n {
        out!(g, o; 0xc4u8);
    }
    out!(g, o; cn(g), b'\n');
}

// 5222:13bb
pub fn galaxy_playable(g: &mut Sre) -> bool {
    if g.time.today < g.galaxy.startdate {
        kout!(g; "This game cannot be played until ", cnum(g));
        let t = g.galaxy.startdate;
        put_time(g, KOUT, t);
        kout!(g; cn(g), b'\n');
        return false;
    }
    if g.time.today > g.galaxy.enddate {
        kout!(g; "This game has ended.\n");
        return false;
    }
    true
}

// 5222:14d3
pub fn empire_mask(g: &mut Sre) -> i32 {
    galaxy_open(g);
    let mut m: i32 = 0;
    for i in 0..MAXEMPIRES {
        if g.emp[i].exists() {
            mask_set(&mut m, i as i16);
        }
    }
    galaxy_close(g);
    m
}

// 5222:1539
pub fn protected_mask(g: &mut Sre) -> i32 {
    galaxy_open(g);
    let mut m: i32 = 0;
    for i in 0..MAXEMPIRES {
        if g.emp[i].exists() && g.emp[i].year < 0 {
            mask_set(&mut m, i as i16);
        }
    }
    galaxy_close(g);
    m
}

// 5222:15b5: how well a search string matches a name (0 = not at all,
// else 1 + the normalization level needed).
fn name_match(mut search: Str, mut name: Str) -> i16 {
    for level in 0..=3 {
        name_normalize(&mut search, level);
        name_normalize(&mut name, level);
        if find(&name, &search, 0) >= 0 {
            return level + 1;
        }
    }
    0
}

// 5222:1666: the empire whose name matches best, or -1 (after listing
// the matches if there are several).
pub fn search_empire(g: &mut Sre, search: &[u8], allowed: i32) -> i16 {
    let mut score = [0i16; MAXEMPIRES];
    let mut best: i16 = 0;
    let mut count: i16 = 0;
    for i in 0..MAXEMPIRES {
        if !mask_test(allowed, i as i16) {
            score[i] = 0;
            continue;
        }
        score[i] = name_match(search.to_vec(), ename(g, i));
        if score[i] > best {
            best = score[i];
            count = 1;
        } else if score[i] == best {
            count += 1;
        }
    }
    if count == 1 {
        for i in 0..MAXEMPIRES {
            if score[i] == best {
                return i as i16;
            }
        }
    }
    if count == 0 || best == 0 {
        kout!(g; cn(g), "No match found.\n");
        return -1;
    }
    let mut rkey = Resource::num(b"Scores.Colors.Key", 14);
    let mut rbrk = Resource::num(b"Scores.Colors.Brackets", 4);
    let mut rcol = Resource::num(b"Scores.Colors.Col0", 15);
    kout!(g; cn(g), "Your string matches:\n");
    for i in 0..MAXEMPIRES {
        if score[i] != best {
            continue;
        }
        let b = rbrk.long(&mut g.resdb) as i16;
        kout!(g; col(b), " (");
        let k = rkey.long(&mut g.resdb) as i16;
        kout!(g; col(k), (g.emp[i].id as u8).wrapping_add(b'A'));
        let b = rbrk.long(&mut g.resdb) as i16;
        kout!(g; col(b), ") ");
        let c = rcol.long(&mut g.resdb) as i16;
        kout!(g; col(c), uname(g, i), b'\n');
    }
    -1
}

// 5222:19df: choose a set of empires by letter (Z toggles all); the
// result is in *chosen.  Returns 1 when done, 0 on ESC, or the key in
// exits that ended the choice.
pub fn choose_empires(g: &mut Sre, allowed: i32, chosen: &mut i32, exits: &[u8], list: fn(&mut Sre, i32), prompt: Option<&[u8]>) -> i16 {
    let p: Str = prompt.unwrap_or(b"Select empires >> ").to_vec();
    let mut sel: Vec<u8> = Vec::new();
    kout!(g; cn(g), &p, cin(g));
    for i in 0..MAXEMPIRES {
        if mask_test(*chosen, i as i16) {
            sel.push(i as u8 + b'A');
            kout!(g; i as u8 + b'A');
        }
    }
    loop {
        kout!(g; cin(g));
        let mut c = in_char(g, KIN);
        if c == b'?' {
            kout!(g; cn(g), b'\n');
            list(g, allowed);
            kout!(g; cn(g), &p, cin(g));
            for j in 0..sel.len() {
                kout!(g; sel[j]);
            }
        }
        if c == b'/' {
            kout!(g; cn(g), "\nEmpire search: ");
            let buf = in_line(g, KIN);
            if !buf.is_empty() {
                let k = search_empire(g, &buf, allowed);
                if k != -1 {
                    c = (k as u8).wrapping_add(b'A');
                }
            }
            kout!(g; cn(g), &p, cin(g));
            for j in 0..sel.len() {
                kout!(g; sel[j]);
            }
        }
        if c == 27 || findch(exits, c, 0) != -1 {
            kout!(g; cn(g));
            for _ in 0..sel.len() {
                kout!(g; "\x08 \x08");
            }
            let mut j: i16 = 0;
            while display_len(g, &p) > j {
                kout!(g; "\x08 \x08");
                j += 1;
            }
            if c == 27 {
                return 0;
            }
            return c as i8 as i16;
        }
        if c == 8 && !sel.is_empty() {
            sel.pop();
            kout!(g; cn(g), "\x08 \x08");
        }
        if ctype_signed(c) & (CT_UPP | CT_LOW) != 0 {
            c = b_toupper(c);
            let all = c == b'Z';
            let (lo, hi) = if all { (b'A' as i8, b'Z' as i8) } else { (c as i8, (c as i8).wrapping_add(1)) };
            let mut x = lo;
            while x < hi {
                let xc = x as u8;
                let mut found = false;
                for j in 0..sel.len() {
                    if sel[j] != xc {
                        continue;
                    }
                    kout!(g; cn(g), "\x08 ");
                    for _ in j..sel.len() {
                        kout!(g; 8u8);
                    }
                    sel.remove(j);
                    for k in j..sel.len() {
                        kout!(g; cin(g), sel[k]);
                    }
                    found = true;
                    break;
                }
                if !found && mask_test(allowed, (xc as i16) - b'A' as i16) {
                    sel.push(xc);
                    kout!(g; cin(g), xc);
                }
                x = x.wrapping_add(1);
            }
        }
        if c == b'\r' || c == b'\n' {
            *chosen = 0;
            for &s in &sel {
                mask_set(chosen, s as i16 - b'A' as i16);
            }
            kout!(g; cn(g), b'\n');
            return 1;
        }
    }
}

// 5222:1ecd: choose one empire; -1 for none.
pub fn choose_empire(g: &mut Sre, allowed: i32) -> i16 {
    const PROMPT: &[u8] = b"Choose an empire: "; // DS:3ee8
    kout!(g; cn(g), PROMPT, " (/ to search) ");
    loop {
        kout!(g; cin(g));
        let mut c = in_char(g, KIN);
        if c == b'?' {
            kout!(g; "List Empires", cn(g), b'\n');
            score_list(g, allowed, None);
            kout!(g; cn(g), PROMPT, cin(g));
        }
        if c == b'/' {
            kout!(g; cn(g), "Search: ");
            let buf = in_line(g, KIN);
            if !buf.is_empty() {
                let k = search_empire(g, &buf, allowed);
                if k != -1 {
                    c = (k as u8).wrapping_add(b'A');
                }
            }
            kout!(g; cn(g), PROMPT, cin(g));
        }
        if c == 27 {
            kout!(g; "Abort", cn(g), b'\n');
            return -1;
        }
        if c == b'\r' || c == b'\n' {
            kout!(g; "None", cn(g), b'\n');
            return -1;
        }
        if ctype_signed(c) & (CT_UPP | CT_LOW) == 0 {
            continue;
        }
        c = b_toupper(c);
        let id = c as i8 as i16 - b'A' as i16;
        let mut rkey = Resource::num(b"Scores.Colors.Key", 14);
        let mut rbrk = Resource::num(b"Scores.Colors.Brackets", 4);
        if !mask_test(allowed, id) {
            continue;
        }
        let b = rbrk.long(&mut g.resdb) as i16;
        kout!(g; col(b), b'(');
        let k = rkey.long(&mut g.resdb) as i16;
        kout!(g; col(k), c);
        let b = rbrk.long(&mut g.resdb) as i16;
        kout!(g; col(b), ") ", cemp(g), uname(g, id as usize), cn(g), b'\n');
        return id;
    }
}
