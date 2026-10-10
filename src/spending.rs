// Government spending: prices and the buy/sell menu (segment 1af6).

use crate::lottery::put_max;
use crate::prelude::*;

// Prices.  The galaxy's w384 flag selects the formulas that grow with
// the empire's year and forces instead of only its net worth.

// 1af6:0006
pub fn price_soldiers(g: &Sre, e: usize) -> i32 {
    let x = &g.emp[e];
    if g.galaxy.w384 == 0 {
        return net_worth(g, e) / 1000 + 180;
    }
    (((x.year as i32) << 1) + 180) as i16 as i32 + x.soldiers / 1800 + x.fighters / 12400 + net_worth(g, e) / 5000
}

// 1af6:00b8
pub fn price_fighters(g: &Sre, e: usize) -> i32 {
    let x = &g.emp[e];
    if g.galaxy.w384 == 0 {
        return net_worth(g, e) / 800 + 240;
    }
    (((x.year as i32 * 5) as i16 as i32) / 2 + 240) as i16 as i32 + x.soldiers / 7000 + x.fighters / 1690
        + net_worth(g, e) / 400
}

// 1af6:0172
pub fn price_stations(g: &Sre, e: usize) -> i32 {
    let x = &g.emp[e];
    if g.galaxy.w384 == 0 {
        return net_worth(g, e) / 700 + 320;
    }
    (x.year as i32 * 3 + 320) as i16 as i32 + x.soldiers / 7500 + x.fighters / 9000 + x.stations / 1460
        + net_worth(g, e) / 300
}

// 1af6:024c
pub fn price_generals(g: &Sre, e: usize) -> i32 {
    let x = &g.emp[e];
    if g.galaxy.w384 == 0 {
        return net_worth(g, e) / 300 + 670;
    }
    14 * x.year as i32 + x.soldiers / 1790 + x.generals / 170 + total_planets(g, e) / 10 + 670
}

// 1af6:0307
pub fn price_cmd_ship(g: &Sre, e: usize) -> i32 {
    let x = &g.emp[e];
    if g.galaxy.w384 == 0 {
        return net_worth(g, e) / 100 + 8800;
    }
    x.soldiers / 850 + x.fighters / 950 + 5000
}

// 1af6:0379
pub fn price_planet(g: &Sre, e: usize) -> i32 {
    let x = &g.emp[e];
    if g.galaxy.w384 == 0 {
        return net_worth(g, e) / 100 + total_planets(g, e) + 13000;
    }
    let k: i32 = if x.year < 0 {
        3
    } else if total_planets(g, e) >= 300 {
        1
    } else {
        2
    };
    let v = total_planets(g, e) + x.score / 2 + net_worth(g, e) / 10 + if x.year < 0 { 0 } else { x.year as i32 };
    (97i32.wrapping_mul(v).wrapping_add(50000)) / k / 10
}

// 1af6:0495
pub fn price_agents(g: &Sre, e: usize) -> i32 {
    let x = &g.emp[e];
    if g.galaxy.w384 == 0 {
        return net_worth(g, e) / 200 + 4300;
    }
    x.agents + 18 * x.year as i32 + 7 * total_planets(g, e) + x.score / 10 + 2218
}

// 1af6:0542
pub fn price_heavy_cruisers(g: &Sre, e: usize) -> i32 {
    let x = &g.emp[e];
    if g.galaxy.w384 == 0 {
        return net_worth(g, e) / 600 + 1100;
    }
    7 * x.year as i32 + x.fighters / 1220 + total_planets(g, e) / 5 + x.stations / 2500 + net_worth(g, e) / 150 + 1100
}

// 1af6:0624
pub fn price_carriers(g: &Sre, e: usize) -> i32 {
    let x = &g.emp[e];
    if g.galaxy.w384 == 0 {
        return net_worth(g, e) / 700 + 850;
    }
    7 * x.year as i32 + total_planets(g, e) / 10 + net_worth(g, e) / 100 + 850
}

// The empire's forces by item number (soldiers 1 .. carriers 9).
fn field_get(x: &Empire, i: usize) -> i32 {
    match i {
        1 => x.soldiers,
        2 => x.fighters,
        3 => x.stations,
        4 => x.generals,
        7 => x.agents,
        8 => x.heavycruisers,
        9 => x.carriers,
        _ => panic!("no such item"),
    }
}

fn field_set(x: &mut Empire, i: usize, v: i32) {
    match i {
        1 => x.soldiers = v,
        2 => x.fighters = v,
        3 => x.stations = v,
        4 => x.generals = v,
        7 => x.agents = v,
        8 => x.heavycruisers = v,
        9 => x.carriers = v,
        _ => panic!("no such item"),
    }
}

// 61a7:010a: buy up to max (-1: as many as affordable) of an item.
fn buy(g: &mut Sre, e: usize, price: i32, field: usize, name: &str, mut max: i32) {
    let m = g.emp[e].money.get();
    if price > 0 && (m / price < max || max < 0) {
        max = m / price;
    }
    if m < price || m <= 0 {
        kout!(g; "You can not afford any ", col(10), name, cn(g), "!\n");
        return;
    }
    if price <= 0 {
        kout!(g; "No ", cnum(g), name, cn(g), " are available to buy this year.\n");
        return;
    }
    kout!(g; cn(g), "Buy how many ", col(10), name, cn(g), "?  ");
    put_max(g, max);
    set_defval(g, KIN, 0);
    set_maxval(g, KIN, max);
    let n = in_i32(g, KIN);
    let v = field_get(&g.emp[e], field).wrapping_add(n);
    field_set(&mut g.emp[e], field, v);
    g.emp[e].money.sub(n.wrapping_mul(price));
}

// 61a7:0384
fn cmd_ship(g: &mut Sre, e: usize, price: i32) {
    if g.emp[e].cmdship >= 100 {
        kout!(g; "You have already completed work on your command ship!\n");
        return;
    }
    if g.emp[e].money.get() < price {
        kout!(g; "You can't afford to start your command ship now.\n");
        return;
    }
    let v: i16;
    if g.emp[e].cmdship > 0 {
        kout!(g; "You added some strength to your command ship.\n");
        v = g.emp[e].cmdship as i16 + 5;
    } else {
        kout!(g; "You started work on your command ship.  Your workers will automatically ");
        kout!(g; "add 5% to the strength of the command ship each year.\n");
        v = 5;
    }
    let x = &mut g.emp[e];
    x.cmdship = v as i8;
    if x.cmdship < 0 {
        x.cmdship = 0;
    }
    if x.cmdship > 100 {
        x.cmdship = 100;
    }
    x.money.sub(price);
}

// 61a7:04d2
fn sell(g: &mut Sre, e: usize, mut price: i32, field: usize, name: &str) {
    let have = field_get(&g.emp[e], field);
    if have <= 0 {
        kout!(g; "You don't own any ", col(10), name, cn(g), "!\n");
        return;
    }
    if price <= 0 {
        price = 1;
    }
    kout!(g; col(14), "Sell", cn(g), " how many ", col(10), name, cn(g), "?  ");
    put_max(g, have);
    set_defval(g, KIN, 0);
    set_maxval(g, KIN, have);
    let mut n = in_i32(g, KIN);
    let m = g.emp[e].money.get();
    if (2000000000 - m) / price < n {
        n = (2000000000 - m) / price;
    }
    let v = field_get(&g.emp[e], field).wrapping_sub(n);
    field_set(&mut g.emp[e], field, v);
    g.emp[e].money.add(n.wrapping_mul(price));
}

// 61a7:0000: a menu of the planet types with the empire's counts.
fn planet_menu(g: &mut Sre, m: &mut Menu, show: bool, e: usize) -> i16 {
    const LETTERS: &[u8] = b"FURPOSTAGE";
    const IDS: &[u8] = b"3740582961";
    let mut count = [0i32; 10];
    for i in 0..10 {
        count[i] = g.emp[e].planets[i].count;
    }
    for i in 0..10 {
        let id = (IDS[i] - b'0') as i16;
        m.add(g, planet_name(LETTERS[i] as i8), LETTERS[i], id);
        m.find(id).unwrap().add_column_num(count[id as usize]);
    }
    m.run(g, show, -1)
}

// 61a7:083d
fn how_many(g: &mut Sre, max: i32) -> i32 {
    kout!(g; cn(g), "How many planets?  ");
    put_max(g, max);
    set_defval(g, KIN, 0);
    set_maxval(g, KIN, max);
    in_i32(g, KIN)
}

// 61a7:091b
fn release_planets(g: &mut Sre, p: &mut Player, e: usize) {
    let mut show = true;
    while total_planets(g, e) > 0 {
        if (p.prefs1 >> 2) & 1 == 0 {
            show = true;
        }
        let mut m = Menu::new(g, b"SellPlanets", b"Release Planets", [7, 8, 1, 9, 7, 3, 7, 7, 7, 7]);
        // 61a7:0902
        m.prompt = Some(Box::new(|g: &mut Sre| {
            kout!(g; "You receive NO money from releasing planets.\n");
            1
        }));
        let k = planet_menu(g, &mut m, show, e);
        if k == -1 {
            kout!(g; b'\n');
            return;
        }
        let k = k as usize;
        let n = how_many(g, g.emp[e].planets[k].count);
        g.emp[e].w192 = g.emp[e].w192.wrapping_sub(n as i16);
        g.emp[e].planets[k].count -= n;
        g.galaxy.d392 += n;
        show = false;
    }
}

// 61a7:071e
fn buy_planets(g: &mut Sre, e: usize, price: i32, max: &mut i32) -> i32 {
    kout!(g; cn(g), "How many planets?  ");
    put_max(g, *max);
    set_defval(g, KIN, 0);
    set_maxval(g, KIN, *max);
    let n = in_i32(g, KIN);
    *max -= n;
    g.emp[e].score = g.emp[e].score.wrapping_add(3i32.wrapping_mul(n));
    g.emp[e].money.sub(n.wrapping_mul(price));
    n
}

// 61a7:0b7f
fn colonize(g: &mut Sre, p: &mut Player, e: usize, price: i32) {
    let mut avail = g.galaxy.d392;
    if g.emp[e].year < 0 && 300 - total_planets(g, e) > avail {
        avail = 300 - total_planets(g, e);
    }
    if avail + total_planets(g, e) > 30000 {
        avail = 30000 - total_planets(g, e);
    }
    if avail <= 0 {
        kout!(g; "There are no planets available to be colonized.\n");
        return;
    }
    let mut max = g.emp[e].money.get() / price;
    if max > avail {
        max = avail;
    }
    kout!(g; "There are ", cnum(g), avail, cn(g), " planets available.\n");
    if g.galaxy.d38e >= 0 && g.galaxy.d38e - (g.emp[e].w192 as i32) < max {
        max = g.galaxy.d38e - g.emp[e].w192 as i32;
        kout!(g; "You may buy ", cnum(g), g.galaxy.d38e, cn(g), " planets per turn.\n");
    }
    let mut show = true;
    while max > 0 {
        if (p.prefs1 >> 2) & 1 == 0 {
            show = true;
        }
        let mut m = Menu::new(g, b"BuyPlanets", b"Buy Planets", [7, 1, 1, 9, 0xf, 0xb, 7, 7, 7, 7]);
        // 61a7:0aab
        m.prompt = Some(Box::new(move |g: &mut Sre| {
            kout!(g; cn(g), "Current earnings:  [Ore = ");
            kout!(g; cnum(g), g.emp[e].planets[P_ORE].b, cn(g), "] [Tourism = ");
            kout!(g; cnum(g), g.emp[e].planets[P_TOURISM].b, cn(g), "] [Petroleum = ");
            let oil = oil_price(g);
            kout!(g; cnum(g), oil, cn(g), "].\n");
            1
        }));
        let k = planet_menu(g, &mut m, show, e);
        kout!(g; cn(g));
        if k == -1 {
            return;
        }
        show = false;
        if p.w70 & (1 << k) == 0 {
            let s = cat(&[b"Planet", &[k as u8]]);
            show_screen(g, None, &s, false);
            p.w70 |= (1i32 << k) as i16;
            crate::help::show_help_topic2(g, b"planet", planet_name(k as i8));
        }
        if k == 9 && g.emp[e].d188 > 0 {
            kout!(g; "There are ", cnum(g), g.emp[e].d188, cn(g), " exatons of pollution in the atmosphere.\n");
        }
        if k == 3 {
            kout!(g; "Each food planet can produce ", cnum(g), g.emp[e].planets[3].b, cn(g),
                  " megatons of food each year.\n");
        }
        if k == 8 {
            kout!(g; "Each supply planet can train ", cnum(g), g.emp[e].planets[8].b, cn(g),
                  " military units per year.  Soldiers count as 1 unit, fighters 1.3, defense stations 1.7, generals 7, carriers 9.5, and cruisers 12.\n");
        }
        let n = buy_planets(g, e, price, &mut max);
        g.emp[e].w192 = g.emp[e].w192.wrapping_add(n as i16);
        g.emp[e].planets[k as usize].count += n;
        g.galaxy.d392 -= n;
        if g.galaxy.d392 < 0 {
            g.galaxy.d392 = 0;
        }
    }
}

// 1af6:06ba
fn spending_header(g: &mut Sre, e: usize) {
    let turns = (g.emp[e].turnsperday as i32 - g.emp[e].turnsused as i32 - 1) as i16;
    kout!(g; cn(g), "You have ", cnum(g), g.emp[e].money.get(), cn(g), " credits, and ", cnum(g), turns, cn(g), " turns.\n");
}

static R_MILITARY: GRes = GRes::num(b"Spending.Military", 1); // 9b1e

// 1af6:07dd
pub fn spending(g: &mut Sre, p: &mut Player, e: usize) {
    let mut price = [0i32; 10];
    let mut sellprice = [0i32; 10];
    let mut owned = [0i32; 10];
    let names: [&str; 10] = [
        "",
        "Soldiers",
        "Fighters",
        "Defense Stations",
        "Generals",
        "Command Ship",
        "Colonize Planets",
        "Covert Agents",
        "Heavy Cruisers",
        "Carriers",
    ];
    price[1] = price_soldiers(g, e);
    price[2] = price_fighters(g, e);
    price[3] = price_stations(g, e);
    price[4] = price_generals(g, e);
    price[5] = price_cmd_ship(g, e);
    price[6] = price_planet(g, e);
    price[7] = price_agents(g, e);
    price[8] = price_heavy_cruisers(g, e);
    price[9] = price_carriers(g, e);
    for i in 1..=9 {
        sellprice[i] = price[i] / 3;
    }
    sellprice[6] = 0; // released planets bring nothing
    let mut selling = false;
    let mut show = true;
    loop {
        crate::play::time_check(g);
        let mut buym = Menu::new(g, b"Buying", b"Government Spending", [0xb, 9, 2, 0xe, 0xa, 0xb, 0xf, 7, 7, 7]);
        let mut sellm = Menu::new(g, b"Selling", b"Selling", [0xe, 4, 8, 0xf, 7, 0xc, 0xf, 7, 7, 7]);
        // 1af6:073d
        buym.prompt = Some(Box::new(move |g: &mut Sre| {
            spending_header(g, e);
            1
        }));
        // 1af6:076a
        sellm.prompt = Some(Box::new(move |g: &mut Sre| {
            spending_header(g, e);
            kout!(g; col(14), "SELL", cn(g), " which item?  ");
            0
        }));
        buym.add(g, b"Item", 0, 100);
        {
            let head = buym.find(100).unwrap();
            head.colors = [7; 8];
            head.load_colors(g);
        }
        buym.find(100).unwrap().add_column_str(b"Price");
        buym.find(100).unwrap().add_column_str(b"Owned");
        let mil = g.grb(&R_MILITARY);
        for i in 1..=9usize {
            let mut k = 0u8;
            if mil || i == 5 || i == 6 || i == 7 {
                k = b'0' + i as u8;
            }
            buym.add(g, names[i].as_bytes(), k, i as i16);
            if i == 5 {
                continue;
            }
            if i == 6 {
                sellm.add(g, b"Release Planets", b'6', 6);
            } else {
                sellm.add(g, names[i].as_bytes(), b'0' + i as u8, i as i16);
            }
        }
        buym.add(g, b"Sell", b'S', 10);
        buym.add(g, b"Operations", b'*', 11);
        sellm.add(g, b"Buy", b'B', 10);
        sellm.add(g, b"Operations", b'*', 11);
        if g.system.regstate != 0 {
            buym.add(g, b"", b'V', 0x50);
            sellm.add(g, b"", b'V', 0x50);
        }
        if (p.prefs2 >> 4) & 1 != 0 {
            buym.add(g, b"Quit", b'Q', 0x63);
        }
        let x = &g.emp[e];
        owned[1] = x.soldiers;
        owned[2] = x.fighters;
        owned[3] = x.stations;
        owned[4] = x.generals;
        owned[5] = x.cmdship as i32;
        owned[6] = total_planets(g, e);
        owned[7] = x.agents;
        owned[8] = x.heavycruisers;
        owned[9] = x.carriers;
        let cmdship = x.cmdship;
        for i in 1..=9usize {
            if let Some(b) = buym.find(i as i16) {
                if b.key != 0 {
                    b.add_column_num(price[i]);
                } else {
                    b.add_column_str(b"");
                }
            }
            if i == 5 {
                let cmdtext = cat(&[&int_str(cmdship as i32), b"%"]);
                if let Some(b) = buym.find(i as i16) {
                    b.add_column_str(&cmdtext);
                }
                continue;
            }
            if let Some(s) = sellm.find(i as i16) {
                s.add_column_num(sellprice[i]);
            }
            if let Some(b) = buym.find(i as i16) {
                b.add_column_num(owned[i]);
            }
            if let Some(s) = sellm.find(i as i16) {
                s.add_column_num(owned[i]);
            }
        }
        if (p.prefs1 >> 2) & 1 == 0 {
            show = true;
        }
        let mut r;
        if selling {
            r = sellm.run(g, show, -1);
            if (1..=9).contains(&r) {
                r += 20;
            }
            if r == -1 {
                r = 10;
            }
        } else {
            r = buym.run(g, show, -1);
        }
        if r == -1 && (p.prefs2 >> 4) & 1 == 0 {
            return;
        }
        if r == 0x63 {
            return;
        }
        show = false;
        match r {
            1 | 2 | 3 | 8 | 9 => buy(g, e, price[r as usize], r as usize, names[r as usize], -1),
            4 => {
                let cap = gov_capacity6(g, e);
                if cap <= 0 {
                    kout!(g; "One government planet is required for every 50 generals.  You don't have space on your planets for more generals.\n");
                } else {
                    buy(g, e, price[4], 4, names[4], cap);
                }
            }
            5 => cmd_ship(g, e, price[5]),
            6 => {
                colonize(g, p, e, price[6]);
                show = true;
            }
            7 => {
                let cap = gov_capacity(g, e);
                if cap <= 0 {
                    kout!(g; "One government planet is required for every 300 agents.  You don't have space on your planets for more covert agents.\n");
                } else {
                    buy(g, e, price[7], 7, names[7], cap);
                }
            }
            10 => {
                selling = !selling;
                show = true;
            }
            11 => {
                crate::operations::operations(g, p, e);
                show = true;
            }
            21 | 22 | 23 | 24 | 27 | 28 | 29 => {
                let i = (r - 20) as usize;
                sell(g, e, sellprice[i], i, names[i]);
            }
            26 => {
                release_planets(g, p, e);
                show = true;
            }
            0x50 => {
                crate::bank::bank(g, p, e);
                show = true;
            }
            _ => {}
        }
        if g.emp[e].name[0] == 0 || total_planets(g, e) <= 0 || g.emp[e].population <= 0 {
            return;
        }
    }
}
