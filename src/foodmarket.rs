// The food market and feeding the people and army (segment 512f).

use crate::prelude::*;

static R_FOODMARKET: GRes = GRes::text(b"FoodMarket.Name", b"the Solar Food Market"); // 3bae

// 512f:0000: the price of food, given the supply.
fn food_price(g: &Sre, extra: i32) -> i32 {
    let mut q = g.galaxy.marketfood.wrapping_add(extra);
    if q < 0 {
        q = 0;
    }
    if q < 160000 {
        return 85 - q / 2000;
    }
    5
}

// 512f:0072
fn food_demand(g: &Sre, extra: i32) -> &'static str {
    let p = food_price(g, extra);
    if p < 10 {
        "very low"
    } else if p < 20 {
        "low"
    } else if p < 40 {
        "moderate"
    } else if p < 60 {
        "medium"
    } else if p < 80 {
        "high"
    } else {
        "great"
    }
}

// 512f:0105: the food needed by the people (1), the army (2), or both.
pub fn food_needed(g: &Sre, e: usize, who: i16) -> i32 {
    if who == 1 {
        return 38i32.wrapping_mul(g.emp[e].population) / 100;
    }
    if who == 2 {
        return 3i32.wrapping_mul(g.emp[e].soldiers) / 100;
    }
    food_needed(g, e, 1).wrapping_add(food_needed(g, e, 2))
}

fn key(g: &mut Sre, c: u8) {
    kout!(g; cbr(g), b'[', cbo(g), c, cbr(g), b']', cn(g));
}

// 512f:018b
pub fn food_market(g: &mut Sre, p: &mut Player, e: usize) {
    let how = " credits per megaton, how many megatons of food would you like to ";
    let give = "How much will you give? ";
    let name = g.grs(&R_FOODMARKET);
    kout!(g; cn(g), "\nWelcome to ", col(14), name, b'\n', cn(g));
    let people = food_needed(g, e, 1);
    let army = food_needed(g, e, 2);
    let grown = g.emp[e].planets[P_FOOD].count * g.emp[e].planets[P_FOOD].output;
    let extra = g.emp[e].food - people - army;
    let price = food_price(g, extra);
    let demand = food_demand(g, extra);
    if food_needed(g, e, 0) > grown && g.emp[e].food / 3 < food_needed(g, e, 0) {
        kout!(g; col(3),
              "Your advisors inform you that your food planets do not produce enough food for you to be self-sufficient.\n",
              cn(g));
    }
    loop {
        kout!(g; "Current quantity available is ", cnum(g), g.galaxy.marketfood, cn(g), ".  ");
        kout!(g; "Food is in ", demand, " demand.\n");
        kout!(g; "You currently have ", cnum(g), g.emp[e].food, cn(g), " megatons of food.\n");
        kout!(g; "Would you like to ");
        key(g, b'B');
        kout!(g; "uy, ");
        key(g, b'S');
        kout!(g; "ell, ");
        if g.system.regstate != RS_UNREGISTERED {
            key(g, b'V');
            kout!(g; "isit the bank, ");
        }
        kout!(g; "or ");
        key(g, b'C');
        kout!(g; "ontinue?  ", cin(g));
        let mut c;
        loop {
            c = get_choice(g, b"CVBS", true);
            if g.system.regstate == RS_UNREGISTERED && c == b'V' {
                c = 0;
            }
            if c != 0 {
                break;
            }
        }
        if c == b'B' {
            let mut max = g.emp[e].money.get() / price;
            if g.galaxy.marketfood < max {
                max = g.galaxy.marketfood;
            }
            let mut need = people + army - g.emp[e].food;
            if need < 0 {
                need = 0;
            }
            if need > max {
                need = max;
            }
            kout!(g; "Buy", cn(g), "\nAt ", cnum(g), price, cn(g), how, "buy? \n", cbr(g), b'[', cbo(g), "MAX=", max, cbr(g),
                  b']', cn(g), "  ");
            set_defval(g, KIN, if need > 0 { need } else { 0 });
            set_maxval(g, KIN, max);
            let n = in_i32(g, KIN);
            g.emp[e].money.sub(n.wrapping_mul(price));
            if g.galaxy.coordfund < 1500000000 {
                g.galaxy.coordfund += n.wrapping_mul(price) / 1000;
            }
            g.emp[e].food += n;
            g.galaxy.marketfood -= n;
        } else if c == b'S' {
            kout!(g; "Sell", cn(g), "\nAt ", cnum(g), price / 2, cn(g), how, "sell? \n", cbr(g), b'[', cbo(g), "MAX=",
                  g.emp[e].food, cbr(g), b']', cn(g), "  ");
            set_defval(g, KIN, 0);
            let f = g.emp[e].food;
            set_maxval(g, KIN, f);
            let n = in_i32(g, KIN);
            g.emp[e].money.add(n.wrapping_mul(price / 2));
            g.galaxy.coordfund -= n.wrapping_mul(price / 2) / 1000;
            if g.galaxy.coordfund < 0 {
                g.galaxy.coordfund = 0;
            }
            g.emp[e].food -= n;
            g.galaxy.marketfood += n;
        } else if c == b'V' {
            kout!(g; "Visit Bank", cn(g), b'\n');
            crate::bank::bank(g, p, e);
        }
        if c != b'C' {
            continue;
        }
        kout!(g; "Continue", cn(g), "\n\n");
        let mut food = g.emp[e].food;
        let mut flags = g.emp[e].shortfalls;
        let mut effchange = g.emp[e].effchange;
        let mut starving = g.emp[e].starving;
        let fedpeople;
        let fedarmy;
        if people > 0 {
            let def = if people < food { people } else { food };
            if has_pref(p.prefs1, PF1_TERSE) {
                kout!(g; "People:  ", cnum(g), people, cn(g), b'\n');
            } else {
                kout!(g; "Your people require ", cnum(g), people, cn(g), " megatons of food.\n");
            }
            kout!(g; give);
            set_defval(g, KIN, def);
            set_maxval(g, KIN, food);
            fedpeople = in_i32(g, KIN);
            if fedpeople < people {
                flags |= SF_STARVED;
                starving = (people - fedpeople).wrapping_mul(g.emp[e].population) / people;
                if starving < 0 {
                    starving = 0;
                }
            }
            food -= fedpeople;
        } else {
            fedpeople = 0;
        }
        if army > 0 {
            let def = if army < food { army } else { food };
            if has_pref(p.prefs1, PF1_TERSE) {
                kout!(g; "Army:    ", cnum(g), army, cn(g), b'\n');
            } else {
                kout!(g; "\nYour Army needs ", cnum(g), army, cn(g), " megatons of food.\n");
            }
            kout!(g; give);
            set_defval(g, KIN, def);
            set_maxval(g, KIN, food);
            fedarmy = in_i32(g, KIN);
            if fedarmy < army {
                let k = (-((39 * (army - fedarmy)) / army)) as i16;
                if (effchange as i16) > k {
                    effchange = k as i8;
                }
                flags |= SF_ARMY;
            }
            food -= fedarmy;
        } else {
            fedarmy = 0;
        }
        if fedpeople < people || fedarmy < army {
            kout!(g; "\nFailure to feed your people/army will lead to civil war and/or loss of forces.\n");
            if yes_no(g, 1, b"Do you wish to reconsider your actions") {
                kout!(g; b'\n');
                continue;
            }
        }
        g.emp[e].shortfalls = flags;
        g.emp[e].effchange = effchange;
        g.emp[e].starving = starving;
        g.emp[e].food = food;
        return;
    }
}
