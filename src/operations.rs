// The operations menu, tax and draft rates, preferences and supply
// production (segment 6a9b).

use crate::prelude::*;

// 6a9b:06c1
fn tax_rates(g: &mut Sre, p: &mut Player, e: usize) {
    let tax = "The tax rate is ";
    kout!(g; "How much do want the tax rate to be (0%-100%)?  ");
    set_defval(g, KIN, g.emp[e].taxrate as i32);
    set_maxval(g, KIN, 100);
    let v = in_i16(g, KIN);
    {
        let x = &mut g.emp[e];
        x.taxrate = v as i8;
        if x.taxrate < 0 {
            x.taxrate = 0;
        }
        if x.taxrate > 100 {
            x.taxrate = 100;
        }
    }
    if !has_pref(p.prefs1, PF1_TERSE) {
        kout!(g; tax, "now ", cnum(g), g.emp[e].taxrate as i16, cn(g), "%.\n");
    }
    let t = g.emp[e].taxrate as i16;
    if t > 80 && random(100) < t {
        kout!(g; "A high tax rate may cause violent revolts in your empire.\n");
    } else if t > 50 && random(90) < t {
        kout!(g; "A high tax rate will cause your citizens to leave the empire.\n");
    } else if t > 30 && random(50) < t {
        kout!(g; "The citizens are unhappy with the tax rate.\n");
    }
    let draft = "The draft rate is ";
    kout!(g; "How much do want the draft rate to be (0%-30%)?  ");
    set_defval(g, KIN, g.emp[e].draftrate as i32);
    set_maxval(g, KIN, 30);
    let v = in_i16(g, KIN);
    {
        let x = &mut g.emp[e];
        x.draftrate = v as i8;
        if x.draftrate < 0 {
            x.draftrate = 0;
        }
        if x.draftrate > 100 {
            x.draftrate = 100;
        }
    }
    if !has_pref(p.prefs1, PF1_TERSE) {
        kout!(g; draft, "now ", cnum(g), g.emp[e].draftrate as i16, cn(g), "%.\n");
    }
}

fn pf(prefs: u8, f: u8) -> i16 {
    has_pref(prefs, f) as i16
}

// 6a9b:09a8
pub fn preferences(g: &mut Sre, p: &mut Player) {
    loop {
        let mut m = Menu::new(g, b"Preferences", b"Player Preferences", [0xb, 3, 3, 0xf, 0xb, 7, 7, 7, 7, 7]);
        let t: &[u8] = if pf(p.prefs1, PF1_SHORTMENUS) != 0 { b"Redisplay Menus" } else { b"Don't redisplay Menus" };
        m.add(g, t, b'1', 1);
        m.add(g, b"Automatic Payment", b'2', 2);
        m.add(g, b"Galactic Coordinator Donations", b'3', 3);
        if g.system.regstate != RS_UNREGISTERED {
            m.add(g, b"Bank Deposits", b'4', 4);
        }
        m.add(g, b"Report Dates", b'6', 6);
        m.add(g, b"Exiting the Spending Menu", b'7', 7);
        m.add(g, b"ANSI Color", b'8', 8);
        m.add(g, b"IBM Character Set", b'9', 9);
        let r = m.run(g, true, -1);
        let ask: i16;
        let yes: i16;
        match r {
            1 => {
                kout!(g; "Spending and Planet menus are now ", cnum(g));
                p.prefs1 ^= PF1_SHORTMENUS;
                if pf(p.prefs1, PF1_SHORTMENUS) != 0 {
                    kout!(g; "SHORT");
                } else {
                    kout!(g; "LONG");
                }
                kout!(g; cn(g), ".\n");
            }
            2 => {
                kout!(g; "SRE can pay your expenses automatically if you wish.  ");
                ask = yes_no(g, pf(p.prefs2, PF2_ASKPAY), b"Do you want the system to ask about automatic payment") as i16;
                if ask != 0 {
                    yes = yes_no(g, pf(p.prefs2, PF2_AUTOPAY), b"Do you want the default to be YES") as i16;
                } else {
                    yes = yes_no(g, pf(p.prefs2, PF2_AUTOPAY), b"Do you want it to automatically pay whenever possible") as i16;
                }
                set_pref(&mut p.prefs2, PF2_ASKPAY, ask & 1 != 0);
                set_pref(&mut p.prefs2, PF2_AUTOPAY, yes & 1 != 0);
            }
            3 => {
                kout!(g; "The Galactic Coordinator appreciates donations.  At the end of each turn, you have the option of donating a portion of your funds to It.  ");
                let (a, y);
                if yes_no(g, (pf(p.prefs1, PF1_ASKDONATE) == 0 && pf(p.prefs1, PF1_DONATE) != 0) as i16, b"Do you want to always donate") {
                    a = 0;
                    y = 1;
                } else if yes_no(g, (pf(p.prefs1, PF1_ASKDONATE) == 0 && pf(p.prefs1, PF1_DONATE) == 0) as i16, b"Do you want to never donate") {
                    a = 0;
                    y = 0;
                } else {
                    a = 1;
                    y = yes_no(g, pf(p.prefs1, PF1_DONATE),
                               b"The system will ask whether you want to donate.  Do you want the default to be YES") as u8;
                }
                set_pref(&mut p.prefs1, PF1_ASKDONATE, a & 1 != 0);
                set_pref(&mut p.prefs1, PF1_DONATE, y & 1 != 0);
            }
            4 => {
                kout!(g; "At the end of each turn, you can deposit the rest of your money into the bank if you wish.  ");
                let (a, y);
                if yes_no(g, (pf(p.prefs1, PF1_ASKDEPOSIT) == 0 && pf(p.prefs1, PF1_DEPOSIT) != 0) as i16, b"Do you want to always deposit") {
                    a = 0;
                    y = 1;
                } else if yes_no(g, (pf(p.prefs1, PF1_ASKDEPOSIT) == 0 && pf(p.prefs1, PF1_DEPOSIT) == 0) as i16, b"Do you want to never deposit") {
                    a = 0;
                    y = 0;
                } else {
                    a = 1;
                    y = yes_no(g, pf(p.prefs1, PF1_DEPOSIT),
                               b"The system will ask whether you want to deposit.  Do you want the default to be YES") as u8;
                }
                set_pref(&mut p.prefs1, PF1_ASKDEPOSIT, a & 1 != 0);
                set_pref(&mut p.prefs1, PF1_DEPOSIT, y & 1 != 0);
            }
            5 => {
                kout!(g; "When you win a battle, SRE can attach a message to the other empire.  ");
                let y = yes_no(g, pf(p.prefs2, PF2_BATTLEMSG), b"Would you like to do this") as u8;
                set_pref(&mut p.prefs2, PF2_BATTLEMSG, y & 1 != 0);
            }
            6 => {
                let y = yes_no(g, pf(p.prefs2, PF2_REPORTDATES), b"Do you want the date to be displayed on reports") as u8;
                set_pref(&mut p.prefs2, PF2_REPORTDATES, y & 1 != 0);
            }
            7 => {
                let y = yes_no(g, (pf(p.prefs2, PF2_ENTERSTAYS) == 0) as i16, b"Do you want Enter to exit from the spending menu");
                set_pref(&mut p.prefs2, PF2_ENTERSTAYS, !y);
            }
            8 => {
                let a = g.dev.ansi;
                g.dev.ansi = yes_no(g, a, b"Use ANSI color") as i16;
            }
            9 => {
                let i = g.dev.ibm;
                g.dev.ibm = yes_no(g, i, b"Enable IBM (high ASCII) characters") as i16;
            }
            _ => {}
        }
        if r == -1 {
            break;
        }
    }
}

// 6a9b:0f63: how supply planets divide their production.
fn supply_production(g: &mut Sre, e: usize) {
    // Menu id -> 1 + index into the empire's allocation (DS:927e).
    const SLOT: [i16; 10] = [0, 1, 2, 3, 6, 0, 0, 0, 4, 5];
    let mut m = Menu::new(g, b"Production", b"Supply Production", [0xb, 9, 3, 0xf, 0xb, 0xe, 7, 7, 7, 7]);
    m.add(g, b"Soldiers", b'1', 1);
    m.add(g, b"Fighters", b'2', 2);
    m.add(g, b"Defense Stations", b'3', 3);
    m.add(g, b"Generals", b'4', 4);
    m.add(g, b"Heavy Cruisers", b'8', 8);
    m.add(g, b"Carriers", b'9', 9);
    for i in 1..=9usize {
        if SLOT[i] == 0 {
            continue;
        }
        let v = g.emp[e].supplyrates[(SLOT[i] - 1) as usize] as i16;
        let it = m.find(i as i16).unwrap();
        it.add_column_num(v as i32);
        it.add_column_str(b"%");
    }
    m.show(g);
    kout!(g; cn(g), "\nEnter the percent of supply production dedicated to\n");
    let mut left: u16 = 100;
    for i in 1..=9usize {
        if SLOT[i] == 0 {
            continue;
        }
        let k = (SLOT[i] - 1) as usize;
        let mut def = g.emp[e].supplyrates[k] as i16 as u16;
        if def > left {
            def = left;
        }
        let text = m.find(i as i16).unwrap().text.clone();
        kout!(g; cn(g), setw(0x14), text);
        kout!(g; setw(0), cbr(g), b'[', cbo(g), "MAX=", left, cn(g), cbr(g), b']', cn(g), "  ");
        set_maxval(g, KIN, left as i32);
        set_defval(g, KIN, def as i32);
        let v = in_i16(g, KIN);
        left = left.wrapping_sub(v as u16);
        let slot = &mut g.emp[e].supplyrates[k];
        *slot = v as i8;
        if *slot < 0 {
            *slot = 0;
        }
        if *slot > 100 {
            *slot = 100;
        }
    }
}

// 6a9b:0000
pub fn operations(g: &mut Sre, p: &mut Player, e: usize) {
    loop {
        crate::play::time_check(g);
        let mut m = Menu::new(g, b"Operations", b"Operations Menu", [0xe, 4, 4, 0xc, 0xf, 7, 7, 7, 7, 7]);
        m.columns = 2;
        if g.system.regstate != RS_UNREGISTERED {
            m.add(g, b"Visit the Bank", b'2', 2);
        }
        if g.emp[e].agents > 0 {
            m.add(g, b"Covert Operations", b'3', 3);
        }
        if g.emp[e].year >= 0 && g.grb(&crate::trading::R_TRADING) {
            m.add(g, b"Trading", b'4', 4);
        }
        m.add(g, b"Display Status", b'5', 5);
        if g.grb(&crate::messages::R_MESSAGES) {
            m.add(g, b"Communications", b'6', 6);
        }
        if g.system.regstate != RS_UNREGISTERED && g.grb(&crate::play::R_LOTTERY) {
            m.add(g, b"Buy Super Lottery Tickets", b'7', 7);
        }
        m.add(g, b"See Scores", b'8', 8);
        if g.grb(&crate::diplomacy::R_DIPLOMACY) {
            m.add(g, b"Diplomacy", b'9', 9);
        }
        m.add(g, b"Visit Advisors", b'A', 10);
        m.add(g, b"Tax & Draft Rates", b'T', 11);
        if g.emp[e].planets[P_SUPPLY].count > 0 {
            m.add(g, b"Supply Planet Production", b'S', 12);
        }
        m.add(g, b"Preferences", b'P', 13);
        m.add(g, b"Instructions File", b'I', 14);
        m.add(g, b"More Help", b'H', 15);
        m.add(g, b"Abdicate", b'^', 16);
        let r = m.run(g, true, -1);
        match r {
            2 => crate::bank::bank(g, p, e),
            3 => crate::covert::covert_ops(g, p, e),
            4 => crate::trading::trade(g, p, e),
            5 => show_status(g, e),
            6 => crate::messages::communications(g, p, e),
            7 => crate::lottery::buy_tickets(g, p, e),
            8 => score_list_all(g),
            9 => {
                crate::diplomacy::treaty_messages(g, p, e);
                crate::diplomacy::diplomacy_menu(g, p, e);
            }
            10 => crate::advisors::advisors(g, p, e),
            11 => tax_rates(g, p, e),
            12 => supply_production(g, e),
            13 => preferences(g, p),
            14 => {
                display_file(g, b"Text/Player.MSG", 0);
            }
            15 => crate::help::more_help(g),
            16 => 'case: {
                if !yes_no(g, 0, b"Are you sure you wish to abdicate") {
                    break 'case;
                }
                kout!(g; "You are about to lose your empire.  If you are sure you would like to quit (and possibly start over), type in your empire name as displayed:\n");
                kout!(g; ">> ", cemp(g), uname(g, e), cn(g), b'\n');
                kout!(g; ">> ");
                set_maxlen(g, KIN, 0x50);
                let buf = in_line(g, KIN);
                set_maxlen(g, KIN, 0);
                if !str_eq_i2(&ename(g, e), &buf) {
                    kout!(g; "Aborted!\n");
                    break 'case;
                }
                kout!(g; "Your reign over ", uname(g, e), " has ended!\n");
                crate::sysop::delete_empire(g, e);
                return;
            }
            _ => {}
        }
        if r == -1 {
            break;
        }
    }
}
