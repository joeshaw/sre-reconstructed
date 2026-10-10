// Paying for the empire's upkeep at the start of a turn (segment 5e48).

use crate::prelude::*;

#[derive(Clone, Copy, Default)]
struct Costs {
    military: i32, // 00
    effect: i32,   // 04 raising the army's effectiveness
    violence: i32, // 08 reducing insurgency
    loan: i32,     // 0c loan payment
    planets: i32,  // 10 planet maintenance
    donation: i32, // 14 to the Galactic Coordinator
}

// The planets neglected this turn (e+0x1c7, ten longs).
pub fn neglected(g: &Sre, e: usize, i: i16) -> i32 {
    get_i32(&g.emp[e].u_1c7, 4 * i as usize)
}

pub fn set_neglected(g: &mut Sre, e: usize, i: i16, v: i32) {
    set_i32(&mut g.emp[e].u_1c7, 4 * i as usize, v);
}

// 5e48:0000
fn compute_costs(g: &Sre, e: usize) -> Costs {
    let x = &g.emp[e];
    let mut c = Costs::default();
    c.military = 2 * x.soldiers + 4 * x.fighters + 12 * x.heavycruisers + 10 * x.lightcruisers + 5 * x.stations;
    let v = (30 * x.fighters + 15 * x.soldiers + 75 * x.generals + 50 * x.heavycruisers + 30 * x.lightcruisers) / 10 + 17;
    let mut k: i16 = (100 - x.effectiveness as i32) as i16;
    if k > 20 {
        k = 20;
    }
    c.effect = k as i32 * (v / 100);
    c.violence = (((x.insurgency as i32 + 1) as i16 / 2) as i32) * (total_planets(g, e) + 2 * x.population);
    c.loan = 0;
    if x.bank.get() < 0 {
        c.loan = x.d1a4;
        if c.loan + x.bank.get() > 0 {
            c.loan = -x.bank.get();
        }
    }
    if g.galaxy.w384 == 0 {
        // Upkeep per planet grows with the logarithm of the size.
        const FRACTION: [i16; 8] = [0, 86, 266, 422, 533, 619, 689, 748];
        let mut per: i32 = -1400;
        let mut v = total_planets(g, e) + 500;
        while v >= 8 {
            per += 800;
            v /= 8;
        }
        per += FRACTION[v as usize] as i32;
        c.planets = per * total_planets(g, e);
    } else {
        c.planets = total_planets(g, e) * (x.year as i32 + 1000);
    }
    if c.planets < 0 {
        c.planets = 0;
    }
    c.donation = x.m1bc.get() * g.galaxy.w3b0 as i32;
    if g.galaxy.d3ac > 1500000000 {
        c.donation = 0;
    }
    c
}

// 5e48:03f6: the consequences of what was paid.
fn consequences(g: &mut Sre, e: usize, c: &Costs, paid: &mut Costs) {
    {
        let x = &mut g.emp[e];
        if paid.military < c.military {
            x.b1c5 = ((paid.military - c.military) / (c.military / 100 + 1)) as i8;
            x.b1c6 = (x.b1c5 as i32 + x.effectiveness as i32) as i8;
            x.b1c4 |= 8;
        }
        let mut per = c.effect;
        if x.effectiveness < 100 {
            let mut k: i16 = (100 - x.effectiveness as i32) as i16;
            if k > 20 {
                k = 20;
            }
            per = c.effect / k as i32;
        }
        if c.effect + c.effect / 2 < paid.effect {
            paid.effect = c.effect + c.effect / 2;
        }
        while paid.effect >= per && per > 0 {
            let v = (x.effectiveness as i32 + 1) as i8;
            x.effectiveness = v;
            if x.effectiveness < 0 {
                x.effectiveness = 0;
            }
            if x.effectiveness > 100 {
                x.effectiveness = 100;
            }
            paid.effect -= per;
        }
        if paid.violence >= c.violence - c.violence / 10 && x.insurgency > 0 {
            x.insurgency -= 1;
        }
        x.bank.add(paid.loan);
        if x.bank.get() >= 0 {
            x.d1a4 = 0;
        }
    }
    if paid.planets < c.planets {
        let n = ((c.planets - paid.planets) / (c.planets / 1000 + 1))
            * (total_planets(g, e) - 3 * g.emp[e].planets[P_GOVERNMENT].count)
            / 2000;
        if n > 0 {
            let k = total_planets(g, e) / n;
            let every: i16 = if k > 10000 {
                10000
            } else if k < 1 {
                1
            } else {
                k as i16
            };
            g.emp[e].b1c4 |= 1;
            let mut total: i32 = 0;
            for i in 0..10i16 {
                let v = g.emp[e].planets[i as usize].count / every as i32;
                set_neglected(g, e, i, v);
                total += neglected(g, e, i);
            }
            let mut j: i16 = 0;
            while j < 100 && total < n {
                let t = ((10 * b_rand() as i32) / 0x8000) as i16;
                if g.emp[e].planets[t as usize].count > neglected(g, e, t) {
                    let v = neglected(g, e, t) + 1;
                    set_neglected(g, e, t, v);
                    total += 1;
                }
                j += 1;
            }
        }
    }
    if paid.donation > c.donation {
        let mut v = (paid.donation - c.donation) / (g.emp[e].m1bc.get() + 1);
        if v > 1000 {
            v -= 1000;
        }
        if v > 0 {
            g.emp[e].d1b0 += v as i16 as i32;
        }
    }
    g.galaxy.d3ac += paid.donation / 1000;
    if total_planets(g, e) > 0 && g.emp[e].m1bc.get() < 2000000 {
        let tp = total_planets(g, e);
        g.emp[e].d1b4 = (1000i32.wrapping_mul(g.emp[e].m1bc.get())
            - c.military
            - c.effect
            - c.violence
            - c.loan
            - c.planets
            - c.donation)
            / tp;
    }
}

// 5e48:09e3: ask how much to pay of an amount.
fn ask_pay(g: &mut Sre, what: &str, amount: i32, avail: &mut Money, mut min: i32) -> i32 {
    if amount <= 0 {
        return 0;
    }
    let mut def = amount;
    if avail.get() < def {
        def = avail.get();
    }
    kout!(g; cn(g));
    if avail.get() < min {
        min = avail.get();
    }
    kout!(g; b'\n', what, b' ', cnum(g), amount, cn(g), " credits.\n");
    let mut v;
    loop {
        kout!(g; "Pay how much?  ");
        set_maxval(g, KIN, avail.get());
        set_defval(g, KIN, def);
        v = in_i32(g, KIN);
        kout!(g; cn(g));
        if v < min {
            kout!(g; "You must pay at least ", cnum(g), min, cn(g), " credits.\n");
        }
        if !(v < min) {
            break;
        }
    }
    avail.sub(v);
    v
}

// 5e48:0b9f
pub fn pay_upkeep(g: &mut Sre, p: &mut Player, e: usize) {
    let c = compute_costs(g, e);
    let mut paid = Costs::default();
    loop {
        let total = c.planets + c.loan + c.military + c.violence + c.donation;
        kout!(g; "Your empire requires ", cnum(g), total, cn(g), " credits this year.  You have ", cnum(g),
              g.emp[e].money.get(), cn(g), " credits.\n");
        if g.emp[e].money.get() >= total {
            let mut autopay = (p.prefs2 >> 1) & 1 != 0;
            if p.prefs2 & 1 != 0 {
                autopay = yes_no(g, autopay as i16, b"Pay everything automatically");
            }
            if autopay {
                paid = c;
                g.emp[e].money.sub(total);
                break;
            }
        }
        if g.system.regstate != 0 && yes_no(g, 0, b"Do you wish to visit the bank") {
            crate::bank::bank(g, p, e);
        }
        let mut avail = g.emp[e].money;
        let mut shortfall = false;
        paid.planets = ask_pay(g, "Maintaining your planets requires", c.planets, &mut avail, 0);
        shortfall |= paid.planets < c.planets;
        paid.loan = ask_pay(g, "For your loan, the bank requires that you pay back", c.loan, &mut avail, c.loan);
        paid.military = ask_pay(g, "Your military requires", c.military, &mut avail, 0);
        shortfall |= paid.military < c.military;
        paid.violence = ask_pay(g, "Violence within your empire can be reduced by spending", c.violence, &mut avail, 0);
        shortfall |= paid.violence < c.violence;
        paid.effect = ask_pay(g, "Your Army's effectiveness can be raised by spending", c.effect, &mut avail, 0);
        shortfall |= paid.effect < c.effect;
        paid.donation = ask_pay(g, "The Galactic Coordinator asks that you donate", c.donation, &mut avail, c.donation);
        if shortfall {
            kout!(g; "\nNot paying for maintenance of your empire may lead to DISASTEROUS consequences!  ");
            if yes_no(g, 1, b"Do you wish to reconsider") {
                continue;
            }
        }
        g.emp[e].money.set(avail.get());
        break;
    }
    consequences(g, e, &c, &mut paid);
}
