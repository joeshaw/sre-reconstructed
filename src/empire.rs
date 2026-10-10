// Empires (segments 48cc and 1c59).

use crate::prelude::*;

const PLANET_CODES: &[u8] = b"PETFROGUSA"; // DS:9d69

// 1c59:0099: planets of a type (index 0..9, or a code letter): the index.
#[allow(dead_code)] // unused, as in the original
pub fn planet_index(g: &mut Sre, mut typ: i16) -> usize {
    if typ > 9 {
        for i in 0..10 {
            if PLANET_CODES[i] as i8 as i16 == typ {
                typ = i as i16;
                break;
            }
        }
    }
    if (0..=9).contains(&typ) {
        return typ as usize;
    }
    kout!(g; "FATAL ERROR: planet-index=", typ, ".\n");
    out!(g, ERRLOG; "FATAL ERROR @Empire::Planets&( ", typ, " ).\n");
    g.exit(1);
}

// 1c59:004b
pub fn total_planets(g: &Sre, e: usize) -> i32 {
    let mut n: i32 = 0;
    for p in g.emp[e].planets.iter() {
        n = n.wrapping_add(p.count);
    }
    n
}

// 1c59:0009
pub fn insurgency_up(g: &mut Sre, e: usize) {
    if g.emp[e].insurgency < 7 {
        g.emp[e].insurgency += 1;
    }
}

// 1c59:002a
#[allow(dead_code)] // unused, as in the original
pub fn insurgency_down(g: &mut Sre, e: usize) {
    if g.emp[e].insurgency > 0 {
        g.emp[e].insurgency -= 1;
    }
}

// 1c59:028b
pub fn net_worth(g: &Sre, e: usize) -> i32 {
    let x = &g.emp[e];
    let mut t: i32 = 0;
    for i in 0..25 {
        let tr = x.treaty[i];
        if tr.typ() == 0 {
            continue;
        }
        t += (tr.a as i32 + 5) / 3;
        if tr.typ() == 2 {
            continue;
        }
        if tr.typ() == 1 {
            continue;
        }
        t += 1;
    }
    let planets = total_planets(g, e);
    t = t + (10 - x.insurgency as i32) * planets + x.money.get() / 100000 + 7 * x.agents + x.soldiers + x.fighters
        + 3 * x.heavycruisers
        + 3 * x.lightcruisers
        + (x.stations << 1)
        + 3 * x.carriers
        + (x.generals << 1)
        + x.population / (x.taxrate as i32 + 10)
        + x.cmdship as i32
        + x.bank.get() / 110000
        - x.d188
        + x.d16c / 2;
    if t < 0 {
        t = 0;
    }
    t
}

// 1c59:072c
pub fn empire_score(g: &Sre, e: usize) -> i32 {
    g.emp[e].score
}

// 1c59:0740
pub fn insurgency_name(g: &Sre, e: usize) -> &'static [u8] {
    match g.emp[e].insurgency {
        0 => b"Peaceful",
        1 => b"Mild Insurgencies",
        2 => b"Occasional Riots",
        3 => b"Violent Demonstrations",
        4 => b"Political Conflicts",
        5 => b"Internal Violence",
        6 => b"Revolutionary Warfare",
        7 => b"Under Coup",
        _ => b"(unknown)",
    }
}

// 1c59:07a8
pub fn planet_name(typ: i8) -> &'static [u8] {
    let mut t = typ as u8;
    if (0..10).contains(&typ) {
        t = PLANET_CODES[typ as usize];
    }
    match t {
        b'P' => b"Petroleum",
        b'E' => b"Education",
        b'T' => b"Tourism",
        b'F' => b"Food",
        b'R' => b"Research",
        b'O' => b"Ore",
        b'G' => b"Government",
        b'U' => b"Urban",
        b'S' => b"Supply",
        b'A' => b"Antipollution",
        _ => b"(unknown planet)",
    }
}

// ---------------------------------------------------------------------
// 48cc:11be
pub fn clear_spending(s: &mut [u8; 0x60]) {
    *s = [0; 0x60];
}

pub fn clamp100(mut v: i16) -> i8 {
    if v < 0 {
        v = 0;
    }
    if v > 100 {
        v = 100;
    }
    v as i8
}

fn set_planets(x: &mut Empire, typ: usize, count: i32, a: i32, b: i32) {
    x.planets[typ] = PlanetSet { count, a, b };
}

// 48cc:0254
pub fn empire_init(g: &mut Sre, e: usize) {
    let now = g.time.now;
    let open = g.open != 0;
    let (protection, grantturns) = (g.galaxy.protection, g.galaxy.grantturns);
    let x = &mut g.emp[e];
    x.id = -1;
    x.name[0] = 0;
    x.player = -1;
    x.lastplay = now;
    x.created = now;
    x.w2a1 = 4;
    set_planets(x, 0, 0, 1900, 1900);
    set_planets(x, 1, 3, 17, 39);
    set_planets(x, 2, 9, 2650, 3100);
    set_planets(x, 3, 1, 46, 53);
    set_planets(x, 4, 1, 9, 11);
    set_planets(x, 5, 9, 2200, 2500);
    set_planets(x, 6, 1, 2, 3);
    set_planets(x, 7, 3, 200, 210);
    set_planets(x, 8, 0, 11, 15);
    set_planets(x, 9, 0, 10, 10);
    for t in x.treaty.iter_mut() {
        t.settype(0);
        t.a = 0;
        t.sete(0);
        t.setd(0);
        t.setc(0);
    }
    for b in x.b2d3.iter_mut() {
        *b = 0;
    }
    x.lightcruisers = 0;
    x.generals = 0;
    x.carriers = 0;
    x.heavycruisers = 0;
    x.stations = 0;
    x.fighters = 0;
    x.soldiers = 0;
    x.b297[0] = clamp100(20);
    x.b297[1] = clamp100(20);
    x.b297[2] = clamp100(20);
    x.b297[3] = clamp100(20);
    x.b297[4] = clamp100(10);
    x.b297[5] = clamp100(10);
    x.d16c = 0;
    x.w172 = 0;
    x.flags = 0;
    x.insurgency = 0;
    x.effectiveness = clamp100(100);
    x.cmdship = clamp100(0);
    x.money.set(10000);
    x.score = 0;
    x.d178 = 0;
    x.population = 40;
    x.d188 = 0;
    x.food = 0;
    x.d1b4 = 0;
    x.d29d = 0;
    if open {
        x.year = protection.wrapping_neg();
        x.turnsused = 0;
        x.turnsperday = grantturns;
    } else {
        x.year = -20;
        x.turnsused = 0;
        x.turnsperday = 5;
    }
    x.w19a = x.year;
    x.bank.set(0);
    x.d1a4 = 0;
    x.w1a8 = 0;
    x.bonds = 0;
    x.maxbonds = 0;
    x.bondpayoff = 0;
    x.bondprice = 0;
    x.bondhours = 0;
    x.bondturns = 0;
    x.bondname[0] = 0;
    x.taxrate = clamp100(23);
    x.w1ab = 0;
    x.draftrate = clamp100(5);
    x.w1ae = 0;
    x.d1b0 = 0;
    x.phase = 0;
    x.m1bc.set(0);
    x.b1c4 &= !0x0f;
    clear_spending(&mut x.u_1c7);
    x.b1c6 = 0;
    x.b1c5 = 0;
    x.weapons &= !0x0f;
    x.weapons &= 0x0f;
    x.troubles = 0;
    x.bonuses = 0;
    x.w229 = 0;
    x.agents = 0;
    x.d22f = [0; 7];
    x.w26f = [0; 9];
    x.d293 = 0;
    x.w192 = 0;
    x.d2ec = 0;
}

// 48cc:112c
pub fn empire_init_id(g: &mut Sre, e: usize, id: i16) {
    empire_init(g, e);
    g.emp[e].id = id;
}

// 48cc:1146
pub fn empire_set_name(g: &mut Sre, e: usize, s: &[u8]) {
    set_name(&mut g.emp[e].name, s);
}

// 48cc:1196
pub fn oil_price(g: &mut Sre) -> i32 {
    if g.open == 0 {
        return 1900;
    }
    galaxy_oil_price(g);
    g.galaxy.d3b6
}

// 48cc:124f: the empire of a player, or -1.
pub fn empire_of_player(g: &mut Sre, player: i16) -> i16 {
    if player == -1 {
        return -1;
    }
    galaxy_open(g);
    for i in 0..MAXEMPIRES {
        if g.emp[i].exists() && g.emp[i].player == player {
            return i as i16;
        }
    }
    -1
}

// 48cc:12b9: continue on a new, indented line if past column n.
pub fn wrap_at(g: &mut Sre, n: i16) {
    if g.ost(KOUT).line.len > n {
        kout!(g; "\n", setw(12), "", setw(0));
    }
}

// 48cc:12fc
pub fn status_item(g: &mut Sre, v: i32, name: &[u8], flag: i16) {
    if v <= 0 {
        return;
    }
    wrap_at(g, (0x48i32 - name.len() as i32 - flag as i32 * 8) as i16);
    kout!(g; col(8), b'[', cn(g), name);
    if flag == 0 {
        kout!(g; if v == 1 { "=" } else { "s=" }, cnum(g), v, cn(g));
    }
    kout!(g; col(8), "] ", cn(g));
}

// 48cc:1400
fn planet_item(g: &mut Sre, typ: i16, p: &PlanetSet) {
    if p.count <= 0 {
        return;
    }
    let name = planet_name(typ as i8);
    wrap_at(g, (0x46i32 - name.len() as i32) as i16);
    kout!(g; col(8), b'[', cn(g), name, b'=', cnum(g), p.count, cn(g), col(8), "] ", cn(g));
}

// 48cc:14ec
pub fn empire_status(g: &mut Sre, e: usize, full: bool) {
    let sb = g.scr.sbcolor;
    kout!(g; col(14), "\nEmpire Status\n");
    hline(g, col(1), 0x4b);
    kout!(g; col(sb), "-", col(sb + 8), "*", col(15), ename(g, e), col(sb + 8), "*", col(sb), "-", cn(g), "\n");
    if full {
        kout!(g; setw(12), "Score:", setw(0), cnum(g), g.emp[e].score, cn(g), b'\n');
        kout!(g; setw(12), "Turns Left:", setw(0), cnum(g), (g.emp[e].turnsperday as i32 - g.emp[e].turnsused as i32 - 1) as i16, cn(g), b'\n');
    }
    kout!(g; setw(12), "Money:", setw(0), cnum(g), g.emp[e].money.get(), cn(g));
    if full {
        if g.emp[e].bank.get() > 0 {
            kout!(g; col(8), "   (", cn(g), "Bank Savings=", cnum(g), g.emp[e].bank.get(), col(8), b')', cn(g));
        } else if g.emp[e].bank.get() < 0 {
            kout!(g; col(8), "   (", cn(g), "Bank Loan=", cnum(g), g.emp[e].bank.get().wrapping_neg(), col(8), b')', cn(g));
        }
    }
    kout!(g; b'\n');
    kout!(g; setw(12), "Pop:", setw(0), cnum(g), g.emp[e].population, cn(g), " Million   ", col(8), b'(', cn(g), "Tax Rate=",
          g.emp[e].taxrate as i16, b'%', col(8), ")\n", cn(g));
    kout!(g; setw(12), "Food:", setw(0), cnum(g), g.emp[e].food, cn(g), " Megatons\n");
    if full {
        kout!(g; setw(12), "Covert:", setw(0), cnum(g), g.emp[e].agents, cn(g), " Agents\n");
    }
    kout!(g; setw(12), "Insurgency:", setw(0), cnum(g), insurgency_name(g, e), cn(g), b'\n');
    kout!(g; setw(12), "Cmd. Ship:", setw(0), cnum(g), g.emp[e].cmdship as i16, cn(g), "% completed\n");
    kout!(g; setw(12), "Military:", setw(0));
    status_item(g, g.emp[e].soldiers, b"Soldier", 0);
    status_item(g, g.emp[e].fighters, b"Fighter", 0);
    status_item(g, g.emp[e].stations, b"Defense Station", 0);
    status_item(g, g.emp[e].generals, b"General", 0);
    status_item(g, g.emp[e].heavycruisers, b"Heavy Cruiser", 0);
    status_item(g, g.emp[e].lightcruisers, b"Light Cruiser", 0);
    status_item(g, g.emp[e].carriers, b"Carrier", 0);
    status_item(g, ((g.emp[e].weapons >> 3) & 1) as i32, b"Mothership", 0);
    status_item(g, (g.emp[e].weapons & 1) as i32, b"Nuclear Bomb", 1);
    status_item(g, ((g.emp[e].weapons >> 1) & 1) as i32, b"Chemical Bomb", 1);
    status_item(g, ((g.emp[e].weapons >> 2) & 1) as i32, b"Psionic Bomb", 1);
    wrap_at(g, 0x34);
    kout!(g; col(8), b'[', cn(g), "Effectiveness=", cnum(g), g.emp[e].effectiveness as i16, cn(g), "%", col(8), "] ", cn(g));
    wrap_at(g, 0x38);
    let nw = net_worth(g, e);
    kout!(g; col(8), b'(', cn(g), "Net Worth=", col(15), nw, cn(g), col(8), ")\n", cn(g));
    kout!(g; setw(12), "Planets:", setw(0));
    for i in 0..10 {
        let p = g.emp[e].planets[i];
        planet_item(g, i as i16, &p);
    }
    wrap_at(g, 0x32);
    let tot = total_planets(g, e);
    kout!(g; col(8), b'(', cn(g), "Total=", col(15), tot, cn(g), col(8), ")\n", cn(g));
    if g.emp[e].troubles != 0 {
        kout!(g; setw(12), "Troubles:", setw(0));
        if g.emp[e].troubles & 1 != 0 {
            status_item(g, 1, b"Plague", 1);
        }
        if g.emp[e].troubles & 2 != 0 {
            status_item(g, 1, b"Famine", 1);
        }
        if g.emp[e].troubles & 4 != 0 {
            status_item(g, 1, b"Space Storms", 1);
        }
        if g.emp[e].troubles & 8 != 0 {
            status_item(g, 1, b"Dust Cloud", 1);
        }
        if g.emp[e].troubles & 0x10 != 0 {
            status_item(g, 1, b"Gas Cloud", 1);
        }
        if g.emp[e].troubles & 0x20 != 0 {
            status_item(g, 1, b"Bank Failure", 1);
        }
        kout!(g; b'\n');
    }
    if g.emp[e].bonuses != 0 {
        kout!(g; setw(12), "Bonuses:", setw(0));
        status_item(g, g.emp[e].bonuses & 1, b"Cloaking Device", 1);
        status_item(g, g.emp[e].bonuses & 2, b"Long Range Scanner", 1);
        status_item(g, g.emp[e].bonuses & 4, b"Genetically Engineered Food", 1);
        status_item(g, g.emp[e].bonuses & 8, b"Planetary Shield", 1);
        status_item(g, g.emp[e].bonuses & 0x10, b"Accura Weapons", 1);
        status_item(g, g.emp[e].bonuses & 0x20, b"Enhanced Spying", 1);
        kout!(g; b'\n');
    }
    if full {
        if g.emp[e].year < -1 {
            kout!(g; "You have ", cnum(g), g.emp[e].year.wrapping_neg(), cn(g), " turn");
            if g.emp[e].year != -1 {
                kout!(g; b's');
            }
            kout!(g; " of protection left.  ");
        } else if g.emp[e].year == -1 {
            kout!(g; "You are in the ", col(15), "LAST", cn(g), " turn of protection!  ");
        } else {
            kout!(g; "It is year ", cnum(g), g.emp[e].year, cn(g), " after the end of protection.  ");
        }
        kout!(g; "You have ", cnum(g), (g.emp[e].turnsperday as i32 - g.emp[e].turnsused as i32 - 1) as i16, cn(g), " turns left today.\n");
    }
    hline(g, col(1), 0x4b);
}

// 48cc:232d: the lottery pool pays out to protected empires.
pub fn protection_bonus(g: &mut Sre, e: usize) -> i32 {
    let x = &g.emp[e];
    if x.year < x.w19a {
        return 0;
    }
    let mut div: i32 = 100;
    if x.year <= 0 {
        div = 300;
    }
    if x.year < -25 {
        div = 400;
    }
    let mut v = g.galaxy.d3ac / div;
    if v < 0 {
        v = 0;
    }
    g.emp[e].money.add(1000i32.wrapping_mul(v));
    g.galaxy.d3ac -= v;
    let gt = g.galaxy.grantturns;
    g.emp[e].w19a = (g.emp[e].year as i32 + gt as i32) as i16;
    v
}

// 48cc:240a
pub fn show_status(g: &mut Sre, e: usize) {
    empire_status(g, e, true);
}

// 48cc:2420
pub fn next_phase(g: &mut Sre, e: usize) {
    let x = &mut g.emp[e];
    if x.phase < 0x15 {
        x.phase += 1;
    } else {
        x.phase = 0;
        x.turnsused = x.turnsused.wrapping_add(1);
        x.year = x.year.wrapping_add(1);
    }
}

// 48cc:2454
pub fn prev_phase(g: &mut Sre, e: usize) {
    let x = &mut g.emp[e];
    if x.phase >= 0 {
        x.phase -= 1;
    } else {
        x.phase = 0x15;
        x.turnsused = x.turnsused.wrapping_sub(1);
        x.year = x.year.wrapping_sub(1);
    }
}

// 48cc:2488: the command ship grows by itself once started.
pub fn build_cmd_ship(g: &mut Sre, e: usize) {
    let x = &mut g.emp[e];
    if x.cmdship > 0 {
        let step: i8 = if x.cmdship < 90 {
            5
        } else if x.cmdship < 95 {
            3
        } else {
            1
        };
        x.cmdship = clamp100(x.cmdship as i16 + step as i16);
    }
    if x.cmdship > 100 {
        x.cmdship = clamp100(100);
    }
    if x.cmdship < 0 {
        x.cmdship = clamp100(0);
    }
}

// 48cc:2598: bank interest on loans and savings.
pub fn bank_interest(g: &mut Sre, e: usize) {
    let interest = g.galaxy.interest as i32;
    let x = &mut g.emp[e];
    let mut b = x.bank.get();
    if b < 0 && b >= -500000000 {
        let v = interest.wrapping_mul(b / 100);
        x.bank.add(v);
    }
    b = x.bank.get();
    if b > 0 && b <= 1000000000 {
        let v = interest.wrapping_mul(b / 500);
        x.bank.add(v);
    }
}

// 14f6:02a6: move b a step towards a.
pub fn move_toward(p: &mut PlanetSet) {
    let diff = if p.a >= 0 { p.a - p.b } else { 0 };
    if diff >= 30 || diff <= -30 {
        p.b += diff / 30;
    } else if p.a > p.b {
        p.b += 1;
    } else if p.a < p.b {
        p.b -= 1;
    }
}

// 48cc:267c
pub fn planets_move_toward(g: &mut Sre, e: usize) {
    for p in g.emp[e].planets.iter_mut() {
        move_toward(p);
    }
}

// 48cc:26b0
pub fn update_score(g: &mut Sre, e: usize) {
    let tp = total_planets(g, e);
    let x = &mut g.emp[e];
    x.score -= x.score / 19;
    x.score += tp / 6;
    x.score += x.population / (x.taxrate as i32 + 10);
}

// 48cc:2731
pub fn update_d1b0(g: &mut Sre, e: usize) {
    let tp = total_planets(g, e);
    let x = &mut g.emp[e];
    if x.d1b0 < 0 {
        x.d1b0 += 1;
    }
    if x.d1b0 > 0 {
        x.d1b0 -= 2;
    }
    if (tp - x.planets[P_PETROLEUM].count) / 10 < x.planets[P_ANTIPOLLUTION].count {
        x.d1b0 += 1;
    }
}

// 48cc:27c6: spare capacity of government planets.
pub fn gov_capacity(g: &Sre, e: usize) -> i32 {
    let x = &g.emp[e];
    300 * x.planets[P_GOVERNMENT].count - 6 * x.generals - x.agents
}

// 48cc:282f
pub fn gov_capacity6(g: &Sre, e: usize) -> i32 {
    gov_capacity(g, e) / 6
}

// Set a treaty's type and its default terms (inlined in the original).
pub fn set_treaty(t: &mut Treaty, typ: i16, a: u8) {
    t.settype(typ);
    t.a = a;
    t.sete(0);
    t.setd(0);
    if t.typ() == 3 {
        t.sete(0x14);
        t.setd(0x14);
    }
    if t.typ() == 4 {
        t.sete(0x3c);
    }
    if t.typ() == 5 {
        t.setd(0x3c);
    }
    if t.typ() == 6 {
        t.sete(0x28);
        t.setd(0x28);
    }
}
