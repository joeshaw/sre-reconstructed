// Civil wars from neglected planets or starvation, and desertion from an
// unpaid or unfed army (segment 44e2).

use crate::prelude::*;

// 44e2:04f2: lose pct percent of the military.
fn lose_military(g: &mut Sre, pct: i32, e: usize, l: &mut ItemList) {
    let x = &mut g.emp[e];
    let soldiers = pct.wrapping_mul(x.soldiers) / 100;
    let fighters = pct.wrapping_mul(x.fighters) / 100;
    let stations = pct.wrapping_mul(x.stations) / 100;
    let heavy = pct.wrapping_mul(x.heavycruisers) / 100;
    let generals = pct.wrapping_mul(x.generals) / 100;
    let carriers = pct.wrapping_mul(x.carriers) / 100;
    let light = pct.wrapping_mul(x.lightcruisers) / 100;
    x.soldiers -= soldiers;
    x.fighters -= fighters;
    x.stations -= stations;
    x.heavycruisers -= heavy;
    x.generals -= generals;
    x.carriers -= carriers;
    x.lightcruisers -= light;
    l.skip_zero();
    l.add(soldiers, b"soldier");
    l.add(fighters, b"fighter");
    l.add(stations, b"defense station");
    l.add(light, b"light cruiser");
    l.add(heavy, b"heavy cruiser");
    l.add(generals, b"general");
    l.add(carriers, b"carrier");
}

// 44e2:07cc
pub fn civil_war(g: &mut Sre, _p: &mut Player, e: usize) {
    let mut civil = false;
    let mut pct: i32;
    let cause: &str;
    let mut size: &str = "";
    let killed: i32;
    let mut total: i32 = 0;
    let mut military = ItemList::new(cn(g), cnum(g));
    let mut lost = ItemList::new(cn(g), cnum(g));
    lost.skip_zero();
    if g.emp[e].shortfalls & SF_NEGLECT != 0 {
        // Neglected planets revolt.
        insurgency_up(g, e);
        for i in 0..10i16 {
            let mut n = g.emp[e].neglected[i as usize];
            if g.emp[e].planets[i as usize].count < n {
                n = g.emp[e].planets[i as usize].count;
            }
            lost.add_plural(n, planet_name(i as i8), b"");
            g.galaxy.marketplanets += n;
            total += n;
            g.emp[e].planets[i as usize].count -= n;
        }
        // The original divides by zero if no planets are left.
        let tp = total_planets(g, e);
        pct = if tp != 0 { 100 * total / tp } else { 100 };
        killed = pct.wrapping_mul(g.emp[e].population) / 100;
        g.emp[e].population -= killed;
        lose_military(g, pct, e, &mut military);
        if lost.count() > 0 {
            size = if pct < 20 {
                "minor"
            } else if pct < 30 {
                "small"
            } else if pct < 50 {
                "large"
            } else if pct < 70 {
                "major"
            } else {
                "disastrous"
            };
            cause = "money";
            civil = true;
        } else {
            cause = "unemployment";
        }
    } else if g.emp[e].shortfalls & SF_STARVED != 0 {
        // Starvation.
        let starving = g.emp[e].starving;
        // The original divides by zero if the population is gone.
        let pop = g.emp[e].population;
        pct = if pop != 0 { 100i32.wrapping_mul(starving) / pop } else { 0 };
        if pct < 0 {
            pct = 0;
        }
        if pct > 30 {
            pct = 30;
        }
        killed = pct.wrapping_mul(g.emp[e].population) / 100;
        g.emp[e].population -= killed;
        insurgency_up(g, e);
        for i in 0..10i16 {
            let n = pct.wrapping_mul(g.emp[e].planets[i as usize].count) / 100;
            lost.add_plural(n, planet_name(i as i8), b"");
            g.galaxy.marketplanets += n;
            total += n;
            g.emp[e].planets[i as usize].count -= n;
        }
        lose_military(g, pct, e, &mut military);
        if lost.count() > 0 {
            cause = "food";
            size = if pct < 10 {
                "inconsequential"
            } else if pct < 20 {
                "minor"
            } else {
                "major"
            };
            civil = true;
        } else {
            cause = "food shortages";
        }
    } else {
        return;
    }
    if !civil && killed > 0 {
        kout!(g; cn(g), "Your people are in riots due to ", cause,
              ", but your officials prevent a civil war from breaking out.");
        if killed > 0 {
            kout!(g; "  However, ", cnum(g), killed, cn(g), " million people were killed in the riots.");
        }
        kout!(g; b'\n');
    }
    if civil {
        if random(100) as i32 <= pct && show_screen(g, None, b"CivilWar", false) {
            pause_prompt(g);
        }
        kout!(g; "Without ", cause, ", your empire plunges into a ", size, col(13), " CIVIL WAR", cn(g), "!\n", cn(g),
              "You lose ");
        put_item_list(g, KOUT, &lost);
        kout!(g; " planets (total ", col(14), total, cn(g), b')');
        if killed > 0 {
            kout!(g; " and ", cnum(g), killed, cn(g), " million people");
        }
        kout!(g; ".\n");
        if g.grb(&R_CIVILWAR) && pct > 30 {
            out!(g, NEWS; cemp(g), uname(g, e), cn(g), " has suffered a major civil war!\n");
        }
    }
    if military.count() > 0 {
        kout!(g; cn(g), "You also lose ");
        put_item_list(g, KOUT, &military);
        kout!(g; ".\n");
    }
    hline(g, col(8), 0x4b);
    if !civil {
        return;
    }
    let topic = b"civil-war";
    use crate::help::show_help_topic2;
    if yes_no(g, (pct > 25) as i16, b"Display hints to avoid civil wars") {
        kout!(g; b'\n');
        show_help_topic2(g, topic, b"general");
        let x = &g.emp[e];
        if x.planets[P_TOURISM].count > x.planets[P_ORE].count {
            show_help_topic2(g, topic, b"tourism");
            let x = &g.emp[e];
            if x.planets[P_ANTIPOLLUTION].output * x.planets[P_ANTIPOLLUTION].count < x.planets[P_PETROLEUM].count {
                kout!(g; "Also, tourism profits decrease when you have pollution caused by petroleum planets.");
            }
            kout!(g; "\n\n");
        }
        let x = &g.emp[e];
        if x.planets[P_FOOD].output * x.planets[P_FOOD].count > crate::foodmarket::food_needed(g, e, 0) {
            show_help_topic2(g, topic, b"long-term-food");
        }
        show_help_topic2(g, topic, b"continuous");
        if g.system.regstate != RS_UNREGISTERED {
            show_help_topic2(g, topic, b"loan");
        }
        kout!(g; "\n\nHope this helps!\n\n");
    }
    let x = &g.emp[e];
    if x.favor >= 25
        && pct > 25
        && x.money.get().wrapping_add(x.bank.get()) < 100000000
        && (x.soldiers + x.fighters + x.stations + x.heavycruisers + x.agents < 2000 || total_planets(g, e) < 300)
    {
        kout!(g; "The Galactic Coordinator offers to give you some funds to recover from the civil war.  ");
        if yes_no(g, 1, b"Do you accept") {
            let mut v = g.emp[e].favor;
            g.emp[e].favor -= 25;
            if v > 100 {
                v = 100;
            }
            v = v.wrapping_mul(15 * total_planets(g, e));
            g.emp[e].money.add(v);
            kout!(g; "You receive ", cnum(g), v, cn(g), " credits.\n");
        } else {
            kout!(g; "The Galactic Coordinator wishes you luck in rebuilding your empire!\n");
        }
    }
    hline(g, col(8), 0x4b);
}

// 44e2:0000
pub fn desertion(g: &mut Sre, _p: &mut Player, e: usize) {
    let x = &mut g.emp[e];
    if x.effnext >= 0 {
        x.effectiveness = (x.effectiveness as i32 + x.effchange as i32) as i8;
        if x.effectiveness < 0 {
            x.effectiveness = 0;
        }
        if x.effectiveness > 100 {
            x.effectiveness = 100;
        }
        return;
    }
    x.effectiveness = 0;
    if x.shortfalls & SF_ARMY == 0 {
        return;
    }
    let mut k: i16 = (x.effnext as i16).wrapping_neg();
    if k > 20 {
        k = 20;
    }
    if k < 0 {
        k = 0;
    }
    let k = k as i32;
    let soldiers = x.soldiers.wrapping_mul(k) / 100;
    let fighters = x.fighters.wrapping_mul(k) / 100;
    let stations = x.stations.wrapping_mul(k) / 100;
    let heavy = x.heavycruisers.wrapping_mul(k) / 100;
    let generals = x.generals.wrapping_mul(k) / 100;
    let carriers = x.carriers.wrapping_mul(k) / 100;
    let light = x.lightcruisers.wrapping_mul(k) / 100;
    x.soldiers -= soldiers;
    x.fighters -= fighters;
    x.stations -= stations;
    x.heavycruisers -= heavy;
    x.generals -= generals;
    x.carriers -= carriers;
    x.lightcruisers -= light;
    if soldiers > 0 {
        kout!(g; cnum(g), soldiers, cn(g), " troops desert you.\n");
    }
    if generals > 0 {
        kout!(g; cnum(g), generals, cn(g), " generals leave your army.\n");
    }
    if fighters > 0 {
        kout!(g; cnum(g), fighters, cn(g), " fighters decay due to lack of maintainance funds.\n");
    }
    if heavy > 0 {
        kout!(g; cnum(g), heavy, cn(g), " heavy cruisers decompose due to lack of funds.\n");
    }
    if light > 0 {
        kout!(g; cnum(g), light, cn(g), " light cruisers fail to work due to lack of proper maintenance.\n");
    }
    if stations > 0 {
        kout!(g; cnum(g), stations, cn(g), " defense stations fall into ruins.\n");
    }
    if carriers > 0 {
        kout!(g; cnum(g), carriers, cn(g), " carriers are no longer functional.\n");
    }
    if soldiers > 0 || generals > 0 || fighters > 0 || heavy > 0 || stations > 0 || carriers > 0 || light > 0 {
        hline(g, col(8), 0x4b);
    }
}
