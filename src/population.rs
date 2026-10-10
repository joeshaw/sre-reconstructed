// End of a turn: donations and deposits, and population change (segment 5f38).

use crate::prelude::*;

// 5f38:0000
pub fn population(g: &mut Sre, _p: &mut Player, e: usize) {
    let mut n: i16 = 0;
    let tp = total_planets(g, e);
    let x = g.emp[e].clone();
    let ins2 = (x.insurgency as i32 * x.insurgency as i32) as i16 as i32;
    let mut born = (tp / 2 + 101 * x.planets[P_URBAN].count + x.population) / (random(10) as i32 + 23) + 1;
    let mut deaths = x.population * (x.population / (random(10) as i32 + 17)) / (total_planets(g, e) + 1) / 10;
    let mut emigrants = x.population * (ins2 + 1) / 80;
    let mut immigrants =
        (36 * (x.planets[P_EDUCATION].b * x.planets[P_EDUCATION].count) + x.population) / (ins2 + 50);
    let untaxed = 100 - x.taxrate as i32;
    immigrants = untaxed * (immigrants * untaxed) / 10000;
    emigrants += emigrants * (x.taxrate as i32 * x.taxrate as i32 + 1) / 200;
    emigrants += x.planets[P_TOURISM].count * total_planets(g, e) / 100000;
    emigrants += x.d188 / 100;
    deaths += x.planets[P_ORE].count / 10;
    let mut pollution = 10 * x.d188 / (total_planets(g, e) + 1);
    if pollution < 0 {
        pollution = 0;
    }
    deaths += pollution;
    if emigrants < 0 {
        emigrants = 0;
    }
    if x.population > 2000000000 {
        immigrants = 0;
        born = 0;
    }
    while emigrants + deaths > x.population && n < 100 {
        emigrants -= emigrants / 7;
        deaths -= deaths / 7;
        n += 1;
    }
    while emigrants + deaths > x.population {
        emigrants -= 1;
    }
    if x.population < deaths {
        deaths = x.population;
    }
    if born < 0 {
        born = 0;
    }
    if deaths < 0 {
        deaths = 0;
    }
    if immigrants < 0 {
        immigrants = 0;
    }
    if emigrants < 0 {
        emigrants = 0;
    }
    if deaths < pollution {
        pollution = deaths;
    }
    g.emp[e].population += born + immigrants - emigrants - deaths;
    if g.emp[e].population < 0 {
        g.emp[e].population = 0;
    }
    hline(g, col(8), 0x4b);
    let x = &g.emp[e];
    if random(10) == 0
        && x.b1c4 & 1 == 0
        && (x.b1c4 >> 1) & 1 == 0
        && (x.b1c4 >> 3) & 1 == 0
        && (x.b1c4 >> 2) & 1 == 0
        && x.d188 == 0
        && x.d1b0 > 0
        && x.insurgency == 0
    {
        kout!(g; "Your people love thee and hope you rule forever!\n\n");
    }
    if born > 0 {
        kout!(g; cnum(g), born, cn(g), " million people were born in your empire.\n");
    }
    if deaths > 0 {
        kout!(g; cnum(g), deaths, cn(g), " million citizens die");
        if pollution > 0 {
            kout!(g; ", including ", cnum(g), pollution, cn(g), " million from galactic pollution");
        }
        kout!(g; ".\n");
    }
    if immigrants > 0 {
        kout!(g; cnum(g), immigrants, cn(g), " million people immigrate to your empire.\n");
    }
    if emigrants > 0 {
        kout!(g; cnum(g), emigrants, cn(g), " million citizens emigrate from your empire.\n");
    }
    kout!(g; "A net population change of ", cnum(g));
    let net = born - deaths + immigrants - emigrants;
    if net > 0 {
        kout!(g; b'+');
    }
    kout!(g; net, cn(g), " million people.\n");
    hline(g, col(8), 0x4b);
}

// 5f38:07ae
pub fn end_of_turn(g: &mut Sre, p: &mut Player, e: usize) {
    kout!(g; b'\n');
    if g.emp[e].turnsused as i32 == g.emp[e].turnsperday as i32 - 1 {
        kout!(g; "This is your last turn today.\n");
    }
    let mut money = g.emp[e].money.get();
    let mut donate = (p.prefs1 >> 7) & 1 != 0;
    let mut gift = money;
    let m = g.emp[e].m1bc.get();
    if 1000i32.wrapping_mul(m) / 100 < gift {
        gift = 1000i32.wrapping_mul(m) / 100;
    }
    if (p.prefs1 >> 6) & 1 != 0 && money > 0 {
        donate = yes_no(g, donate as i16, b"Donate to the Galactic Coordinator");
        if donate {
            kout!(g; "Donate how much?  ");
            set_maxval(g, KIN, money);
            set_defval(g, KIN, gift);
            gift = in_i32(g, KIN);
        }
    }
    if donate {
        money = money.wrapping_sub(gift);
    }
    if g.system.regstate != 0 {
        let mut deposit = (p.prefs1 >> 5) & 1 != 0;
        if (p.prefs1 >> 4) & 1 != 0 && money > 0 {
            deposit = yes_no(g, deposit as i16, b"Deposit the rest of your funds into the bank");
        }
        if deposit {
            let old = g.emp[e].bank.get();
            g.emp[e].bank.add(money);
            money = money.wrapping_sub(g.emp[e].bank.get().wrapping_sub(old));
        }
    }
    if donate {
        g.galaxy.d3ac += gift / 1000;
        // The original divides by zero (and aborts) when there was no income.
        let m = g.emp[e].m1bc.get();
        let mut k = if m != 0 { gift / m / 10 } else { 0 };
        if k > 10 {
            k = 10;
        }
        if k < 0 {
            k = 0;
        }
        g.emp[e].d1b0 += k;
    }
    g.emp[e].money.set(money);
}
