// The production of the planets at the start of a turn (segment 462a).

use crate::prelude::*;

#[derive(Default)]
struct Income {
    petroleum: i32,     // 00
    urban: i32,         // 04
    tourism: i32,       // 08
    ore: i32,           // 0c
    food: i32,          // 10
    pollution: i32,     // 14 added by petroleum planets
    cleaned: i32,       // 18 removed by anti-pollution planets
    soldiers: i32,      // 1c
    fighters: i32,      // 20
    stations: i32,      // 24
    heavycruisers: i32, // 28
    carriers: i32,      // 2c
    generals: i32,      // 30
    credits: i32,       // 34 from supply planets
    taxes: i32,         // 38
}

// 462a:0000: produce everything; the total income (in thousands) is kept
// in m1bc.
fn produce(g: &mut Sre, e: usize, r: &mut Income) {
    *r = Income::default();
    let mut total: i32 = 0;
    let p = |g: &Sre, t: usize| g.emp[e].planets[t];
    if p(g, P_FOOD).count != 0 {
        r.food = p(g, P_FOOD).count * p(g, P_FOOD).b - g.emp[e].d188 / 10;
        if r.food < 0 {
            r.food = 0;
        }
        g.emp[e].food += r.food;
    }
    if p(g, P_ORE).count != 0 {
        r.ore = p(g, P_ORE).count * p(g, P_ORE).b;
        if p(g, P_ORE).count < 100 {
            let mut i: i32 = 0;
            while i < p(g, P_ORE).count {
                r.ore += random(100) as i32;
                i += 1;
            }
        }
        if r.ore < 0 {
            r.ore = 0;
        }
        total += r.ore / 1000;
        g.emp[e].money.add(r.ore);
    }
    if p(g, P_SUPPLY).count != 0 {
        let x = &mut g.emp[e];
        let c = x.planets[P_SUPPLY].count * x.planets[P_SUPPLY].b;
        let soldiers = c * x.b297[0] as i32 / 100;
        let fighters = c * x.b297[1] as i32 / 130;
        let stations = c * x.b297[2] as i32 / 170;
        let generals = c * x.b297[5] as i32 / 700;
        let carriers = c * x.b297[4] as i32 / 950;
        let cruisers = c * x.b297[3] as i32 / 1200;
        let rest = c - (10 * soldiers + 13 * fighters + 17 * stations + 70 * generals + 95 * carriers + 120 * cruisers) / 10;
        let credits = 100 * rest;
        r.soldiers = soldiers;
        x.soldiers += soldiers;
        r.fighters = fighters;
        x.fighters += fighters;
        r.stations = stations;
        x.stations += stations;
        r.heavycruisers = cruisers;
        x.heavycruisers += cruisers;
        r.generals = generals;
        x.generals += generals;
        r.carriers = carriers;
        x.carriers += carriers;
        r.credits = credits;
        x.money.add(credits);
        total += credits / 1000;
    }
    if p(g, P_TOURISM).count != 0 {
        let a = random(1000) as i32;
        let b = random(1000) as i32;
        let x = &mut g.emp[e];
        r.tourism = x.planets[P_TOURISM].count
            * (x.planets[P_TOURISM].b + a - b - 350 * x.insurgency as i32 - x.d188 / 10);
        if r.tourism < 0 {
            r.tourism = 0;
        }
        x.money.add(r.tourism);
        total += r.tourism / 1000;
    }
    if p(g, P_URBAN).count != 0 {
        let x = &mut g.emp[e];
        r.urban = x.planets[P_URBAN].count
            * (x.planets[P_URBAN].b - (x.insurgency as i32 * x.insurgency as i32) as i16 as i32);
        if r.urban < 0 {
            r.urban = 0;
        }
        x.money.add(r.urban);
        total += r.urban / 1000;
    }
    if p(g, P_PETROLEUM).count != 0 {
        let oil = oil_price(g);
        r.petroleum = p(g, P_PETROLEUM).count * oil;
        if r.petroleum < 0 {
            r.petroleum = 0;
        }
        g.emp[e].money.add(r.petroleum);
        total += r.petroleum / 1000;
        if oil_price(g) > 10000 {
            total += r.petroleum / 3000;
        }
        r.pollution = p(g, P_PETROLEUM).count;
        if r.pollution < 0 {
            r.pollution = 0;
        }
        g.emp[e].d188 += r.pollution;
    }
    if p(g, P_ANTIPOLLUTION).count != 0 {
        let x = &mut g.emp[e];
        r.cleaned = x.planets[P_ANTIPOLLUTION].b * x.planets[P_ANTIPOLLUTION].count;
        if r.cleaned < 0 {
            r.cleaned = 0;
        }
        if x.d188 - r.cleaned < 0 {
            r.cleaned = x.d188;
        }
        if r.cleaned < 0 {
            r.cleaned = 0;
        }
        x.d188 -= r.cleaned;
        if x.d188 < 0 {
            x.d188 = 0;
        }
    }
    if p(g, P_RESEARCH).count != 0 {
        let x = &mut g.emp[e];
        let mut v = x.planets[P_RESEARCH].count * x.planets[P_RESEARCH].b;
        if v < 0 {
            v = 0;
        }
        if v + x.d293 > 100000 {
            x.d293 = 100000;
        } else {
            x.d293 += v;
        }
    }
    if g.emp[e].population > 0 {
        let x = &g.emp[e];
        r.taxes = 3 * (x.population * x.taxrate as i32) / (x.insurgency as i32 + 1);
        for i in 0..MAXEMPIRES {
            if g.emp[e].treaty[i].typ() != 2 {
                continue;
            }
            if g.emp[i].exists() {
                let pop = g.emp[i].population;
                r.taxes += pop * random(16) as i32;
            }
        }
        if r.taxes < 0 {
            r.taxes = 0;
        }
        g.emp[e].money.add(r.taxes);
        total += r.taxes / 1000;
    }
    g.emp[e].m1bc.set(total);
}

// 462a:0b58: the report of the production.
pub fn turn_report(g: &mut Sre, _p: &mut Player, e: usize) {
    let mut r = Income::default();
    produce(g, e, &mut r);
    let pc = |g: &Sre, t: usize| g.emp[e].planets[t].count;
    if pc(g, P_FOOD) > 0 || r.food > 0 {
        if r.food > 0 {
            kout!(g; cnum(g), r.food, cn(g), " megatons of food grown.\n");
        } else {
            kout!(g; cnum(g), "NO", cn(g), " megatons of food were grown, due to excessive pollution.\n");
        }
    }
    if pc(g, P_RESEARCH) > 0 {
        let k = g.emp[e].lightcruisers / 70 + 10;
        let n = 10 * pc(g, P_RESEARCH) / k;
        if n > 0 {
            kout!(g; cnum(g), n, cn(g), " light cruisers were produced by your research planets.\n");
            g.emp[e].lightcruisers += n;
        }
    }
    if pc(g, P_SUPPLY) != 0 {
        let mut l = ItemList::new(cn(g), cnum(g));
        l.skip_zero();
        l.add(r.soldiers, b"soldier");
        l.add(r.fighters, b"fighter");
        l.add(r.stations, b"defense station");
        l.add(r.heavycruisers, b"heavy cruiser");
        l.add(r.generals, b"general");
        l.add(r.carriers, b"carrier");
        l.add(r.credits, b"credit");
        kout!(g; "Your supply planets produce ");
        put_item_list(g, KOUT, &l);
        kout!(g; ".\n");
    }
    if pc(g, P_ORE) > 0 || r.ore > 0 {
        kout!(g; cnum(g), r.ore, cn(g), " credits worth of ore has been mined.\n");
    }
    if pc(g, P_TOURISM) > 0 || r.tourism > 0 {
        if r.tourism > 0 {
            kout!(g; cnum(g), r.tourism, cn(g), " credits have been earned from tourist attractions.\n");
        } else {
            kout!(g; cnum(g), "NO", cn(g), " credits have been earned from tourist attractions,\n");
            kout!(g; "   possibly due to the poor conditions in your empire.\n");
        }
    }
    if pc(g, P_URBAN) > 0 || r.urban > 0 {
        kout!(g; cnum(g), r.urban, cn(g), " credits were collected as trade taxes on urban planets.\n");
    }
    if pc(g, P_PETROLEUM) > 0 || r.petroleum > 0 {
        kout!(g; cnum(g), r.petroleum, cn(g), " credits have been earned from petroleum pumping.\n");
    }
    let mut net = r.pollution - r.cleaned;
    let mut how = "released into";
    if net < 0 {
        net = -net;
        how = "removed from";
    }
    if net > 0 {
        kout!(g; cnum(g), net, cn(g), " exatons of pollutants have been ", how, " the galactic atmosphere.\n");
    }
    if r.taxes > 0 {
        kout!(g; "\nYou collect ", cnum(g), r.taxes, cn(g), " credits in taxes.\n");
    }
}
