// Diplomacy: proposing, accepting and breaking treaties (segment 70d2)
// and the treaty names (14f6:044e, 14f6:04ba).
//
// e.treaty[o] is e's treaty with o.  Its state field c records a
// message from o to e: 1 = o proposes this treaty, 2 = e rejected o's
// proposal, 3 = o accepted e's proposal, 4 = o breaks the treaty.

use crate::prelude::*;

pub static R_DIPLOMACY: GRes = GRes::num(b"Diplomacy", 1); // b01e
static R_NEUTRAL: GRes = GRes::num(b"Treaty.Neutrality", 1); // b02a
static R_FREETRADE: GRes = GRes::num(b"Treaty.FreeTrade", 1); // b036
static R_MINOR: GRes = GRes::num(b"Treaty.MinorAlliance", 1); // b042
static R_ARMED: GRes = GRes::num(b"Treaty.ArmedDefense", 1); // b04e
static R_CRUISER: GRes = GRes::num(b"Treaty.CruiserDefense", 1); // b05a
static R_TOTAL: GRes = GRes::num(b"Treaty.TotalDefense", 1); // b066
static R_BREAK: GRes = GRes::num(b"Treaty.Break", 1); // b072
static R_CUSTOM: GRes = GRes::num(b"Treaty.Custom", 1); // b07e

pub fn r_diplomacy(g: &mut Sre) -> bool {
    g.grb(&R_DIPLOMACY)
}

// 14f6:044e
pub fn treaty_name(typ: i16) -> Str {
    const NAMES: [&[u8]; 8] = [
        b"No Relations",
        b"Neutrality/Peace Treaty",
        b"Free Trade Agreement",
        b"Minor Alliance Pact",
        b"Armed Defense Pact",
        b"Cruiser Protection Plan",
        b"Total Defense Pact",
        b"Custom Treaty",
    ];
    if typ as u16 > TT_CUSTOM as u16 {
        return b"#<unknown-treaty-type>".to_vec();
    }
    NAMES[typ as usize].to_vec()
}

// 14f6:04ba: the name, with the percentages of a custom treaty.
pub fn treaty_desc(t: Treaty) -> Str {
    let s = treaty_name(t.typ());
    if t.typ() != TT_CUSTOM {
        return s;
    }
    cat(&[&s, b" (", &int_str(t.soldierpct() as i32), b"%/", &int_str(t.cruiserpct() as i32), b"%)"])
}

fn alive(g: &Sre, e: usize) -> bool {
    g.emp[e].exists() && total_planets(g, e) > 0 && g.emp[e].population > 0
}

// A treaty, named by whose it is and with whom.
#[derive(Clone, Copy)]
struct T(usize, usize);

fn tr(g: &mut Sre, t: T) -> &mut Treaty {
    &mut g.emp[t.0].treaty[t.1]
}

// 70d2:17b1: the treaties of an empire with the others; the empires in
// mark are bracketed.
fn relations(g: &mut Sre, me: usize, mark: i32) {
    kout!(g; col(14), "\nRelations with other empires\n", cn(g));
    hline(g, col(2), 0x46);
    galaxy_open(g);
    for i in 0..MAXEMPIRES {
        if key_ready(g, KIN) {
            galaxy_close(g);
            return;
        }
        if g.emp[me].id == i as i16 {
            continue;
        }
        if !g.emp[i].exists() {
            continue;
        }
        let letter = if g.emp[i].id == -1 { 0 } else { (g.emp[i].id as u8).wrapping_add(b'A') };
        if mask_test(mark, i as i16) {
            kout!(g; col(2), b'[', col(14), letter, col(2), b']');
        } else {
            kout!(g; col(7), b' ', letter, b' ');
        }
        kout!(g; "   ", col(15), setw(0x20), uname(g, i), setw(0));
        let mine = g.emp[me].treaty[g.emp[i].id as usize];
        let theirs = g.emp[i].treaty[g.emp[me].id as usize];
        if mine.typ() != TT_NONE {
            kout!(g; cnum(g), treaty_desc(mine));
            if mine.days > 0 {
                kout!(g; b' ', cbr(g), b'[', cbo(g), mine.days as i16, cbr(g), b']', cn(g));
            }
        } else if theirs.typ() != TT_NONE {
            kout!(g; col(10), treaty_desc(theirs), b' ');
            match theirs.status() {
                TS_PROPOSED => kout!(g; "Proposed"),
                TS_REJECTED => kout!(g; "Rejected"),
                TS_ACCEPTED => kout!(g; "Accepted"),
                TS_BROKEN => kout!(g; "Treaty Broken"),
                _ => {}
            }
        } else {
            kout!(g; cn(g), treaty_name(TT_NONE));
        }
        kout!(g; b'\n');
    }
    galaxy_close(g);
    hline(g, col(2), 0x46);
    kout!(g; b'\n');
}

// 70d2:09f5
fn relations_list(g: &mut Sre, mask: i32) {
    let c = g.current as usize;
    relations(g, c, mask);
}

// 70d2:03bd: may me propose this treaty to o?
fn check_proposal(g: &mut Sre, o: usize, meid: i16, typ: i16, days: i16) -> bool {
    let mut shown = false;
    let t = T(o, meid as usize);
    if tr(g, t).status() == TS_REJECTED {
        kout!(g; "You just rejected ", uname(g, o), "'s treaty.  ");
        if !yes_no(g, 0, b"Do you wish to send a new treaty instead") {
            return false;
        }
    }
    if g.emp[o].id == g.galaxy.weaponsmaster && typ != TT_NEUTRALITY && typ != TT_FREETRADE {
        kout!(g; uname(g, o), " is the Weapons Master; you cannot get a defense treaty with today's Weapons Master.\n");
        return false;
    }
    if tr(g, t).status() == TS_PROPOSED {
        let tt = *tr(g, t);
        kout!(g; "You have already proposed a ", cnum(g), treaty_desc(tt), cn(g));
        if tt.days > 0 {
            kout!(g; " for ", tt.days as i16, " turns");
        }
        kout!(g; " to ", uname(g, o), "!\n");
        shown = true;
    }
    if tr(g, t).status() == TS_BROKEN {
        kout!(g; "You have sent word that you would like to break a treaty!\n");
        shown = true;
    }
    if !shown {
        let mine = g.emp[meid as usize].treaty[g.emp[o].id as usize];
        if mine.typ() == typ && mine.days as i16 >= days {
            kout!(g; "You already have a ", cnum(g), treaty_desc(mine), cn(g));
            if mine.days > 0 {
                kout!(g; " for ", mine.days as i16, " turns");
            }
            kout!(g; " with ", uname(g, o), "!\n");
            return false;
        }
    }
    if shown && !yes_no(g, 0, b"Replace previous actions") {
        return false;
    }
    true
}

// 70d2:0a10
fn propose(g: &mut Sre, me: usize, typ: i16) {
    if typ != TT_NEUTRALITY && typ != TT_FREETRADE && g.emp[me].id == g.galaxy.weaponsmaster {
        kout!(g; "You cannot get a defense treaty while you are the Weapons Master!\n");
        return;
    }
    let mut t = Treaty::default();
    set_treaty(&mut t, typ, 0);
    if t.cruiserpct() > 0 || t.soldierpct() > 0 {
        kout!(g; "This treaty will provide ", t.soldierpct(), "% soldier defense and ", t.cruiserpct(), "% cruiser defense.\n");
    }
    let mut days: i16 = -1;
    let mut soldiers: i16 = 0;
    let mut cruisers: i16 = 0;
    let mut chosen: i32 = 0;
    let mut allowed = empire_mask(g);
    mask_clear(&mut allowed, g.emp[me].id);
    for i in 0..MAXEMPIRES {
        if !alive(g, i) {
            mask_clear(&mut allowed, i as i16);
        }
    }
    if choose_empires(g, allowed, &mut chosen, b"", relations_list, None) != 1 {
        return;
    }
    for i in 0..MAXEMPIRES {
        if !mask_test(chosen, i as i16) {
            continue;
        }
        let o = i;
        if days == -1 {
            kout!(g; "How many days?  ", cbr(g), b'[', cbo(g), "MAX=", 14i16, cbr(g), b']', cn(g), "  ");
            set_defval(g, KIN, 0);
            set_maxval(g, KIN, 14);
            days = in_i16(g, KIN);
            if typ == TT_CUSTOM {
                kout!(g; "In a Custom Treaty, you may set the percentage of soldiers and ");
                kout!(g; "heavy cruisers that are sent in the event of your treaty partner ");
                kout!(g; "being attacked.  ");
                kout!(g; "The total percentage of cruisers and soldiers can be up to 60%.\n");
                loop {
                    kout!(g; "Percentage of soldiers to send:  ");
                    set_defval(g, KIN, 10);
                    set_maxval(g, KIN, 60);
                    soldiers = in_i16(g, KIN);
                    kout!(g; "Percentage of cruisers to send:  ");
                    set_defval(g, KIN, 10);
                    set_maxval(g, KIN, 60 - soldiers as i32);
                    cruisers = in_i16(g, KIN);
                    if !(cruisers as i32 + soldiers as i32 > 60) {
                        break;
                    }
                }
            }
        }
        let meid = g.emp[me].id;
        if !check_proposal(g, o, meid, typ, days) {
            continue;
        }
        kout!(g; col(10), i as u8 + b'A', cn(g), ": ");
        let ot = tr(g, T(o, meid as usize));
        set_treaty(ot, typ, days as u8);
        ot.setstatus(TS_PROPOSED);
        if typ == TT_CUSTOM {
            ot.settype(TT_CUSTOM);
            ot.days = days as u8;
            ot.setsoldierpct(soldiers);
            ot.setcruiserpct(cruisers);
        }
        kout!(g; cnum(g), treaty_name(typ), cn(g), " proposed");
        if days > 0 {
            kout!(g; " for ", cnum(g), days, cn(g), " days");
        }
        kout!(g; ".\n");
    }
}

// 70d2:1139
fn break_treaty(g: &mut Sre, me: usize) {
    let mut chosen: i32 = 0;
    let mut allowed = empire_mask(g);
    mask_clear(&mut allowed, g.emp[me].id);
    for i in 0..MAXEMPIRES {
        if !alive(g, i) {
            mask_clear(&mut allowed, i as i16);
        }
    }
    if choose_empires(g, allowed, &mut chosen, b"", relations_list, None) != 1 {
        return;
    }
    for i in 0..MAXEMPIRES {
        if !mask_test(chosen, i as i16) {
            continue;
        }
        kout!(g; col(10), i as u8 + b'A', cn(g), ": ");
        let o = i;
        let mine = T(me, i);
        let theirs = T(o, g.emp[me].id as usize);
        if tr(g, mine).typ() == TT_NONE {
            kout!(g; "You have no relations with ", cemp(g), uname(g, o), cn(g), "!\n");
        } else if tr(g, mine).days == 0 {
            let d = treaty_desc(*tr(g, mine));
            kout!(g; "You break the ", cnum(g), d, cn(g), ".\n");
            set_treaty(tr(g, mine), TT_NONE, 0);
            tr(g, mine).setstatus(TS_NONE);
            tr(g, theirs).setstatus(TS_BROKEN);
        } else if tr(g, theirs).status() == TS_PROPOSED {
            let d = treaty_desc(*tr(g, theirs));
            kout!(g; "You have proposed a ", cnum(g), d, cn(g), " to ", cemp(g), uname(g, o), cn(g), ".  ");
            if yes_no(g, 0, b"Do you wish to cancel the proposal") {
                *tr(g, theirs) = *tr(g, mine);
                tr(g, mine).setstatus(TS_NONE);
                tr(g, theirs).setstatus(TS_NONE);
            }
        } else if tr(g, theirs).status() == TS_BROKEN {
            kout!(g; "You have already sent word that you wish to break the treaty!\n");
        } else {
            tr(g, theirs).setstatus(TS_BROKEN);
            kout!(g; "You send word that you wish to break the treaty.\n");
        }
    }
}

// 70d2:1787
pub fn view_relations_of(g: &mut Sre, me: usize) {
    relations(g, me, 0);
}

// 70d2:0000: the diplomacy menu.
pub fn diplomacy_menu(g: &mut Sre, _p: &mut Player, me: usize) {
    if !g.grb(&R_DIPLOMACY) {
        return;
    }
    loop {
        let mut m = Menu::new(g, b"Diplomacy", b"Diplomacy", [0xe, 2, 2, 0xe, 0xa, 0xd, 0xd, 7, 7, 7]);
        // 70d2:0399
        m.prompt = Some(Box::new(|g: &mut Sre| {
            kout!(g; cn(g), "What would you like to do?  ");
            0
        }));
        if g.grb(&R_NEUTRAL) {
            m.add(g, &treaty_name(TT_NEUTRALITY), b'1', TT_NEUTRALITY);
        }
        if g.grb(&R_FREETRADE) {
            m.add(g, &treaty_name(TT_FREETRADE), b'2', TT_FREETRADE);
        }
        if g.grb(&R_MINOR) {
            m.add(g, &treaty_name(TT_MINORALLIANCE), b'3', TT_MINORALLIANCE);
        }
        if g.grb(&R_ARMED) {
            m.add(g, &treaty_name(TT_ARMEDDEFENSE), b'4', TT_ARMEDDEFENSE);
        }
        if g.grb(&R_CRUISER) {
            m.add(g, &treaty_name(TT_CRUISERPROTECTION), b'5', TT_CRUISERPROTECTION);
        }
        if g.grb(&R_TOTAL) {
            m.add(g, &treaty_name(TT_TOTALDEFENSE), b'6', TT_TOTALDEFENSE);
        }
        if g.grb(&R_BREAK) {
            m.add(g, b"Break Treaty", b'7', 7);
        }
        m.add(g, b"View Relations", b'8', 8);
        if g.grb(&R_CUSTOM) {
            m.add(g, b"Propose Other Treaty", b'9', 9);
        }
        let r = m.run(g, true, -1);
        match r {
            1..=6 => propose(g, me, r),
            7 => break_treaty(g, me),
            8 => view_relations_of(g, me),
            9 => propose(g, me, TT_CUSTOM),
            _ => {}
        }
        if r == -1 {
            break;
        }
    }
}

// 70d2:1ddf: messages about treaties from the other empires.
pub fn treaty_messages(g: &mut Sre, _p: &mut Player, me: usize) {
    for i in 0..MAXEMPIRES {
        let o = i;
        if !g.emp[o].exists() {
            continue;
        }
        let a = T(me, g.emp[o].id as usize);
        let b = T(o, g.emp[me].id as usize);
        if tr(g, a).status() == TS_BROKEN {
            kout!(g; col(15), uname(g, o), cn(g), " has ");
            if tr(g, a).days > 0 {
                kout!(g; "dishonorably ");
            }
            kout!(g; "broken a treaty with you!\n");
            set_treaty(tr(g, a), TT_NONE, 0);
            tr(g, a).setstatus(TS_NONE);
            set_treaty(tr(g, b), TT_NONE, 0);
            tr(g, b).setstatus(TS_NONE);
        }
        if tr(g, a).status() == TS_PROPOSED {
            if g.emp[me].id == g.galaxy.weaponsmaster
                && total_planets(g, me) > 500
                && tr(g, a).typ() != TT_FREETRADE
                && tr(g, a).typ() != TT_NEUTRALITY
            {
                kout!(g; "Because you are the Weapons Master, the treaty has been changed to Neutrality.\n");
                let aa = tr(g, a).days;
                set_treaty(tr(g, a), TT_NEUTRALITY, aa);
            }
            let ta = *tr(g, a);
            kout!(g; col(15), uname(g, o), cn(g), " proposes a ", cnum(g), treaty_name(ta.typ()), cn(g));
            if ta.days > 0 {
                kout!(g; " for ", cnum(g), ta.days as i16, cn(g), " days");
            }
            kout!(g; ".\n");
            if ta.typ() == TT_CUSTOM {
                kout!(g; "It involves ", cnum(g), ta.soldierpct(), cn(g), "% soldier defense and ");
                kout!(g; cnum(g), ta.cruiserpct(), cn(g), "% cruiser defense.\n");
            }
            if yes_no(g, 1, b"Do you accept") {
                kout!(g; "Treaty Accepted");
                *tr(g, b) = *tr(g, a);
                tr(g, b).setstatus(TS_ACCEPTED);
                tr(g, a).setstatus(TS_NONE);
            } else {
                let typ = tr(g, a).typ();
                let days = tr(g, a).days;
                kout!(g; "Treaty Rejected");
                *tr(g, a) = *tr(g, b);
                tr(g, a).setstatus(TS_NONE);
                set_treaty(tr(g, b), typ, days);
                tr(g, b).setstatus(TS_REJECTED);
            }
            kout!(g; cn(g), ".\n");
        }
        if tr(g, a).status() == TS_ACCEPTED {
            let d = treaty_desc(*tr(g, a));
            kout!(g; cnum(g), uname(g, o), cn(g), " accepts the ", cnum(g), d, cn(g), ".\n");
            tr(g, a).setstatus(TS_NONE);
            tr(g, b).setstatus(TS_NONE);
        }
        if tr(g, a).status() == TS_REJECTED {
            let d = treaty_desc(*tr(g, a));
            kout!(g; cnum(g), uname(g, o), cn(g), " rejects the ", cnum(g), d, cn(g), " proposal.\n");
            *tr(g, a) = *tr(g, b);
        }
    }
}
