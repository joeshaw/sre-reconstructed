// Random events at the start of a turn: pirates, revolts, disasters,
// research developments and weapons offers (segment 6572).

use crate::pirates::*;
use crate::prelude::*;
use crate::research::*;

static R_NUCLEAR: GRes = GRes::num(b"Attacks.Nuclear", 1); // 816a
static R_CHEMICAL: GRes = GRes::num(b"Attacks.Chemical", 1); // 8176
static R_PSIONIC: GRes = GRes::num(b"Attacks.Psionic", 1); // 8182

// 6572:0000: good news.
fn good(g: &mut Sre) {
    kout!(g; col(2), b"\xfe ", cn(g));
}

// 6572:0040: bad news.
fn bad(g: &mut Sre) {
    kout!(g; col(1), b"\xfe ", cn(g));
}

// 6572:0080
fn appeased(g: &mut Sre, e: usize, p: usize) {
    let id = g.emp[e].id as usize;
    if g.pir[p].wd4[id] > 0 {
        g.pir[p].wd4[id] -= 1;
    }
}

fn maybe_screen(g: &mut Sre, odds: i16, name: &[u8]) {
    if random(odds) == 0 && show_screen(g, None, name, false) {
        pause_prompt(g);
    }
}

fn add_clamp(v: &mut i32, n: i32) {
    *v = v.wrapping_add(n);
    if *v < 0 {
        *v = 0;
    }
}

fn rand_of(k: i16) -> i16 {
    ((k as i32 * b_rand() as i32) / 0x8000) as i16
}

fn pirate(g: &mut Sre, p: usize) {
    put_pirate(g, KOUT, p);
}

// 6572:00ac: what happens to an empire because of the pirates and
// fate.
fn fate(g: &mut Sre, e: usize, p: usize) {
    let mut lost = [0i16; 10];
    if g.emp[e].turnsused == g.galaxy.grantturns / 2
        && g.emp[e].year > 0
        && (g.emp[e].id == g.galaxy.planetmaster || total_planets(g, e) / 100 > pirate_planets(g, p))
    {
        let mut total: i32 = 0;
        maybe_screen(g, 5, b"Revolt");
        for i in 0..10 {
            let mut count = g.emp[e].planets[i].count;
            if count > 300 {
                count = 300;
            }
            let mut m = ((count.wrapping_mul((g.emp[e].taxrate / 10) as i32)) / 100) as u16;
            if m > 20 {
                m = 20;
            }
            lost[i] = rand_of(m as i16);
            if lost[i] as i32 >= count {
                lost[i] = (count - 1) as i16;
            }
            if lost[i] < 0 {
                lost[i] = 0;
            }
            total += lost[i] as i32;
        }
        if total > 0 {
            for i in 0..10 {
                g.emp[e].planets[i].count -= lost[i] as i32;
            }
            bad(g);
            kout!(g; cnum(g), total, cn(g), " planets revolt under the leadership of ");
            let w = random(40) - 1;
            let mut pir = false;
            match w {
                0 => kout!(g; col(11), "Amit Patel"),
                1 => kout!(g; col(13), "Emperor Reko"),
                2 => kout!(g; col(14), "Bill Clinton"),
                3 => kout!(g; col(9), "Gengher Kahn"),
                4 => kout!(g; col(2), "King Jello"),
                5 => kout!(g; col(3), "a Talking Horse"),
                6 => kout!(g; col(6), "Ziggy the Dinosaur"),
                7 if g.dev.remote != 0 => kout!(g; col(10), cs(&g.system.regto)),
                _ => pir = true,
            }
            if pir {
                pirate(g, p);
                for i in 0..10 {
                    g.pir[p].wb4[i] = g.pir[p].wb4[i].wrapping_add(lost[i]);
                }
                appeased(g, e, p);
            }
            kout!(g; cn(g), ".\n");
        }
    }
    if net_worth(g, e) < pirate_net_worth(g, p) / 20 {
        return;
    }

    if random(5) == 0 && total_planets(g, e) > 60 && g.emp[e].year > 0 {
        let n = (g.emp[e].money.get() / 10000).wrapping_mul(random(100) as i32);
        if n > 0 {
            add_clamp(&mut g.pir[p].money, n);
            g.emp[e].money.sub(n);
            maybe_screen(g, 15, b"Thieves");
            bad(g);
            kout!(g; cnum(g), n, cn(g), " credits were stolen by ");
            pirate(g, p);
            kout!(g; cn(g), "!\n");
        }
    }
    if random(45) == 0 && net_worth(g, e) > 10000 && g.emp[e].year > 0 {
        let n = (g.emp[e].heavycruisers / 100).wrapping_mul(random(10) as i32);
        if n > 0 && g.emp[e].heavycruisers / 5 >= n {
            g.emp[e].heavycruisers -= n;
            add_clamp(&mut g.pir[p].da4, n);
            bad(g);
            kout!(g; cnum(g), n, cn(g), " cruisers were stolen by ");
            pirate(g, p);
            kout!(g; cn(g), ".\n");
        }
    }
    if random(45) == 0 && net_worth(g, e) < 100000 {
        let k = (10i32.wrapping_mul(total_planets(g, e))) as i16;
        let n = rand_of(k) as i32;
        if n > 0 {
            g.emp[e].heavycruisers += n;
            good(g);
            kout!(g; cnum(g), n, cn(g), " cruisers were given to you by Aunt Edna!\n");
        }
    }
    let t = g.emp[e].taxrate as i32;
    if t * t > random(8000) as i32 && g.emp[e].population > 500 && total_planets(g, e) > 0 {
        let k = g.emp[e].taxrate as i16;
        let mut n = (g.emp[e].population / 200).wrapping_mul((k as i32 * b_rand() as i32) / 0x8000);
        let r = 3i32.wrapping_mul(g.emp[e].score) / total_planets(g, e);
        if r > 100 {
            n /= 2;
        }
        if r > 50 {
            n -= n / 5;
        }
        if r > 10 {
            n -= n / 11;
        }
        if r > 1 {
            n -= n / 17;
        }
        if n > 0 {
            let mut violence = false;
            g.emp[e].population -= n;
            if g.emp[e].insurgency < 7 {
                violence = true;
                insurgency_up(g, e);
                if random(3) == 0 {
                    insurgency_up(g, e);
                }
                if random(7) == 0 {
                    insurgency_up(g, e);
                }
            }
            maybe_screen(g, 15, b"HighTax");
            bad(g);
            kout!(g; cnum(g), n, cn(g), " million people were killed in riots due to high tax rates!\n");
            if violence {
                bad(g);
                kout!(g; "Violence increases in your empire!\n");
            }
        }
    }
    if random(30) == 0 && total_planets(g, e) > 30 && g.emp[e].year > 0 {
        let n = (g.emp[e].food / 700).wrapping_mul(random(100) as i32);
        if n > 0 {
            g.emp[e].food -= n;
            bad(g);
            let mut who = random(10) - 1;
            if who == 4 && g.dev.remote == 0 {
                who = -1;
            }
            match who {
                0 => kout!(g; cnum(g), n, cn(g), " megatons of food were eaten by ", col(6), "velociraptors", cn(g), "!\n"),
                1 => kout!(g; col(6), "Crop disease", cn(g), " destroyed ", cnum(g), n, cn(g), " megatons of food!\n"),
                2 => kout!(g; col(10), "Arcadian locusts", cn(g), " destroy ", cnum(g), n, cn(g), " megatons of food!\n"),
                3 => kout!(g; col(3), "Amit Patel", cn(g), " swallowed ", cnum(g), n, cn(g), " megatons of food in one gulp!\n"),
                4 => kout!(g; col(15), "Your hungry Sysop", ", ", cs(&g.system.sysop), b',', cn(g), " ate ", cnum(g), n, cn(g),
                           " megatons of food!\n"),
                _ => {
                    add_clamp(&mut g.pir[p].d94, n);
                    appeased(g, e, p);
                    if random(15) == 0 && show_screen(g, None, b"FoodLost", false) {
                        pause_prompt(g);
                        bad(g);
                    }
                    kout!(g; cnum(g), n, cn(g), " megatons of food were stolen by ");
                    pirate(g, p);
                    kout!(g; cn(g), "!\n");
                }
            }
        }
    }
    if random(1000) <= 50 && g.emp[e].planets[P_FOOD].count > 100 && g.emp[e].year > 0 {
        let n = g.emp[e].planets[P_FOOD].count.wrapping_mul((random(5000) / 3) as i32);
        if n > 0 && g.emp[e].food >= n {
            g.emp[e].food -= n;
            bad(g);
            kout!(g; cnum(g), n, cn(g), " megatons of food were destroyed due to harsh weather.\n");
        }
    }
    if random(1000) <= 30 {
        let n = random(5000) as i32;
        if n > 0 {
            g.emp[e].food += n;
            good(g);
            kout!(g; "You received ", cnum(g), n, cn(g), " megatons of ");
            match random(5) {
                0 => kout!(g; "potatoes and onions"),
                1 => kout!(g; "carrots and peas"),
                2 => kout!(g; "leafy vegetables"),
                3 => kout!(g; "candy bars"),
                4 => kout!(g; "cheese"),
                _ => {}
            }
            kout!(g; " for your birthday.\n");
        }
    }
    if random(1000) <= 30 && g.emp[e].planets[P_PETROLEUM].count > 10 && g.emp[e].year > 0 {
        let k = g.emp[e].planets[P_PETROLEUM].count as i16;
        let n = (((k as i32 * b_rand() as i32) / 0x8000) / 3) as i16 as i32;
        if n > 0 {
            g.emp[e].d188 += n;
            bad(g);
            kout!(g; cnum(g), n, cn(g), " exatons of toxic fumes were released due to a pumping accident.\n");
        }
    }
    if random(1000) <= 10 && g.emp[e].planets[P_PETROLEUM].count > 100 && g.emp[e].year > 0 {
        let k = g.emp[e].planets[P_PETROLEUM].count as i16;
        let n = (((k as i32 * b_rand() as i32) / 0x8000) / 10) as i16 as i32;
        if n > 0 {
            g.emp[e].d188 += n;
            bad(g);
            kout!(g; cnum(g), n, cn(g), " exatons of toxic fumes were released when one of your advisors burped.\n");
        }
    }
    if random(1000) <= 50 && g.emp[e].planets[P_SUPPLY].count > 5 && g.emp[e].year > 0 {
        let n = (g.emp[e].planets[P_SUPPLY].count / 3).wrapping_mul(random(25) as i32);
        if n > 0 && g.emp[e].soldiers >= n {
            g.emp[e].soldiers -= n;
            add_clamp(&mut g.pir[p].d98, n);
            bad(g);
            kout!(g; cnum(g), n, cn(g), " soldiers desert your training camps.\n");
        }
    }
    if random(1000) <= 10 {
        let n = 5i32.wrapping_mul(total_planets(g, e));
        if n > 0 {
            g.emp[e].soldiers += n;
            good(g);
            kout!(g; cnum(g), n, cn(g), " soldiers joined your army.\n");
        }
    }
    if random(1000) <= 50 && g.emp[e].generals > 50 && g.emp[e].year > 0 {
        add_clamp(&mut g.pir[p].dac, 1);
        g.emp[e].generals -= 1;
        bad(g);
        kout!(g; cnum(g), 1i16, cn(g), " general was captured by ");
        pirate(g, p);
        kout!(g; cn(g), ".\n");
    }
    if random(1000) <= 10 && g.emp[e].generals > 50 && g.emp[e].year > 0 {
        g.emp[e].generals -= 1;
        bad(g);
        kout!(g; cnum(g), 1i16, cn(g), " general lost a game of Solar Hangman, and was hanged.\n");
    }
    if random(1000) <= 10 && g.emp[e].generals > 50 && g.emp[e].year > 0 {
        g.emp[e].generals -= 1;
        bad(g);
        kout!(g; cnum(g), 1i16, cn(g), " general was lost bungee-jumping off the cliffs of Raekel IV.\n");
    }
    if random(1000) <= 30 && g.emp[e].agents > 500 && g.emp[e].year > 0 {
        let k: i16 = if g.emp[e].agents > 3000 { 200 } else { (g.emp[e].agents / 20) as i16 };
        let n = ((k as i32 * b_rand() as i32) / 0x8000) as u16;
        if n > 0 {
            add_clamp(&mut g.pir[p].db0, n as i32);
            g.emp[e].agents -= n as i32;
            bad(g);
            kout!(g; cnum(g), n, cn(g), " covert agents were captured by ");
            pirate(g, p);
            kout!(g; cn(g), ".\n");
            appeased(g, e, p);
        }
    }
    if random(1000) <= 50 && g.emp[e].soldiers > 1000 && g.emp[e].year > 0 {
        let k = (g.emp[e].soldiers / 20) as i16;
        let n = rand_of(k) as i32;
        if n > 0 {
            add_clamp(&mut g.pir[p].d98, n);
            g.emp[e].soldiers -= n;
            bad(g);
            kout!(g; cnum(g), n, cn(g), " soldiers have been forced to join the ");
            pirate(g, p);
            kout!(g; cn(g), " force.\n");
            appeased(g, e, p);
        }
    }
    if random(90) == 0 && total_planets(g, e) > 90 && g.emp[e].year > 0 {
        let mut total: i32 = 0;
        for i in 0..10 {
            let k = (g.emp[e].planets[i].count / 179) as i16;
            let mut x = rand_of(k);
            if x < 0 {
                x = 0;
            }
            if x as i32 > g.emp[e].planets[i].count / 100 {
                let c = g.emp[e].planets[i].count;
                out!(g, ERRLOG; "random() - planets lost @14:  ", x, " > ", c, " [", i as i16, "]\n");
                x = 0;
            }
            total += x as i32;
            g.emp[e].planets[i].count -= x as i32;
            lost[i] = x;
        }
        if total > 0 {
            maybe_screen(g, 15, b"Planets");
            bad(g);
            kout!(g; cnum(g), total, cn(g), " planet", if total == 1 { " was " } else { "s were " });
            match random(19) {
                0 => kout!(g; "devastated by a ", col(3), "tear in the fabric of space-time"),
                1 => kout!(g; "destroyed in a ", col(11), "Supernova Explosion"),
                2 => kout!(g; "swallowed by a ", col(2), "hungry sysop"),
                3 => kout!(g; "lost in a ", col(3), "Nebular Collapse"),
                4 => kout!(g; "lost in a ", col(4), "BBS Crash"),
                5 => kout!(g; "sucked into a ", col(8), "black hole"),
                6 => kout!(g; "destroyed in ", col(12), "inter-planetary warfare"),
                7 => kout!(g; "eaten by a ", col(13), "One-Eyed Purple Planet Eater"),
                8 => kout!(g; "towed away by a ", col(10), "Federation Starship"),
                _ => {
                    kout!(g; "captured by ");
                    pirate(g, p);
                    for i in 0..10 {
                        g.pir[p].wb4[i] = g.pir[p].wb4[i].wrapping_add(lost[i]);
                    }
                    appeased(g, e, p);
                }
            }
            kout!(g; cn(g), "!\n");
        }
    }
}

// 6572:230c: a pirate gang, chosen by how much it hates the empire.
fn choose_pirate(g: &Sre, id: i16) -> i16 {
    let mut sum: i32 = 0;
    for k in 1..=9i32 {
        let h = g.pir[(k - 1) as usize].wd4[id as usize] as i32;
        if h > 0 {
            sum += h * k;
        }
    }
    let hi = ((sum >> 15) + 1) as i16;
    let a = rand_of(hi);
    let lo = ((sum & 0x7fff) + 1) as i16;
    let b = rand_of(lo);
    let mut r = ((((a as i32) << 15) as u16) | (b as u16)) as i16 as i32;
    for k in 1..=9i32 {
        let h = g.pir[(k - 1) as usize].wd4[id as usize] as i32;
        if h > 0 {
            r -= h * k;
        }
        if r <= 0 {
            return k as i16;
        }
    }
    1
}

// A weapon for sale on the underground market.
fn offer(g: &mut Sre, e: usize, price: i32, buyprompt: &[u8], bought: &str, cannot: &str, bit: u16) {
    if g.emp[e].money.get() >= price || g.emp[e].money.get().wrapping_add(g.emp[e].bank.get()) >= price {
        if !yes_no(g, 2, buyprompt) {
            return;
        }
        if g.emp[e].bank.get() >= price && yes_no(g, 1, b"Withdraw funds from the bank for this purchase") {
            g.emp[e].money.add(price);
            g.emp[e].bank.sub(price);
        }
        kout!(g; bought);
        let x = &mut g.emp[e];
        x.weapons |= bit;
        if x.money.get() >= price {
            x.money.sub(price);
        } else {
            let m = x.money.get();
            x.bank.sub(price.wrapping_sub(m));
            x.money.set(0);
        }
    } else {
        kout!(g; cannot);
    }
}

// 6572:2459
pub fn advisor(g: &mut Sre, _p: &mut Player, e: usize) {
    if pirate_count(g) > 0 {
        let k = choose_pirate(g, g.emp[e].id);
        fate(g, e, (k - 1) as usize);
    }
    if random(3) == 0 {
        let count = research_count(g);
        let idx = rand_of(count);
        let mut d = Development::default();
        research_load(g, &mut d, idx);
        if d.typ < 0 || d.typ > 9 {
            let rc = research_count(g);
            out!(g, ERRLOG; "R.index( ", idx, " ) in random() is ", d.typ, "/", rc, ".\n");
        } else {
            let t = d.typ as usize;
            if d.cost.wrapping_mul(g.emp[e].planets[t].count) <= g.emp[e].d293
                && g.emp[e].planets[t].count > 0
                && random(2) == 0
            {
                let ps = &mut g.emp[e].planets[t];
                ps.a = (ps.a as i16).wrapping_add(d.permanent() as i16) as i32;
                ps.b = (ps.b as i16).wrapping_add(d.temporary() as i16) as i32;
                let c = d.cost.wrapping_mul(g.emp[e].planets[t].count);
                g.emp[e].d293 -= c;
                kout!(g; b'\n');
                d.show(g);
            }
        }
    } else if g.emp[e].d293 >= 10000 && g.emp[e].weapons & 8 == 0 {
        g.emp[e].weapons |= 8;
        kout!(g; col(14), "Your research planets develop a Mothership carrier capable of carrying 100,000 fighters into battle.\n",
              cn(g));
    }
    if gov_capacity6(g, e) < 0 || gov_capacity(g, e) < 0 {
        let mut agents = gov_capacity(g, e).wrapping_neg();
        let mut generals = gov_capacity6(g, e).wrapping_neg();
        if agents < 0 {
            agents = 0;
        }
        if g.emp[e].agents < agents {
            agents = g.emp[e].agents;
        }
        if generals < 0 {
            generals = 0;
        }
        if g.emp[e].generals < generals {
            generals = g.emp[e].generals;
        }
        if agents > 2 {
            agents /= 3;
        }
        generals /= 3;
        g.emp[e].agents -= agents;
        g.emp[e].generals -= generals;
        if agents > 0 {
            kout!(g; cnum(g), agents, cn(g), " covert agents are killed in covert civil wars caused by spy overpopulation.\n");
        }
        if generals > 0 {
            kout!(g; cnum(g), generals, cn(g),
                  " generals kill each other in military conflicts caused by overpopulation of generals.\n");
        }
    }
    if g.system.regstate != 0 && g.grb(&R_NUCLEAR) {
        let v = (2i32.wrapping_mul(g.emp[e].agents)) / (total_planets(g, e) + 200);
        if v > random(20) as i32 && g.emp[e].weapons & 1 == 0 && g.emp[e].year > 0 {
            maybe_screen(g, 3, b"Nuclear");
            kout!(g; "You are approached by ");
            match random(5) {
                0 => kout!(g; "a sleazy trader"),
                1 => kout!(g; "Bauk'ar de Gorash,"),
                2 => kout!(g; "Torras Ysarr,"),
                3 => kout!(g; "a well dressed trader"),
                4 => kout!(g; "Diku Mudd"),
                _ => {}
            }
            kout!(g; " who offers you a ", col(14), "Nuclear Bomb", cn(g), "!\n");
            offer(g, e, 3000000, b"Buy a nuclear bomb for 3,000,000 credits",
                  "You have bought one nuclear bomb from the underground trading market.\n",
                  "But you can't afford it right now.\n", 1);
        }
    }
    if g.system.regstate != 0 && g.grb(&R_CHEMICAL) {
        let v = (2i32.wrapping_mul(g.emp[e].agents)) / (total_planets(g, e) + 200);
        if v > random(20) as i32 && g.emp[e].weapons & 2 == 0 && g.emp[e].year > 0 {
            maybe_screen(g, 3, b"Chemical");
            kout!(g; "A well-dressed trader approaches you and offers to sell a megaton of ", col(14), "Chemicals", cn(g),
                  " for chemical warfare.\n");
            offer(g, e, 1000000, b"Buy a barrel of chemicals for 1,000,000 credits",
                  "You have bought one barrel of chemicals from the underground trading market.\n",
                  "But you can't afford it.\n", 2);
        }
    }
    if g.system.regstate != 0 && g.grb(&R_PSIONIC) {
        let v = (4i32.wrapping_mul(g.emp[e].agents)) / (total_planets(g, e) + 200);
        if v > random(20) as i32 && g.emp[e].weapons & 4 == 0 && g.emp[e].year > 0 {
            let price = g.galaxy.d3c2;
            maybe_screen(g, 3, b"Psionic");
            kout!(g; "A well-known smuggler approaches you and offers to sell a one-pound ", col(14), "Psionic Bomb", cn(g),
                  " to you.\n");
            if g.emp[e].money.get() >= price || g.emp[e].money.get().wrapping_add(g.emp[e].bank.get()) >= price {
                kout!(g; "Buy a psionic bomb for ", price, " credits");
                if yes_no(g, 2, b"") {
                    if g.emp[e].bank.get() >= price && yes_no(g, 1, b"Withdraw funds from the bank for this purchase") {
                        g.emp[e].money.add(price);
                        g.emp[e].bank.sub(price);
                    }
                    kout!(g; "You have bought a psionic bomb from the smuggler.\n");
                    let x = &mut g.emp[e];
                    x.weapons |= 4;
                    if x.money.get() >= price {
                        x.money.sub(price);
                    } else {
                        let m = x.money.get();
                        x.bank.sub(price.wrapping_sub(m));
                        x.money.set(0);
                    }
                    g.galaxy.d3c2 = g.galaxy.d3c2.wrapping_add(2i32.wrapping_mul(g.galaxy.d3c2).wrapping_add(1000000));
                }
            } else {
                kout!(g; "You tell him that you cannot afford it.\n");
            }
        }
    }
}
