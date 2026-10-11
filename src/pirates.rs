// Pirates (segment 4b60).

use crate::prelude::*;

// 4b60:0000 (constructor) + 4b60:0090
pub fn pirate_new(g: &mut Sre, p: usize) {
    g.pir[p].name = [0; 0x20];
    g.pir[p].level = 0;
    pirate_init(g, p);
}

// 4b60:0090
pub fn pirate_init(g: &mut Sre, p: usize) {
    let now = g.time.now;
    let x = &mut g.pir[p];
    x.lasthour = now;
    x.money = 0;
    x.food = 0;
    x.soldiers = 0;
    x.fighters = 0;
    x.stations = 0;
    x.heavycruisers = 0;
    x.carriers = 0;
    x.generals = 0;
    x.agents = 0;
    x.color = 7;
    x.planets = [0; 16];
    x.hatred = [0; 25];
    x.u_026 = [0; 0x69];
}

// 4b60:01ab
pub fn pirate_set_name(g: &mut Sre, p: usize, s: &[u8]) {
    let s = cstr(s);
    let mut i = 0;
    while i < s.len() && i < 0x1f {
        g.pir[p].name[i] = s[i];
        i += 1;
    }
    g.pir[p].name[i] = 0;
}

// 4b60:01ed
pub fn pirate_planets(g: &Sre, p: usize) -> i32 {
    let mut n: i32 = 0;
    for &w in g.pir[p].planets.iter() {
        n += w as i32;
    }
    n
}

// 4b60:022f
pub fn pirate_net_worth(g: &Sre, p: usize) -> i32 {
    let x = &g.pir[p];
    10 * pirate_planets(g, p) + x.money / 100000 + 7 * x.agents + x.soldiers + x.fighters + x.food / 10000 + 3 * x.heavycruisers
        + (x.stations << 1)
        + (x.carriers << 1)
        + 5 * x.generals
}

// 4b60:034a: pirates rebuild once an hour.
pub fn pirate_hourly(g: &mut Sre, p: usize) {
    let now = b_time();
    if now.wrapping_sub(g.pir[p].lasthour) < 3600 {
        return;
    }
    g.pir[p].lasthour = now;
    let lv = g.pir[p].level as i32;
    let cap = |m: i32| (lv * m) as i16 as i32;
    let planets = |g: &Sre, i: usize| g.pir[p].planets[i] as i32;
    if cap(3000) > g.pir[p].soldiers {
        let v = random(100) as i32 + (5 * planets(g, P_SUPPLY)) / 10;
        g.pir[p].soldiers += v;
    }
    if cap(3000) > g.pir[p].fighters {
        let v = random(100) as i32 + (4 * planets(g, P_SUPPLY)) / 10;
        g.pir[p].fighters += v;
    }
    if cap(3000) > g.pir[p].heavycruisers {
        let v = random(100) as i32 + planets(g, P_SUPPLY) / 10;
        g.pir[p].heavycruisers += v;
    }
    if cap(3000) > g.pir[p].stations {
        let v = random(100) as i32 + (4 * planets(g, P_SUPPLY)) / 10;
        g.pir[p].stations += v;
    }
    if cap(300) > g.pir[p].carriers {
        let v = random(10) as i32 + (2 * planets(g, P_SUPPLY)) / 10;
        g.pir[p].carriers += v;
    }
    if cap(300) > g.pir[p].generals {
        let v = random(10) as i32 + (2 * planets(g, P_SUPPLY)) / 10;
        g.pir[p].generals += v;
    }
    if cap(500) > g.pir[p].agents {
        let v = random(10) as i32;
        g.pir[p].agents += v;
    }
    if 10000000i32.wrapping_mul(lv) > g.pir[p].food {
        let v = random(30) as i32 + 6 * planets(g, P_FOOD);
        g.pir[p].food += v;
    }
    if 10000000i32.wrapping_mul(lv) > g.pir[p].money {
        let x = &mut g.pir[p];
        x.money += 10000 * lv;
        x.money += 230 * x.planets[P_PETROLEUM] as i32;
        x.money += 50 * x.planets[P_EDUCATION] as i32;
        x.money += 260 * x.planets[P_TOURISM] as i32;
        x.money += 210 * x.planets[P_ORE] as i32;
        x.money += 70 * x.planets[P_URBAN] as i32;
    }
    if 10000 * lv > g.pir[p].agents {
        let v = (g.pir[p].planets[P_GOVERNMENT] as i32) << 1;
        g.pir[p].agents += v;
    }
}

const ADJ: [&[u8]; 19] = [
    b"Arcadian", b"Galactic", b"Extraterrestrial", b"Intergalactic", b"Infinite", b"SRE", b"Solar", b"Space", b"Storm",
    b"Xandorian", b"Freedom", b"Planetary", b"Doomed", b"Compiler", b"Nemesis", b"Uncertain", b"Empire", b"Gooey",
    b"Runtime",
];
const NOUN: [&[u8]; 17] = [
    b"Gangs", b"Invaders", b"Looters", b"Marauders", b"Raiders", b"Twins", b"Thieves", b"Thugs", b"Babies", b"Pirates",
    b"Dieters", b"Bugs", b"Blobs", b"Nimwits", b"Warhogs", b"Sharks", b"Rogues",
];
const COLORS: [i16; 9] = [10, 15, 3, 11, 9, 13, 12, 14, 6];

fn pirate_key(level: i16, what: &[u8]) -> Str {
    cat(&[b"Pirate", &[(level as u8).wrapping_add(b'0')], what])
}

// 4b60:088b: name the pirate gangs if they have not been set up.
pub fn create_pirates(g: &mut Sre) {
    let p0 = &g.pir[0];
    let need = !(p0.level > 0 && p0.name[0] != 0 && p0.color > 0 && p0.color <= 15);
    b_srand(b_time() as u16);
    if !need {
        return;
    }
    let (na, nb): (i16, i16) = (19, 17);
    let mut a = random(na);
    let mut b = random(nb);
    let mut sa;
    loop {
        sa = random(na - 1) + 1;
        if na % sa != 0 {
            break;
        }
    }
    let mut sb;
    loop {
        sb = random(nb - 1) + 1;
        if nb % sb != 0 {
            break;
        }
    }
    for i in 1..=9i16 {
        let p = (i - 1) as usize;
        pirate_init(g, p);
        g.pir[p].level = i;
        let mut rn = Resource::text(&pirate_key(i, b".Name"), Some(b""));
        let r = rn.str(&mut g.resdb);
        let name = if !r.is_empty() { r } else { cat(&[ADJ[a as usize], b" ", NOUN[b as usize]]) };
        pirate_set_name(g, p, &name);
        g.pir[p].color = COLORS[p] as u8;
        let mut rc = Resource::num(&pirate_key(i, b".Color"), 0);
        if rc.long(&mut g.resdb) > 0 {
            g.pir[p].color = rc.long(&mut g.resdb) as u8;
        }
        a = (a + sa) % na;
        b = (b + sb) % nb;
    }
}

// 4b60:0bdf
pub fn pirate_count(g: &Sre) -> i16 {
    let mut n = 0;
    for p in g.pir.iter() {
        if p.level > 0 && p.name[0] != 0 {
            n += 1;
        }
    }
    n
}

// 4b60:0c38
pub fn pirate_name(g: &mut Sre, p: usize) -> Str {
    let mut r = Resource::text(&pirate_key(g.pir[p].level, b".Name"), Some(b""));
    let s = r.str(&mut g.resdb);
    if !s.is_empty() {
        return s;
    }
    cs(&g.pir[p].name)
}

// 4b60:0ced
pub fn pirate_color(g: &mut Sre, p: usize) -> u8 {
    let mut r = Resource::num(&pirate_key(g.pir[p].level, b".Color"), 0);
    if r.long(&mut g.resdb) > 0 {
        return r.long(&mut g.resdb) as u8;
    }
    g.pir[p].color
}

// 4b60:0d87
pub fn put_pirate(g: &mut Sre, o: Sid, p: usize) {
    let c = pirate_color(g, p) as i16;
    out!(g, o; col(c));
    let n = pirate_name(g, p);
    put_untranslated(g, o, &n);
    out!(g, o; cn(g));
}
