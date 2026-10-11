// Covert operations (segment 409c).

use crate::events::*;
use crate::player::random_pronoun;
use crate::prelude::*;

// Covert operations (the menu's items are one more).
pub const OP_SPY: i16 = 0;
pub const OP_INSURGENTAID: i16 = 1;
pub const OP_SETUP: i16 = 2;
pub const OP_DISSENSION: i16 = 3;
pub const OP_DEMORALIZE: i16 = 4;
pub const OP_BOMBING: i16 = 5;
pub const OP_INFILTRATION: i16 = 6;
pub const OP_HOSTAGES: i16 = 7;
pub const OP_BRIBE: i16 = 8;

const CAUGHT: &str = "Your covert agent was caught and was publicly executed!\n";

// 14f6:05dc: the mask of empires an operation was done to this turn
// (or today).
fn opmask_ref(x: &mut Empire, op: i16) -> &mut i32 {
    match op {
        OP_INSURGENTAID => &mut x.aidmask,
        OP_SETUP => &mut x.setupmask,
        OP_DISSENSION => &mut x.dissensionmask,
        OP_DEMORALIZE => &mut x.demoralizemask,
        OP_BOMBING => &mut x.bombmask,
        OP_HOSTAGES => &mut x.hostagemask,
        OP_BRIBE => &mut x.bribemask,
        _ => &mut x.othermask,
    }
}

pub fn opmask(g: &Sre, e: usize, op: i16) -> i32 {
    let x = &g.emp[e];
    match op {
        OP_INSURGENTAID => x.aidmask,
        OP_SETUP => x.setupmask,
        OP_DISSENSION => x.dissensionmask,
        OP_DEMORALIZE => x.demoralizemask,
        OP_BOMBING => x.bombmask,
        OP_HOSTAGES => x.hostagemask,
        OP_BRIBE => x.bribemask,
        _ => x.othermask,
    }
}

pub fn set_opmask(g: &mut Sre, e: usize, op: i16, v: i32) {
    *opmask_ref(&mut g.emp[e], op) = v;
}

pub fn opmask_set_bit(g: &mut Sre, e: usize, op: i16, bit: i16) {
    let mut m = opmask(g, e, op);
    mask_set(&mut m, bit);
    set_opmask(g, e, op, m);
}

fn alive(g: &Sre, e: usize) -> bool {
    g.emp[e].exists() && total_planets(g, e) > 0 && g.emp[e].population > 0
}

// 409c:0000: the odds of a covert operation.
fn chance(g: &Sre, me: usize, t: usize) -> &'static str {
    let mut ours = g.emp[me].agents.wrapping_mul(total_planets(g, t) + 300);
    let mut theirs = g.emp[t].agents.wrapping_mul(total_planets(g, me) + 300);
    let ours0 = ours;
    ours = ours.wrapping_add(theirs / 5);
    theirs = theirs.wrapping_add(ours0 / 5);
    // Halve both until one is down to 1: what is left of the other is the
    // ratio of the two, in powers of two.
    while ours > 1 && theirs > 1 {
        ours >>= 1;
        theirs >>= 1;
    }
    if ours == theirs {
        return "FAIR";
    }
    if ours <= 1 {
        if theirs > 15 {
            return "NONEXISTENT";
        }
        if theirs > 7 {
            return "VERY SLIM";
        }
        if theirs > 3 {
            return "SLIM";
        }
        return "UNFAVORABLE";
    }
    if ours > 15 {
        return "ALMOST CERTAIN";
    }
    if ours > 7 {
        return "FAIRLY GOOD";
    }
    if ours > 3 {
        return "GOOD";
    }
    "FAVORABLE"
}

// 409c:0197: does ours covert operation of me against t succeed?
pub fn spy_succeeds(g: &Sre, me: usize, t: usize) -> bool {
    if me == t {
        return true;
    }
    // Each side's strength: its agents, against the size of the other.
    let mut ours = g.emp[me].agents.wrapping_mul(total_planets(g, t) + 300);
    let mut theirs = g.emp[t].agents.wrapping_mul(total_planets(g, me) + 300);
    let ours0 = ours;
    ours = ours.wrapping_add(theirs / 5);
    theirs = theirs.wrapping_add(ours0 / 5);
    while ours > 32000 || theirs > 32000 {
        ours /= 2;
        theirs /= 2;
    }
    let us = ours as u16;
    let them = theirs as u16;
    if us == 0 {
        return false;
    }
    if them == 0 {
        return true;
    }
    // Each side adds a random part of the other's strength.
    let ourroll = us.wrapping_add(((them as i16 as i32 * b_rand() as i32) / 0x8000) as u16);
    let theirroll = them.wrapping_add(((us as i16 as i32 * b_rand() as i32) / 0x8000) as u16);
    ourroll > theirroll
}

// 409c:0508: a covert operation ends the treaties between two empires.
fn break_treaties(g: &mut Sre, me: usize, t: usize) {
    let (mid, tid) = (g.emp[me].id as usize, g.emp[t].id as usize);
    set_treaty(&mut g.emp[me].treaty[tid], TT_NONE, 0);
    g.emp[me].treaty[tid].setstatus(TS_NONE);
    set_treaty(&mut g.emp[t].treaty[mid], TT_NONE, 0);
    g.emp[t].treaty[mid].setstatus(TS_NONE);
}

// 409c:033d
fn choose_target(g: &mut Sre, me: usize, op: i16) -> i16 {
    let mut allowed = empire_mask(g) & !protected_mask(g);
    for i in 0..MAXEMPIRES {
        if !alive(g, i) {
            mask_clear(&mut allowed, i as i16);
        }
    }
    if op != OP_SETUP {
        mask_clear(&mut allowed, g.emp[me].id);
    }
    if op != OP_SPY && op != OP_INFILTRATION {
        for i in 0..MAXEMPIRES {
            let t = g.emp[me].treaty[i];
            if (t.typ() != TT_NONE && t.days > 0) || t.typ() == TT_NEUTRALITY {
                mask_clear(&mut allowed, i as i16);
            }
        }
    }
    let id = choose_empire(g, allowed);
    if id == -1 {
        return -1;
    }
    if op != OP_SPY && op != OP_INFILTRATION && op != OP_BRIBE && mask_test(opmask(g, me, op), id) {
        kout!(g; "But you have already performed covert ops with that empire!  Limit one empire per ");
        kout!(g; if op == OP_INSURGENTAID || op == OP_DISSENSION || op == OP_DEMORALIZE { "turn.\n" } else { "day.\n" });
        return -1;
    }
    id
}

// 409c:084d: pay for an operation and choose its target.
fn attempt(g: &mut Sre, me: usize, cost: i32, target: &mut i16, op: i16) -> bool {
    if g.emp[me].money.get() < cost {
        kout!(g; "You cannot afford that.\n");
        return false;
    }
    *target = choose_target(g, me, op);
    if *target == -1 {
        return false;
    }
    if op != OP_INSURGENTAID && op != OP_SPY && op != OP_BRIBE {
        let c = chance(g, me, *target as usize);
        kout!(g; cn(g), "Chance of success:  ", col(13), c, cn(g), ".\n");
        if !yes_no(g, 1, b"Do you want to continue") {
            return false;
        }
    }
    if op != OP_SPY && op != OP_INFILTRATION && op != OP_BRIBE {
        opmask_set_bit(g, me, op, *target);
    }
    g.emp[me].money.sub(cost);
    true
}

// 409c:09b3
fn covert_prompt(me: usize) -> MenuHook {
    Box::new(move |g: &mut Sre| {
        kout!(g; "You have ", cnum(g), g.emp[me].money.get(), cn(g), " credits and ", cnum(g), g.emp[me].agents, cn(g),
              " covert agent");
        if g.emp[me].agents != 1 {
            kout!(g; b's');
        }
        kout!(g; ".\nWhat would you like to do?  ");
        0
    })
}

// 409c:0a6d
pub fn covert_ops(g: &mut Sre, p: &mut Player, me: usize) {
    let mut cost = [0i32; 10];
    cost[1] = if g.emp[me].year < 0 { 880 } else { 3100 };
    cost[2] = 10000;
    cost[3] = 20000;
    cost[4] = 25000;
    cost[5] = 30000;
    cost[6] = 30000;
    cost[7] = 30000;
    cost[8] = 50000;
    cost[9] = 50000;
    while g.emp[me].agents > 0 {
        let mut m = Menu::new(g, b"Covert", b"Covert Operations", [0xe, 5, 5, 0xe, 0xd, 0xf, 7, 7, 7, 7]);
        m.prompt = Some(covert_prompt(me));
        m.add(g, b"Send spy", b'1', 1);
        if g.emp[me].year >= 0 {
            m.add(g, b"Insurgent aid", b'2', 2);
            m.add(g, b"Set up", b'3', 3);
            m.add(g, b"Support dissension", b'4', 4);
            m.add(g, b"Demoralize troops", b'5', 5);
            m.add(g, b"Bombing operations", b'6', 6);
        }
        m.add(g, b"Infiltration", b'7', 7);
        if g.emp[me].year >= 0 {
            if g.system.regstate != RS_UNREGISTERED {
                m.add(g, b"Take hostages", b'8', 8);
            }
            m.add(g, b"Bribe spy", b'9', 9);
        }
        if g.system.regstate != RS_UNREGISTERED {
            m.add(g, b"", b'V', 0x50);
        }
        m.add(g, b"", b'S', 0x51);
        for i in 1..=9 {
            if let Some(it) = m.find(i) {
                it.add_column_num(cost[i as usize]);
            }
        }
        let r = m.run(g, true, -1);
        let mut id: i16 = -1;
        let meid = g.emp[me].id;
        match r {
            1 => 'case: {
                if !attempt(g, me, cost[1], &mut id, OP_SPY) {
                    break 'case;
                }
                let t = id as usize;
                if spy_succeeds(g, me, t) || mask_test(opmask(g, me, OP_BRIBE), g.emp[t].id) {
                    kout!(g; "Your spy was successful.\nSpy results are...\n");
                    empire_status(g, t, false);
                } else {
                    let rep = covert_report(g, g.emp[t].id);
                    kout!(g; CAUGHT);
                    out!(g, rep; "A covert agent from ", cemp(g), uname(g, me), cn(g),
                         " was caught spying on your empire and was terminated!");
                    g.emp[me].agents -= 1;
                    report_done(g, rep);
                }
            }
            2 => 'case: {
                if !attempt(g, me, cost[2], &mut id, OP_INSURGENTAID) {
                    break 'case;
                }
                let t = id as usize;
                break_treaties(g, me, t);
                if g.emp[t].insurgency >= 7 {
                    kout!(g; "That empire already is ", cnum(g), insurgency_name(g, t), cn(g), ".\n");
                    break 'case;
                }
                opmask_set_bit(g, me, OP_INSURGENTAID, id);
                let mut k: i16 = 0;
                while k < g.emp[t].insurgency {
                    if !spy_succeeds(g, me, t) {
                        let rep = covert_report(g, g.emp[t].id);
                        out!(g, rep; "A covert agent from ", cemp(g), uname(g, me), cn(g),
                             " was caught trying to aid insurgents and was executed!");
                        g.emp[me].agents -= 1;
                        if g.grb(&R_FAILEDCOVERT) {
                            out!(g, NEWS; cemp(g), uname(g, me), cn(g), " attempted to aid insurgents in ", col(13),
                                 uname(g, t), cn(g), "!\n");
                        }
                        kout!(g; CAUGHT);
                        report_done(g, rep);
                        break;
                    }
                    k += 1;
                }
                if g.emp[t].insurgency == k {
                    insurgency_up(g, t);
                    kout!(g; "Your spy was successful in raising ", cnum(g), uname(g, t), "'s", cn(g), " insurgency to ",
                          cnum(g), insurgency_name(g, t), cn(g), ".\n");
                }
            }
            3 => 'case: {
                if g.emp[me].money.get() < cost[3] {
                    kout!(g; "You can't afford to set anyone up.\n");
                    break 'case;
                }
                kout!(g; "Choose an 'attacker' empire:\n");
                let a = choose_target(g, me, OP_SETUP);
                if a == -1 {
                    break 'case;
                }
                kout!(g; "Choose a 'defender' empire:\n");
                let d = choose_target(g, me, OP_SETUP);
                if d == -1 {
                    break 'case;
                }
                if a == d {
                    kout!(g; "That makes no sense.\n");
                    break 'case;
                }
                g.emp[me].money.sub(cost[3]);
                let (ea, ed) = (a as usize, d as usize);
                break_treaties(g, me, ea);
                break_treaties(g, me, ed);
                opmask_set_bit(g, me, OP_SETUP, a);
                opmask_set_bit(g, me, OP_SETUP, d);
                g.emp[me].agents -= 1;
                let ok = spy_succeeds(g, me, ea) || spy_succeeds(g, me, ed);
                {
                    let mut ev = Event::setup(g, meid, g.emp[ea].id, g.emp[ed].id, ok as i16);
                    add_event(g, &mut ev);
                }
                kout!(g; "You have sent an agent to set up ", col(14), uname(g, ea), cn(g), " and ", col(11), uname(g, ed),
                      cn(g), ".\n");
            }
            4 | 5 | 8 => 'case: {
                let (op, kind) = match r {
                    4 => (OP_DISSENSION, EvKind::Dissension),
                    5 => (OP_DEMORALIZE, EvKind::Demoralize),
                    _ => (OP_HOSTAGES, EvKind::Hostages),
                };
                if !attempt(g, me, cost[r as usize], &mut id, op) {
                    break 'case;
                }
                let t = id as usize;
                // (The order of these differs between the operations.)
                if r == 5 {
                    g.emp[me].agents -= 1;
                    break_treaties(g, me, t);
                    opmask_set_bit(g, me, OP_DEMORALIZE, id);
                } else if r == 4 {
                    break_treaties(g, me, t);
                    g.emp[me].agents -= 1;
                    opmask_set_bit(g, me, OP_DISSENSION, id);
                } else {
                    break_treaties(g, me, t);
                    opmask_set_bit(g, me, OP_HOSTAGES, id);
                    g.emp[me].agents -= 1;
                }
                let ok = spy_succeeds(g, me, t);
                {
                    let mut ev = Event::covert(g, kind, meid, g.emp[t].id, ok as i16);
                    ev.set_due(g, 0, 12, 0);
                    add_event(g, &mut ev);
                }
                kout!(g; "You have sent a covert agent to ", cnum(g), uname(g, t), cn(g));
                kout!(g; match r {
                    4 => " to support dissension.\n",
                    5 => " to demoralize troops.\n",
                    _ => " to take hostages.\n",
                });
            }
            6 => 'case: {
                if !attempt(g, me, cost[6], &mut id, OP_BOMBING) {
                    break 'case;
                }
                let t = id as usize;
                break_treaties(g, me, t);
                opmask_set_bit(g, me, OP_BOMBING, id);
                if !spy_succeeds(g, me, t) {
                    let rep = covert_report(g, g.emp[t].id);
                    g.emp[me].agents -= 1;
                    out!(g, rep; "A covert agent from ", cemp(g), uname(g, me), cn(g),
                         " was caught trying to destroy your food supplies and was executed!");
                    if random(3) == 0 {
                        kout!(g; "Your covert agent was unsuccessful, and now ");
                        let pr = random_pronoun(g, 0);
                        kout!(g; pr, "'s dead.\n");
                    } else if random(2) == 0 {
                        kout!(g; "Your covert agent accidentally blew ");
                        let pr = random_pronoun(g, 1);
                        kout!(g; pr, "self up.\n");
                    } else {
                        kout!(g; "Your covert agent was unsuccessful, and ");
                        let pr = random_pronoun(g, 0);
                        kout!(g; pr, " was electrocuted.\n");
                    }
                    report_done(g, rep);
                } else {
                    {
                        let rep = covert_report(g, g.emp[t].id);
                        out!(g, rep; "A bomb blew up food supplies!");
                        report_done(g, rep);
                    }
                    if random(3) == 0 {
                        g.emp[t].food -= g.emp[t].food / 2;
                        kout!(g; "You destroyed some");
                    } else {
                        g.emp[t].food = 0;
                        if random(8) == 0 && show_screen(g, Some(b"Screen/"), b"CovBomb", false) {
                            pause_prompt(g);
                        }
                        kout!(g; "You successfully destroyed all");
                    }
                    kout!(g; " of the enemy's food supplies.\n");
                }
            }
            7 => loop {
                let mut im = Menu::new(g, b"Infiltration", b"Infiltration Ops", [0xe, 5, 5, 0xe, 0xd, 0xf, 7, 7, 7, 7]);
                im.prompt = Some(covert_prompt(me));
                im.add(g, b"Intercept Messages", b'1', 1);
                im.add(g, b"Diplomatic Status", b'2', 2);
                let c = im.run(g, true, -1);
                if c == -1 {
                    break;
                }
                if c == 1 {
                    if !attempt(g, me, cost[7], &mut id, OP_INFILTRATION) {
                        continue;
                    }
                    let t = id as usize;
                    g.emp[me].agents -= 1;
                    let mi = g.emp[me].id as usize;
                    if g.emp[t].intercepts[mi] as u8 == 0xff || !spy_succeeds(g, me, t) {
                        let rep = covert_report(g, g.emp[t].id);
                        out!(g, rep; "A covert agent from ", cemp(g), uname(g, me), cn(g),
                             " was caught trying to intercept your messages and ");
                        let pr = random_pronoun(g, 0);
                        out!(g, rep; pr, " was killed!");
                        kout!(g; CAUGHT);
                        report_done(g, rep);
                    } else {
                        kout!(g; "Your spy was has been planted.\n");
                        g.emp[t].intercepts[mi] = g.emp[t].intercepts[mi].wrapping_add(1);
                    }
                } else if c == 2 {
                    if !attempt(g, me, cost[7], &mut id, OP_INFILTRATION) {
                        continue;
                    }
                    let t = id as usize;
                    if !spy_succeeds(g, me, t) {
                        let rep = covert_report(g, g.emp[t].id);
                        out!(g, rep; "A covert agent from ", cemp(g), uname(g, me), cn(g),
                             " was caught trying to discover relations of your empire and ");
                        let pr = random_pronoun(g, 0);
                        out!(g, rep; pr, " was killed!");
                        g.emp[me].agents -= 1;
                        kout!(g; CAUGHT);
                        report_done(g, rep);
                    } else {
                        kout!(g; "Your spy was successful.\nSpy results are...\n");
                        crate::diplomacy::view_relations_of(g, t);
                    }
                }
            },
            9 => 'case: {
                if !attempt(g, me, cost[9], &mut id, OP_BRIBE) {
                    break 'case;
                }
                let t = id as usize;
                let tid = g.emp[t].id;
                if mask_test(opmask(g, me, OP_BRIBE), tid) {
                    g.emp[me].money.add(cost[9]);
                    kout!(g; "You already have a spy working for you in that empire!\n");
                    break 'case;
                }
                let mut bribed: i32 = 0;
                for i in 0..MAXEMPIRES {
                    if mask_test(opmask(g, i, OP_BRIBE), tid) {
                        bribed += 1;
                    }
                }
                if g.emp[t].agents <= bribed {
                    g.emp[me].money.add(cost[9]);
                    kout!(g; "That empire has no agents to bribe!\n");
                    break 'case;
                }
                let mut avail = g.emp[t].agents;
                for i in 0..MAXEMPIRES {
                    avail -= mask_test(opmask(g, i, OP_BRIBE), tid) as i32;
                }
                if spy_succeeds(g, me, t) && spy_succeeds(g, me, t) && avail > 0 {
                    opmask_set_bit(g, me, OP_BRIBE, tid);
                    kout!(g; "You have successfully bribed an agent from ", cemp(g), uname(g, t), cn(g), ".\n");
                } else {
                    let rep = covert_report(g, g.emp[t].id);
                    out!(g, rep; "A covert agent from ", cemp(g), uname(g, me), cn(g),
                         " was caught trying to bribe your covert agent and ");
                    let pr = random_pronoun(g, 0);
                    out!(g, rep; pr, " was shot!");
                    g.emp[me].agents -= 1;
                    kout!(g; "Your covert agent was executed!\n");
                    report_done(g, rep);
                }
            }
            0x50 => crate::bank::bank(g, p, me),
            0x51 => {
                list_covert_ops(g, meid);
            }
            _ => {}
        }
        if r == -1 {
            break;
        }
    }
}
