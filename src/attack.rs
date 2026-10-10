// Attacks: the attack menu (segment 5090), conventional invasions (3b7b),
// guerilla ambushes (545e) and nuclear, chemical and psionic weapons
// (5c5c).

use crate::battle::*;
use crate::diplomacy::treaty_name;
use crate::player::*;
use crate::prelude::*;

static R_MAXATTACKS: GRes = GRes::num(b"Attacks.Maximum", -1); // 384a
static R_CONVENTIONAL: GRes = GRes::num(b"Attacks.Conventional", 1); // 3856
static R_GUERILLA: GRes = GRes::num(b"Attacks.Guerilla", 1); // 3862
static R_PIRATERAID: GRes = GRes::num(b"Attacks.PirateRaid", 1); // 386e
static R_PIRATESPY: GRes = GRes::num(b"Attacks.PirateSpy", 1); // 387a

fn clamp100b(v: i16) -> i8 {
    let mut b = v as i8;
    if b < 0 {
        b = 0;
    }
    if b > 100 {
        b = 100;
    }
    b
}

// The treaty an attack breaks (SetTreaty inlined, and the state reset).
fn break_treaty(t: &mut Treaty) {
    set_treaty(t, 0, 0);
    t.setc(0);
}

fn break_treaties(g: &mut Sre, a: usize, b: usize) {
    let (aid, bid) = (g.emp[a].id as usize, g.emp[b].id as usize);
    break_treaty(&mut g.emp[a].treaty[bid]);
    break_treaty(&mut g.emp[b].treaty[aid]);
}

// 5090:0000: attacking ends the attacker's protection.
fn void_protection(g: &mut Sre, e: usize) -> bool {
    if g.emp[e].year >= 0 {
        return true;
    }
    let what = if g.emp[e].stations < 200 {
        Some("defense")
    } else if g.emp[e].soldiers < 200 {
        Some("soldiers")
    } else {
        None
    };
    if let Some(w) = what {
        kout!(g; "You do not have enough ", w, " to be out of protection yet.\n");
        return false;
    }
    if total_planets(g, e) < 200 {
        kout!(g; "Your empire is too small", " to be out of protection yet.\n");
        return false;
    }
    if !yes_no(g, 0, b"Are you sure you would like to void your protection by attacking") {
        return false;
    }
    let x = &mut g.emp[e];
    if x.year < 0 {
        x.year = 0;
        x.score -= x.year as i32;
        if x.score < 0 {
            x.score = 0;
        }
    }
    true
}

// 5c5c:0000
fn can_attack(g: &mut Sre, me: usize, t: usize) -> bool {
    if g.emp[t].year < 0 {
        kout!(g; "You can not attack ", cnum(g), uname(g, t), cn(g), " because that empire is under protection!\n");
        return false;
    }
    let tr = g.emp[me].treaty[g.emp[t].id as usize];
    if tr.typ() != 0 {
        kout!(g; "You have a ", col(10), treaty_name(tr.typ()), cn(g));
        kout!(g; " with ", cnum(g), uname(g, t), cn(g), ".\n");
        if tr.typ() == 1 {
            kout!(g; "Your Neutrality Treaty prevents you from attacking.\n");
            return false;
        }
        if tr.a != 0 {
            kout!(g; "You cannot attack your allies unless you first break the treaty from the Diplomacy Menu.\n");
            return false;
        }
        if !yes_no(g, 0, b"Do you wish to continue") {
            return false;
        }
    }
    if !g.emp[t].exists() || total_planets(g, t) <= 0 || g.emp[t].population <= 0 {
        kout!(g; "That empire has collapsed!\n");
        return false;
    }
    if !void_protection(g, me) {
        return false;
    }
    break_treaties(g, me, t);
    true
}

// 3b7b:0000
fn lose(v: &mut i32, n: i32) {
    *v = v.wrapping_sub(n);
    if *v < 0 {
        *v = 0;
    }
}

// 3b7b:0035: the first of the forces that is left takes the damage.
#[allow(clippy::too_many_arguments)]
fn hit(a: &mut i32, da: i32, b: &mut i32, db: i32, c: &mut i32, dc: i32, d: &mut i32, dd: i32) {
    if *a != 0 {
        lose(a, da);
    } else if *b != 0 {
        lose(b, db);
    } else if *c != 0 {
        lose(c, dc);
    } else {
        lose(d, dd);
    }
}

// 3b7b:00a9: b * a / c, reduced to stay within range.
fn proportion(mut a: i32, mut b: i32, mut c: i32) -> i32 {
    while a > 10000 && c > 10000 {
        a /= 10;
        c /= 10;
    }
    while b > 10000 && c > 10000 {
        b /= 10;
        c /= 10;
    }
    for k in 2..=7 {
        while a % k == 0 && c % k == 0 {
            a /= k;
            c /= k;
        }
        while b % k == 0 && c % k == 0 {
            b /= k;
            c /= k;
        }
    }
    b.wrapping_mul(a) / c
}

// 3b7b:023b
fn show_victory(g: &mut Sre, c: Color, credits: i32, planets: &ItemList) {
    if random(5) == 0 && show_screen(g, None, b"WinWar", false) {
        pause_prompt(g);
    }
    kout!(g; "You have won the battle!\n\n");
    kout!(g; "You capture ", c, credits, cn(g), " credits");
    if planets.count() > 0 {
        kout!(g; " and ");
        put_item_list(g, KOUT, planets);
        kout!(g; " planets");
    }
    kout!(g; ".\n\n");
}

// 3b7b:0325
fn conventional(g: &mut Sre, me: usize, t: usize) -> bool {
    let heavy = g.emp[me].heavycruisers;
    let light = g.emp[me].lightcruisers;
    let mut stations = g.emp[t].stations; // defense stations fight like fighters
    let mut tlight = g.emp[t].lightcruisers;
    let mut score: i16 = 0;
    if !can_attack(g, me, t) {
        return false;
    }
    // The lists kept between attacks (3b7b:2aa9, DS:063e..066e).
    let mut s_forces = ItemList::new(col(7), col(11));
    let x = &g.emp[me];
    s_forces.add(x.fighters, b"fighter");
    s_forces.add(x.carriers, b"carrier");
    s_forces.add(x.soldiers, b"soldier");
    s_forces.add(x.generals, b"general");
    s_forces.add(x.heavycruisers, b"heavy cruiser");
    s_forces.add(x.lightcruisers, b"light cruiser");
    if (x.weapons >> 3) & 1 != 0 {
        s_forces.add(1, b"mothership");
    }
    kout!(g; "You have ");
    put_item_list(g, KOUT, &s_forces);
    kout!(g; b'.');
    // (Any carrier or mothership lets up to 100,000 fighters fly.)
    let x = &g.emp[me];
    let mut fcap: i32 = if 100i32.wrapping_mul(x.carriers) + ((x.weapons >> 3) & 1) as i32 != 0 { 100000 } else { 0 };
    let scap: i32 = if 50i32.wrapping_mul(x.generals) <= x.soldiers { 50i32.wrapping_mul(x.generals) } else { x.soldiers };
    fcap = if x.fighters < fcap { x.fighters } else { fcap };
    let nogen = x.soldiers > scap;
    let nocarrier = x.fighters > fcap;
    if nogen || nocarrier {
        kout!(g; "  You do not have enough");
    }
    if nogen {
        kout!(g; " generals");
    }
    if nogen && nocarrier {
        kout!(g; " and");
    }
    if nocarrier {
        kout!(g; " carriers");
    }
    if nogen || nocarrier {
        kout!(g; " to lead all your forces into battle.");
    }
    kout!(g; b'\n');
    kout!(g; "Take how many soldiers?  ");
    set_defval(g, KIN, scap);
    set_maxval(g, KIN, scap);
    let mut sa = in_i32(g, KIN);
    kout!(g; "Take how many fighters?  ");
    set_defval(g, KIN, fcap);
    set_maxval(g, KIN, fcap);
    let mut fa = in_i32(g, KIN);
    kout!(g; "Take how many light cruisers?  ");
    set_defval(g, KIN, light);
    set_maxval(g, KIN, light);
    let mut la = in_i32(g, KIN);
    kout!(g; "Take how many heavy cruisers?  ");
    set_defval(g, KIN, heavy);
    set_maxval(g, KIN, heavy);
    let mut ha = in_i32(g, KIN);
    let (sentf, sents, senth, sentl) = (fa, sa, ha, la);
    if sa.wrapping_add(fa).wrapping_add(ha).wrapping_add(la) <= 0 {
        kout!(g; col(12), "ABORTED", cn(g), b'\n');
        return false;
    }
    break_treaties(g, me, t);
    // Allies of the defender send some of their forces.
    let mut allys = [0i32; 25];
    let mut allyh = [0i32; 25];
    let mut sd: i32 = 0;
    let mut hd: i32 = 0;
    let tid = g.emp[t].id as usize;
    for i in 0..25 {
        allyh[i] = 0;
        allys[i] = 0;
        let a = &g.emp[i];
        if !a.exists() {
            continue;
        }
        if a.treaty[tid].c() != 0 {
            continue;
        }
        allys[i] = (a.treaty[tid].e() as i32).wrapping_mul(a.soldiers / 100);
        allyh[i] = (a.treaty[tid].d() as i32).wrapping_mul(a.heavycruisers / 100);
        sd = sd.wrapping_add(allys[i]);
        hd = hd.wrapping_add(allyh[i]);
    }
    if sd > 0 || hd > 0 {
        kout!(g; cnum(g), uname(g, t), cn(g), "'s allies send ", cnum(g), sd, cn(g), " soldiers and ", cnum(g), hd, cn(g),
              " heavy cruisers.  ");
    }
    let drafted = (g.emp[t].draftrate as i32).wrapping_mul(g.emp[t].population / 100);
    g.emp[t].population -= drafted / 10;
    sd = sd.wrapping_add(drafted);
    if drafted > 0 {
        kout!(g; cnum(g), drafted, cn(g), " soldiers are drafted.  ");
    }
    kout!(g; b'\n');
    sd = sd.wrapping_add(g.emp[t].soldiers);
    let sd0 = sd;
    hd = hd.wrapping_add(g.emp[t].heavycruisers);
    let hd0 = hd;
    kout!(g; col(12));
    let ds = (sa + sd) / 50 + 1;
    let df = (fa + stations) / 50 + 1;
    let dh = (ha + hd) / 50 + 1;
    let dl = (la + tlight) / 50 + 1;
    let qa = (sa + fa + ha + la) / 10;
    let qd = (sd + stations + hd + tlight) / 10;
    let mut round: i16 = 0;
    while sa + fa + ha + la > qa && sd + stations + hd + tlight > qd {
        if round >= 3 && random(200) < g.emp[t].effectiveness as i16 + 90 {
            hit(&mut sa, ds, &mut ha, dh, &mut fa, df, &mut la, dl);
        }
        if round >= 3 && random(200) < g.emp[me].effectiveness as i16 + 90 {
            hit(&mut sd, ds, &mut hd, dh, &mut stations, df, &mut tlight, dl);
        }
        if round >= 2 && random(200) < 100 {
            hit(&mut fa, df, &mut sa, ds, &mut ha, dh, &mut la, dl);
        }
        if round >= 2 && random(200) < 100 {
            hit(&mut stations, df, &mut sd, ds, &mut hd, dh, &mut tlight, dl);
        }
        // (The original mixes in the defender's light cruisers here.)
        if round >= 5 && random(200) < g.emp[t].cmdship as i16 / 2 + 50 {
            hit(&mut ha, dh, &mut tlight, dl, &mut fa, df, &mut sa, ds);
        }
        if round >= 5 && random(200) < g.emp[me].cmdship as i16 / 2 + 50 {
            hit(&mut hd, dh, &mut tlight, dl, &mut stations, df, &mut sd, ds);
        }
        if round >= 0 && random(200) < 100 {
            hit(&mut la, dl, &mut ha, dh, &mut fa, df, &mut sa, ds);
        }
        if round >= 0 && random(200) < 100 {
            hit(&mut tlight, dl, &mut hd, dh, &mut stations, df, &mut sd, ds);
        }
        kout!(g; b'.');
        let r = round;
        round += 1;
        if r > 0x4c {
            break;
        }
    }
    kout!(g; cn(g), b'\n');
    // Score each kind of force.
    let f = [(sa, sd), (fa, stations), (ha, hd), (la, tlight)];
    for (a, d) in f {
        if a > 0 {
            score += 1;
        }
        if 2 * d < a {
            score += 1;
        }
        if a < 0 {
            score -= 1;
        }
        if d > 0 {
            score -= 1;
        }
        if 2 * a < d {
            score -= 1;
        }
        if d < 0 {
            score += 1;
        }
    }
    let cc = col(14);
    let mut s_captured = ItemList::new(col(7), col(14));
    s_captured.skip_zero();
    let mut won = false;
    let mut credits: i32 = 0;
    if score > 0 {
        won = true;
        let mut total: i32 = 0;
        score *= 2;
        if net_worth(g, me) > net_worth(g, t) && total_planets(g, me) > total_planets(g, t) {
            score = score / 3 + 1;
        }
        let mut div: i16 = 20 / score + 1;
        if div < 3 {
            div = 3;
        }
        let mut take = [0i16; 10];
        for i in 0..10 {
            let ps = &mut g.emp[t].planets[i];
            if ps.count < 0 {
                ps.count = 0;
            }
            take[i] = (ps.count / div as i32) as i16;
            if take[i] == 0 && ps.count > 0 {
                take[i] = 1;
            }
            if take[i] == 1 && ps.count > 1 {
                take[i] = 2;
            }
            if take[i] as i32 > ps.count {
                out!(g, ERRLOG; "Player::ConventionalAttack() - pl/planets incorrect assignment.\n");
            }
            total += take[i] as i32;
        }
        while total_planets(g, t) / div as i32 > total && total_planets(g, t) > total {
            let k = random(10) as usize;
            if g.emp[t].planets[k].count > take[k] as i32 {
                take[k] += 1;
                total += 1;
            }
        }
        credits = g.emp[t].money.get() / div as i32;
        g.emp[me].effectiveness = clamp100b(g.emp[me].effectiveness as i16 - 13);
        g.emp[me].cmdship = clamp100b(g.emp[me].cmdship as i16 - 7);
        g.emp[t].effectiveness = clamp100b(g.emp[t].effectiveness as i16 - 3);
        let r = (random(5) + 2) as i8;
        g.emp[t].cmdship = clamp100b(g.emp[t].cmdship as i16 - r as i16);
        for i in 0..10 {
            g.emp[me].planets[i].count += take[i] as i32;
            g.emp[t].planets[i].count -= take[i] as i32;
        }
        g.emp[me].money.add(credits);
        let tscore = g.emp[t].score;
        g.emp[me].score = g.emp[me].score.wrapping_add(17 * total + tscore.wrapping_mul(score as i32) / 10);
        g.emp[t].money.sub(credits);
        let tscore = g.emp[t].score;
        g.emp[t].score = tscore.wrapping_sub(7 * total + tscore.wrapping_mul(score as i32) / 20);
        if g.emp[t].score < 0 {
            g.emp[t].score = 0;
        }
        if total_planets(g, t) <= 0 {
            let ts = g.emp[t].score;
            g.emp[me].score = g.emp[me].score.wrapping_add(ts);
            g.emp[t].score = 0;
        }
        for i in 0..10 {
            if take[i] > 0 {
                s_captured.add_plural(take[i] as i32, planet_name(i as i8), b"");
            }
        }
    } else {
        let r1 = random(7);
        let r2 = random(3);
        let r = (r1 + r2 + 3) as i8;
        g.emp[me].cmdship = clamp100b(g.emp[me].cmdship as i16 - r as i16);
        let bonus = (score << 1) as i8;
        g.emp[me].effectiveness = clamp100b(g.emp[me].effectiveness as i16 + bonus as i16);
        g.emp[t].cmdship = clamp100b(g.emp[t].cmdship as i16 - 5);
        g.emp[t].effectiveness = clamp100b(g.emp[t].effectiveness as i16 - 4);
        let v = random(100) as i32 + 110;
        g.emp[me].score -= v;
        if g.emp[me].score < 0 {
            g.emp[me].score = 0;
        }
    }
    attack_news(g, me, t, score > 0, score);
    // A third of the destroyed forces were really lost.
    let lf = (sentf - fa) / 3;
    let ls = (sents - sa) / 3;
    let lh = (senth - ha) / 3;
    let ll = (sentl - la) / 3;
    let lg = (ls + 50) / 200;
    let mut lc = (lf + 200) / 1000;
    {
        let x = &mut g.emp[me];
        if x.carriers < lc {
            lc = x.carriers;
        }
        x.fighters -= lf;
        x.heavycruisers -= lh;
        x.lightcruisers -= ll;
        x.soldiers -= ls;
        x.carriers -= lc;
        x.generals -= lg;
    }
    let mut s_lost = ItemList::new(col(7), col(12));
    s_lost.skip_zero();
    s_lost.add(ls, b"soldier");
    s_lost.add(lf, b"fighter");
    s_lost.add(ll, b"light cruiser");
    s_lost.add(lh, b"heavy cruiser");
    s_lost.add(lg, b"general");
    s_lost.add(lc, b"carrier");
    let kst = (g.emp[t].stations - stations) / 3;
    let ks = (sd0 - sd) / 3;
    let kh = (hd0 - hd) / 3;
    let kl = (g.emp[t].lightcruisers - tlight) / 3;
    let mut s_killed = ItemList::new(col(7), col(11));
    s_killed.skip_zero();
    s_killed.add(ks, b"soldier");
    s_killed.add(kst, b"defense station");
    s_killed.add(kl, b"light cruiser");
    s_killed.add(kh, b"heavy cruiser");
    for i in 0..25usize {
        if g.emp[t].id == i as i16 {
            continue;
        }
        if allys[i] <= 0 && allyh[i] <= 0 {
            continue;
        }
        if !g.emp[i].exists() {
            continue;
        }
        let mut als: i32 = 0;
        let mut alh: i32 = 0;
        if sd0 > 0 && allys[i] > 0 {
            als = proportion(ks, allys[i], sd0);
        }
        if hd0 > 0 && allyh[i] > 0 {
            alh = proportion(kh, allyh[i], hd0);
        }
        if score < 0 && g.emp[i].effectiveness > 0 {
            g.emp[i].effectiveness = clamp100b(g.emp[i].effectiveness as i16 - 1);
        }
        g.emp[i].soldiers -= als;
        g.emp[i].heavycruisers -= alh;
        if score < 0 {
            let v = g.emp[t].score / 30;
            g.emp[i].score -= v;
        }
        if score > 0 {
            let v = g.emp[me].score / 40;
            g.emp[i].score += v;
        }
        let r = battle_report_to(g, i as i16);
        out!(g, r; "You defended ", cemp(g), uname(g, t), cn(g), " against ", col(14), uname(g, me), cn(g), " and lost ",
             cnum(g), als, cn(g), " soldiers and ", cnum(g), alh, cn(g), " heavy cruisers.");
        report_done(g, r);
    }
    if sd0 > 0 {
        let v = proportion(ks, g.emp[t].soldiers, sd0);
        g.emp[t].soldiers -= v;
    }
    if hd0 > 0 {
        let v = proportion(kh, g.emp[t].heavycruisers, hd0);
        g.emp[t].heavycruisers -= v;
    }
    g.emp[t].stations -= kst;
    g.emp[t].lightcruisers -= kl;
    {
        let r = battle_report_to(g, g.emp[t].id);
        let mn = ename(g, me);
        attack_message(g, r, &mn, score);
        g.emp[me].w229 = g.emp[me].w229.wrapping_add(1);
        if won {
            show_victory(g, cc, credits, &s_captured);
        } else {
            if random(5) == 0 && show_screen(g, None, b"LoseWar", false) {
                pause_prompt(g);
            }
            kout!(g; "You have lost the battle.\n");
        }
        kout!(g; "You lost ");
        put_item_list(g, KOUT, &s_lost);
        kout!(g; ".  ");
        kout!(g; cemp(g), uname(g, t), cn(g), " lost ");
        put_item_list(g, KOUT, &s_killed);
        kout!(g; ".\n");
        if won && total_planets(g, t) <= 0 {
            kout!(g; col(12), b'\n', uname(g, t), col(14), " is completely destroyed!!!\n", cn(g));
        }
        report_done(g, r);
    }
    true
}

// 545e:0000
fn guerilla(g: &mut Sre, me: usize, t: usize) -> bool {
    let mut theirs = g.emp[t].soldiers;
    let mut score: i16 = 0;
    let mut rounds: i16 = 0;
    if !can_attack(g, me, t) {
        return false;
    }
    kout!(g; "You have ", cnum(g), g.emp[me].soldiers, cn(g), " soldiers.\n");
    kout!(g; "How many soldiers to send?  ");
    let s = g.emp[me].soldiers;
    set_defval(g, KIN, s);
    set_maxval(g, KIN, s);
    let mut mine = in_i32(g, KIN);
    kout!(g; "Withdraw after loss of how many soldiers?  ");
    set_defval(g, KIN, mine);
    set_maxval(g, KIN, mine);
    let mut retreat = in_i32(g, KIN);
    if mine == 0 || retreat == 0 {
        kout!(g; col(12), "ABORTED", cn(g), b'\n');
        return false;
    }
    let sent = mine;
    retreat = mine - retreat;
    if retreat == 0 {
        kout!(g; "A battle to the death...\n");
    }
    kout!(g; col(13));
    let cap = |v: i32| (if v > 30000 { 30000 } else { v }) as i16;
    while mine > retreat && theirs > 0 && rounds < 0x4e {
        let mut a = random(cap(mine));
        let mut b = random(cap(theirs));
        if a < b {
            mine -= theirs / 300 + 1;
        }
        a = random(cap(mine));
        b = random(cap(mine));
        if a < b {
            mine -= mine / 500 + 1;
        }
        a = random(cap(theirs));
        b = random(cap(mine));
        if a < b {
            theirs -= mine / 100 + 1;
        }
        rounds += 1;
        kout!(g; b'x');
    }
    kout!(g; cn(g), b'\n');
    if mine > retreat {
        score += 1;
    }
    if mine < retreat {
        score -= 1;
    }
    if theirs > 0 {
        score -= 1;
    }
    if theirs < 0 {
        score += 1;
    }
    if random(13) == 0 && show_screen(g, None, b"Guerilla", false) {
        pause_prompt(g);
    }
    if mine < 0 {
        mine = 0;
    }
    let mut lost = sent - mine;
    if g.emp[me].soldiers < lost {
        lost = g.emp[me].soldiers;
    }
    kout!(g; "You lost ", cnum(g), lost, cn(g), " soldiers.\n");
    g.emp[me].soldiers -= lost;
    g.emp[me].w229 = g.emp[me].w229.wrapping_add(1);
    let before = g.emp[t].soldiers;
    {
        let x = &mut g.emp[t];
        x.soldiers -= (x.soldiers - theirs) / 3;
        if x.soldiers < 0 {
            x.soldiers = 0;
        }
    }
    let after = g.emp[t].soldiers;
    let mut secret = false;
    let mut riot = false;
    if score > 0 {
        secret = true;
        g.emp[me].score = g.emp[me].score.wrapping_add((score as i32 * 10) as i16 as i32);
        g.emp[me].effectiveness = clamp100b(g.emp[me].effectiveness as i16 - 10);
        if g.emp[me].effectiveness < 0 {
            g.emp[me].effectiveness = clamp100b(0);
        }
        g.emp[t].effectiveness = clamp100b(g.emp[t].effectiveness as i16 - 5);
        if g.emp[t].insurgency < 7 && random(2) == 0 {
            riot = true;
            insurgency_up(g, t);
        }
    } else {
        let bonus = (score << 1) as i8;
        g.emp[me].effectiveness = clamp100b(g.emp[me].effectiveness as i16 + bonus as i16);
        let v = (random(200) as i32 + 0xdc - score as i32 * 0x28) as i16 as i32;
        g.emp[me].score -= v;
        if g.emp[me].score < 0 {
            g.emp[me].score = 0;
        }
        break_treaties(g, me, t);
        let r = battle_report_to(g, g.emp[t].id);
        out!(g, r; cemp(g), uname(g, me), cn(g), " attempted a secret guerilla invasion of your empire!");
        out!(g, r; "  You lost ", cnum(g), before - after, cn(g), " soldiers defending.");
        out!(g, NEWS; cemp(g), uname(g, me), cn(g), " attempted a guerilla invasion of ", col(9), uname(g, t), cn(g), ".\n");
        report_done(g, r);
    }
    kout!(g; "You lost ", cnum(g), lost, cn(g), " soldiers.\n");
    if secret {
        kout!(g; "You have weakened the enemy without revealing your identity.\n");
    } else {
        kout!(g; "Your immoral acts have been discovered and reported to ", cnum(g), uname(g, t), cn(g), ".\n");
    }
    if riot {
        kout!(g; "Riots break out in your opponent's empire due to your ambush.\n");
    }
    true
}

// 5c5c:060b
fn nuclear(g: &mut Sre, p: &mut Player, me: usize, t: usize) -> bool {
    let mut div: i16 = 100 / g.galaxy.w3b4 + 1;
    if g.emp[me].weapons & 1 == 0 {
        kout!(g; "You have no nuclear weapons!\n");
        return false;
    }
    if !can_attack(g, me, t) {
        return false;
    }
    if !yes_no(g, 0, b"The Galactic Coordinator has outlawed nuclear weapons due to the environmental destruction they cause.  By using the bomb, you will be seriously angering It.\nAre you sure you wish to do this") {
        return false;
    }
    break_treaties(g, me, t);
    g.emp[me].weapons &= !1;
    g.emp[me].w229 = g.emp[me].w229.wrapping_add(1);
    if g.galaxy.w3b2 > 50 {
        g.galaxy.w3b2 -= 30;
    }
    if g.galaxy.w3b4 > 3 {
        g.galaxy.w3b4 -= 1;
    }
    if random(33) == 0 {
        // The bomb goes off at home.
        let tp = total_planets(g, me);
        while tp / div as i32 > g.galaxy.w3b2 as i32 && div < 100 {
            div += 1;
        }
        for i in 0..10 {
            let c = g.emp[me].planets[i].count;
            g.emp[me].planets[i].count -= c / div as i32;
        }
        let lost = tp - total_planets(g, me);
        out!(g, NEWS; cemp(g), uname(g, me), cn(g), " was hit by ");
        let pr = player_pronoun(g, p, 2);
        out!(g, NEWS; pr, " own nuclear bomb!\n");
        kout!(g; "Your bomb was booby-trapped, and you completely destroyed ", lost, " of your own planets with the Nuclear Bomb!\n");
        return true;
    }
    let tp = total_planets(g, t);
    while tp / div as i32 > g.galaxy.w3b2 as i32 && div < 100 {
        div += 1;
    }
    for i in 0..10 {
        let c = g.emp[t].planets[i].count;
        g.emp[t].planets[i].count -= c / div as i32;
    }
    let lost = tp - total_planets(g, t);
    let r = battle_report_to(g, g.emp[t].id);
    out!(g, r; "You lost ", cnum(g), lost, cn(g), " planets in a Nuclear Attack from ", cemp(g), uname(g, me), cn(g), b'!');
    g.emp[me].d1b0 /= 2;
    out!(g, NEWS; col(9), uname(g, t), cn(g), " was hit by a nuclear bomb traced to ", cemp(g), uname(g, me), cn(g), ".\n");
    if random(3) == 0 && show_screen(g, None, b"NukeWar", false) {
        pause_prompt(g);
    }
    kout!(g; "You completely destroyed ", lost, " planets with the Nuclear Bomb!\n");
    report_done(g, r);
    true
}

// 5c5c:0f19
fn chemical(g: &mut Sre, p: &mut Player, me: usize, t: usize) -> bool {
    if (g.emp[me].weapons >> 1) & 1 == 0 {
        kout!(g; "You have no chemical weapons!\n");
        return false;
    }
    if !can_attack(g, me, t) {
        return false;
    }
    if !yes_no(g, 0, b"The Galactic Coordinator has outlawed chemical weapons.  By using the barrel of chemicals, you will be seriously angering It.  Are you sure you wish to do this") {
        return false;
    }
    g.emp[me].d1b0 = 0;
    break_treaties(g, me, t);
    g.emp[me].weapons &= !2;
    g.emp[me].w229 = g.emp[me].w229.wrapping_add(1);
    if random(10) == 0 && g.emp[t].agents > 0 {
        // The target's agents sabotage the weapon.
        let killed = g.emp[me].population / 2;
        g.emp[me].population -= killed;
        {
            let r = report_out_to(g, g.emp[t].id, None);
            out!(g, r; cemp(g), uname(g, me), cn(g),
                 " attempted to use chemical weapons on you; your covert intelligence discovered their plot and sabotaged ");
            let pr = player_pronoun(g, p, 2);
            out!(g, r; pr, " weapon!\n");
            out!(g, NEWS; "The ruler of ", cemp(g), uname(g, me), cn(g), " killed ", killed, " million of ");
            let pr = player_pronoun(g, p, 2);
            out!(g, NEWS; pr, " own civilians with a chemical weapon!\n");
            let mut tp = Player::default();
            let tpl = g.emp[t].player;
            player_load(g, &mut tp, tpl);
            kout!(g; "Your opponent sent ");
            let pr = player_pronoun(g, &tp, 2);
            kout!(g; pr, " agents to sabotage your weapon.  You kill ", killed, " million of your own civilians!\n");
            player_save(g, &mut tp);
            report_done(g, r);
        }
        return true;
    }
    let killed = g.emp[t].population / 2;
    g.emp[t].population -= killed;
    let r = battle_report_to(g, g.emp[t].id);
    out!(g, r; cnum(g), killed, cn(g), " million civilians were killed by chemical attacks by ", cemp(g), uname(g, me), cn(g),
         b'.');
    out!(g, NEWS; cemp(g), uname(g, me), cn(g), " used chemical weapons to kill innocent civilians in ", col(9), uname(g, t),
         cn(g), ".\n");
    if random(3) == 0 && show_screen(g, None, b"ChemWar", false) {
        pause_prompt(g);
    }
    kout!(g; "You kill ", killed, " million innocent civilians.\n");
    report_done(g, r);
    true
}

// 5c5c:1862
fn psionic(g: &mut Sre, me: usize, t: usize) -> bool {
    if (g.emp[me].weapons >> 2) & 1 == 0 {
        kout!(g; "You have no psionic bomb!\n");
        return false;
    }
    if !can_attack(g, me, t) {
        return false;
    }
    if !yes_no(g, 0, b"Are you sure you wish to attack with the psionic weapon") {
        return false;
    }
    break_treaties(g, me, t);
    g.emp[me].weapons &= !4;
    g.emp[me].w229 = g.emp[me].w229.wrapping_add(1);
    g.emp[t].insurgency = 7;
    g.emp[t].effectiveness = 0;
    let r = battle_report_to(g, g.emp[t].id);
    out!(g, r; "A psionic weapon from ", cemp(g), uname(g, me), cn(g), " devastated your population.\n");
    g.emp[me].d1b0 /= 2;
    out!(g, NEWS; col(9), uname(g, t), cn(g), " was hit by a psionic bomb from ", cemp(g), uname(g, me), cn(g), ".\n");
    kout!(g; "You brought ", uname(g, t), " down to UNDER COUP.\n");
    report_done(g, r);
    true
}

// 5090:0121: true when an attack was made.
pub fn attack(g: &mut Sre, p: &mut Player, e: usize, mut how: u8) -> bool {
    loop {
        let mut m = Menu::new(g, b"Attack", b"Attack Options", [0xf, 6, 6, 0xf, 0xe, 7, 7, 7, 7, 7]);
        let mut key = b'1';
        let mut add = |g: &mut Sre, m: &mut Menu, text: &[u8], id: i16| {
            m.add(g, text, key, id);
            key += 1;
        };
        if g.grb(&R_CONVENTIONAL) {
            add(g, &mut m, b"Conventional Invasion", 1);
        }
        if g.grb(&R_GUERILLA) {
            add(g, &mut m, b"Guerilla Ambush", 2);
        }
        if g.grb(&R_PIRATERAID) {
            add(g, &mut m, b"Pirate Raid", 3);
        }
        if g.grb(&R_PIRATESPY) {
            add(g, &mut m, b"Spy on Pirates", 4);
        }
        if g.emp[e].weapons & 1 != 0 {
            add(g, &mut m, b"Nuclear Offensive", 5);
        }
        if (g.emp[e].weapons >> 1) & 1 != 0 {
            add(g, &mut m, b"Chemical Warfare", 6);
        }
        if (g.emp[e].weapons >> 2) & 1 != 0 {
            add(g, &mut m, b"Psionic Attack", 7);
        }
        if g.system.regstate != 0 {
            m.add(g, b"", b'V', 0x50);
        }
        if key == b'1' {
            return false;
        }
        let mut r: i16 = -1;
        for i in 1..=7 {
            if let Some(it) = m.find(i) {
                if it.key == how {
                    r = it.id;
                    how = 0;
                }
            }
        }
        if r == -1 {
            r = m.run(g, true, -1);
        }
        if r == -1 {
            return false;
        }
        if r == 0x50 {
            crate::bank::bank(g, p, e);
            continue;
        }
        if r == 3 {
            if crate::pirateraid::pirate_raid(g, p, e) {
                return true;
            }
            continue;
        }
        if r == 4 {
            crate::pirateraid::spy_on_pirates(g);
            return true;
        }
        let maxattacks = g.grl(&R_MAXATTACKS);
        if maxattacks != -1 && g.emp[e].w229 as i32 >= g.grl(&R_MAXATTACKS) {
            kout!(g; "The ", col(15), "Galactic Coordinator", cn(g));
            kout!(g; " will not let you attack more than ", cnum(g));
            let n = g.grl(&R_MAXATTACKS);
            kout!(g; n, cn(g), " times per ", cnum(g));
            kout!(g; g.galaxy.grantturns, cn(g), " turns.\n");
            return false;
        }
        kout!(g; col(14), "Attack Empire:\n");
        let mut allowed = empire_mask(g) & !protected_mask(g);
        mask_clear(&mut allowed, g.emp[e].id);
        let mut target = choose_empire(g, allowed);
        if target == -1 {
            continue;
        }
        let mut done;
        loop {
            let t = target as usize;
            done = match r {
                1 => conventional(g, e, t),
                2 => guerilla(g, e, t),
                5 => nuclear(g, p, e, t),
                6 => chemical(g, p, e, t),
                7 => psionic(g, e, t),
                _ => false,
            };
            if !done {
                target = choose_empire(g, allowed);
            } else if total_planets(g, t) <= 0 {
                let news = report_out(g, None);
                out!(g, news; cemp(g), uname(g, t), cn(g), " has been wiped out by ", col(14), uname(g, e), cn(g), b'.');
                let blank = Report::new(g);
                let mut rep = std::mem::replace(&mut rep_of(g, news).rep, blank);
                rep.add_all(g);
                rep_of(g, news).rep = rep;
                report_done(g, news);
            }
            if !(!done && target != -1) {
                break;
            }
        }
        if done {
            return true;
        }
    }
}
