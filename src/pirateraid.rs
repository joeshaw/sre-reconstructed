// Raids on the pirate teams (segment 5fe0).

use crate::pirates::*;
use crate::prelude::*;

// 5fe0:0000
pub fn choose_pirates(g: &mut Sre) -> i16 {
    let mut n = pirate_count(g);
    if n == 0 {
        kout!(g; "No pirate teams have been created.\n");
        return -1;
    }
    if n > 9 {
        n = 9;
    }
    let mut m = Menu::new(g, b"Pirates", b"Pirate Teams", [0xf, 6, 6, 0xf, 0xe, 7, 7, 7, 7, 7]);
    for i in 1..=n {
        let pt = (i - 1) as usize;
        let name = pirate_name(g, pt);
        m.add(g, &name, b'0' + i as u8, i);
        let c = pirate_color(g, pt);
        let it = m.find(i).unwrap();
        it.colors[0] = 6;
        it.colors[1] = 0xf;
        it.colors[2] = c;
        for k in 3..8 {
            it.colors[k] = 7;
        }
        it.load_colors(g);
    }
    m.run(g, true, -1)
}

// 5fe0:018a
fn ask_fleet(g: &mut Sre, name: &str, def: i32, max: i32) -> i32 {
    kout!(g; name, ":  ");
    if def != max {
        kout!(g; cbr(g), b'[', cbo(g), "MAX=", col(3), max, cbr(g), b']', cn(g), "  ");
    }
    set_defval(g, KIN, def);
    set_maxval(g, KIN, max);
    in_i32(g, KIN)
}

fn shrink(v: &mut i32, by: i32) {
    *v = v.wrapping_add(by);
    if *v < 0 {
        *v = 0;
    }
}

// 5fe0:0297
pub fn pirate_raid(g: &mut Sre, _p: &mut Player, e: usize) -> bool {
    let mut r: i16 = 0;
    let team = choose_pirates(g);
    if team == -1 {
        return false;
    }
    let pt = (team - 1) as usize;
    kout!(g; col(3), "Specify your attack fleet:\n", cn(g));
    let money = g.emp[e].money.get();
    let credits = ask_fleet(g, "Credits", money / 10, money);
    let mut chance: i16;
    if 500i32.wrapping_mul(net_worth(g, e)) < credits {
        chance = 1000;
    } else {
        chance = (2i32.wrapping_mul(credits) / net_worth(g, e)) as i16;
    }
    if chance < 200 {
        chance = 200;
    }
    let v = g.emp[e].soldiers;
    let mut s = ask_fleet(g, "Soldiers", v, v);
    let sents = s;
    let v = g.emp[e].fighters;
    let mut f = ask_fleet(g, "Fighters", v, v);
    let sentf = f;
    let v = g.emp[e].heavycruisers;
    let mut c = ask_fleet(g, "Cruisers", v, v);
    let sentc = c;
    if s == 0 && f == 0 && c == 0 {
        return false;
    }
    g.emp[e].money.sub(credits);
    let ps = s.wrapping_add(g.pir[pt].d98);
    let pf = f.wrapping_add(g.pir[pt].d9c);
    let pc = c.wrapping_add(g.pir[pt].da4);
    kout!(g; col(13));
    let chance = chance as i16;
    loop {
        if random(1100) < chance && g.pir[pt].d98 > 0 {
            shrink(&mut g.pir[pt].d98, -ps / 80 - 1);
        }
        if random(1300) < chance && g.pir[pt].da4 > 0 {
            shrink(&mut g.pir[pt].da4, -pc / 80 - 1);
        }
        if random(1200) < chance && g.pir[pt].d9c > 0 {
            shrink(&mut g.pir[pt].d9c, -pf / 80 - 1);
        }
        if random(1200) < chance && g.pir[pt].da0 > 0 {
            let d = g.pir[pt].da0;
            shrink(&mut g.pir[pt].da0, -d / 80 - 1);
        }
        if random(500) < 80 && s > 0 {
            s -= ps / 80 + 1;
        }
        if random(500) < 80 && c > 0 {
            c -= pc / 80 + 1;
        }
        if random(500) < 80 && f > 0 {
            f -= pf / 80 + 1;
        }
        kout!(g; if r % 2 != 0 { b'/' } else { b'\\' });
        r += 1;
        if r > 25 {
            break;
        }
        if !(sents / 2 < s || sentc / 2 < c || sentf / 2 < f) {
            break;
        }
        let x = &g.pir[pt];
        if !(x.d98 > 0 || x.da4 > 0 || x.d9c > 0) {
            break;
        }
        if (ps + pc + pf) / 5 > s + c + f {
            break;
        }
    }
    kout!(g; cn(g), b'\n');
    if s < 0 {
        s = 0;
    }
    if f < 0 {
        f = 0;
    }
    if c < 0 {
        c = 0;
    }
    g.emp[e].soldiers -= sents - s;
    g.emp[e].heavycruisers -= sentc - c;
    g.emp[e].fighters -= sentf - f;
    let mut score: i16 = 0;
    {
        let x = &g.pir[pt];
        if x.d9c > 0 {
            score -= 1;
        }
        if x.d98 > 0 {
            score -= 1;
        }
        if x.da4 > 0 {
            score -= 1;
        }
        if s > 0 {
            score += 1;
        }
        if c > 0 {
            score += 1;
        }
        if f > 0 {
            score += 1;
        }
        if x.d9c > f {
            score -= 1;
        }
        if x.d98 > s {
            score -= 1;
        }
        if x.da4 > c {
            score -= 1;
        }
        if x.d9c < f {
            score += 1;
        }
        if x.d98 < s {
            score += 1;
        }
        if x.da4 < c {
            score += 1;
        }
        if 2 * f < x.d9c {
            score -= 1;
        }
        if 2 * s < x.d98 {
            score -= 1;
        }
        if 2 * c < x.da4 {
            score -= 1;
        }
        if 2 * x.d9c < f {
            score += 1;
        }
        if 2 * x.d98 < s {
            score += 1;
        }
        if 2 * x.da4 < c {
            score += 1;
        }
        if 2 * x.d9c + x.d98 + 13 * x.da4 > 2 * f + s + 13 * c {
            score -= 5;
        } else {
            score += 5;
        }
    }
    let mut lost = ItemList::new(cn(g), cnum(g));
    lost.skip_zero();
    lost.add(sents - s, b"soldier");
    lost.add(sentf - f, b"fighter");
    lost.add(sentc - c, b"heavy cruiser");
    if score > 0 {
        let mut total: i32 = 0;
        let mut div: i16 = 3;
        if random(13) == 0 && show_screen(g, None, b"RaidWar", false) {
            pause_prompt(g);
        }
        if g.emp[e].year < 0 {
            div += 3;
        }
        if g.emp[e].score > 100000 {
            div += 2;
        }
        if total_planets(g, e) > 500 {
            div += 2;
        }
        if net_worth(g, e) > 100000 {
            div += 4;
        }
        if score > 3 {
            div -= 1;
        }
        if score > 6 {
            div -= 1;
        }
        if score > 9 {
            div -= 1;
        }
        if div < 2 {
            div = 2;
        }
        for k in 0..10 {
            let mut take = g.pir[pt].wb4[k] / div;
            if take > 100 {
                take = 100;
            }
            if take < 0 {
                take = 0;
            }
            g.emp[e].planets[k].count += take as i32;
            g.pir[pt].wb4[k] = g.pir[pt].wb4[k].wrapping_sub(take);
            if g.pir[pt].wb4[k] < 0 {
                g.pir[pt].wb4[k] = 0;
            }
            total += take as i32;
        }
        let d = div as i32;
        let mut got = ItemList::new(cn(g), col(14));
        got.skip_zero();
        let x = g.pir[pt].clone();
        got.add(total, b"planet");
        got.add_plural(x.d94, b"megatons of food", b"");
        got.add(x.d98 / d, b"soldier");
        got.add(x.d9c / d, b"fighter");
        got.add(x.da0 / d, b"defense station");
        got.add(x.da4 / d, b"heavy cruiser");
        got.add(x.da8 / d, b"carrier");
        got.add(x.dac / d, b"general");
        got.add(x.db0 / d, b"covert agent");
        got.add(x.money, b"credit");
        {
            let y = &mut g.emp[e];
            y.soldiers += x.d98 / d;
            y.fighters += x.d9c / d;
            y.generals += x.dac / d;
            y.heavycruisers += x.da4 / d;
            y.carriers += x.da8 / d;
            y.stations += x.da0 / d;
            y.agents += x.db0 / d;
            y.food = y.food.wrapping_add(x.d94);
            y.money.add(x.money);
        }
        let id = g.emp[e].id as usize;
        let p = &mut g.pir[pt];
        p.wd4[id] = p.wd4[id].wrapping_add(1);
        let v = p.d98;
        shrink(&mut p.d98, -v / d);
        let v = p.d9c;
        shrink(&mut p.d9c, -v / d);
        let v = p.dac;
        shrink(&mut p.dac, -v / d);
        let v = p.da4;
        shrink(&mut p.da4, -v / d);
        let v = p.da8;
        shrink(&mut p.da8, -v / d);
        let v = p.da0;
        shrink(&mut p.da0, -v / d);
        let v = p.db0;
        shrink(&mut p.db0, -v / d);
        p.d94 = 0;
        p.money = 0;
        if g.grb(&R_PIRATERAIDS) {
            let name = pirate_name(g, pt);
            out!(g, NEWS; cemp(g), uname(g, e), cn(g), " raided ", col(12), name, cn(g), ".\n");
        }
        if random(13) == 0 && show_screen(g, None, b"RaidWar", false) {
            pause_prompt(g);
        }
        kout!(g; "You lost ");
        put_item_list(g, KOUT, &lost);
        kout!(g; ", and you ");
        kout!(g; if score > 10 {
            "slaughtered"
        } else if score > 8 {
            "crushed"
        } else if score > 5 {
            "humiliated"
        } else {
            "defeated"
        });
        kout!(g; " the ");
        put_pirate(g, KOUT, pt);
        kout!(g; cn(g), ".  ");
        kout!(g; "You captured ");
        put_item_list(g, KOUT, &got);
        kout!(g; ".\n");
    } else {
        if g.grb(&R_PIRATERAIDS) {
            let name = pirate_name(g, pt);
            out!(g, NEWS; cemp(g), uname(g, e), cn(g), " attempted to raid ", col(12), name, cn(g), ", but failed.\n");
        }
        kout!(g; "You lost ");
        put_item_list(g, KOUT, &lost);
        kout!(g; "; you were unable to defeat the ");
        put_pirate(g, KOUT, pt);
        kout!(g; cn(g), ".\n");
    }
    true
}

// 5fe0:19d9
pub fn spy_on_pirates(g: &mut Sre) -> i16 {
    let n = pirate_count(g);
    let mut m = Menu::new(g, b"PirateSpy", b"Spy Report on Pirate Teams", [7, 6, 7, 7, 7, 7, 7, 7, 7, 7]);
    m.add(g, b"Team", 0, 100);
    m.find(100).unwrap().add_column_str(b"Planets");
    m.find(100).unwrap().add_column_str(b"Net Worth");
    for i in 1..=n {
        let pt = (i - 1) as usize;
        let name = pirate_name(g, pt);
        m.add(g, &name, b'0' + i as u8, i);
        let c = pirate_color(g, pt);
        let (pp, pn) = (pirate_planets(g, pt), pirate_net_worth(g, pt));
        let it = m.find(i).unwrap();
        it.colors = [c & 7, 0xf, c, 0xb, 0xd, 7, 7, 7];
        it.load_colors(g);
        it.add_column_num(pp);
        it.add_column_num(pn);
    }
    m.show(g);
    0
}
