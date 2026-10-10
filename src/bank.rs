// The bank (segment 3e41) and investments in bonds (segment 1719).

use crate::events::*;
use crate::prelude::*;

static R_INVESTMENTS: GRes = GRes::num(b"Bank.Investments", 1); // 0988
static R_PLAYERINVEST: GRes = GRes::num(b"Bank.PlayerInvestments", 1); // 46fe

fn key(g: &mut Sre, c: u8) {
    kout!(g; cbr(g), b'[', cbo(g), c, cbr(g), b']', cn(g));
}

// "[MAX=n] " after a prompt.
fn max1(g: &mut Sre, max: i32) {
    kout!(g; cbr(g), b'[', cbo(g), "MAX=", max, cbr(g), b']', cn(g), b' ');
}

// 1719:000a
fn treasury_bonds(g: &mut Sre, e: usize) {
    let mut max = g.emp[e].money.get() / 8500;
    let mut turns: i16 = 5;
    let mut m = Money { v: 0 };
    m.set(8500000);
    let mut i: i16 = 0;
    while i < 200 && m.get() < 10000000 {
        let v = (g.galaxy.interest as i32).wrapping_mul(m.get() / (100 * g.galaxy.grantturns as i32));
        m.add(v);
        i += 1;
    }
    i /= 2;
    if i > turns {
        turns = i;
    }
    if max > 3392 {
        max = 3392;
    }
    kout!(g; col(12), " * ", cin(g), "Galactic Treasury Bonds", cn(g), b'\n');
    hline(g, col(4), 0x28);
    kout!(g; "For 8,500 credits, you can buy Galactic Treasury Bonds, ");
    kout!(g; "which return 10,000 credits after ", turns);
    kout!(g; " turns, or 1 (real) hour, whichever is later.\n");
    kout!(g; "How many would you like?  ");
    max1(g, max);
    set_defval(g, KIN, 0);
    set_maxval(g, KIN, max);
    let n = in_i32(g, KIN);
    if n <= 0 {
        return;
    }
    let year = (g.emp[e].year as i32 + turns as i32) as i16;
    let mut ev = Event::bank(g, g.emp[e].id, b"Galactic Treasury Bonds", 10000i32.wrapping_mul(n), year, -1);
    ev.set_due(g, 0, 1, 0);
    add_event(g, &mut ev);
    g.emp[e].money.sub(8500i32.wrapping_mul(n));
    kout!(g; cn(g), "You will be notified when your ", cnum(g), n, cn(g), " bonds mature.  (Sometime after ", "year ",
          year, ".)\n");
}

// 1719:1067
fn invest_prompt(e: usize) -> MenuHook {
    Box::new(move |g: &mut Sre| {
        kout!(g; "You have ", cnum(g), g.emp[e].money.get(), cn(g), " credits.\n");
        1
    })
}

// 1719:0394: buy the Treasury's or another empire's bonds.
fn make_investment(g: &mut Sre, e: usize) {
    let mut m = Menu::new(g, b"Investments.Buy", b"Make an Investment", [0xf, 3, 2, 0xe, 7, 7, 7, 7, 7, 7]);
    m.prompt = Some(invest_prompt(e));
    m.add(g, b"Galactic Treasury Bonds", b'$', 0x32);
    for i in 0..25usize {
        if g.emp[i].bondprice <= 0 {
            continue;
        }
        let mut k = 0u8;
        if g.emp[i].bonds < g.emp[i].maxbonds && g.emp[e].id != i as i16 {
            k = b'A' + i as u8;
        }
        let name = cs(&g.emp[i].bondname);
        m.add(g, &name, k, i as i16);
    }
    if m.count() > 7 {
        m.columns = 2;
    }
    if m.count() > 14 {
        m.columns = 3;
    }
    let r = if g.grb(&R_PLAYERINVEST) { m.run(g, true, -1) } else { 0x32 };
    if r == -1 {
        return;
    }
    if r == 0x32 {
        treasury_bonds(g, e);
        return;
    }
    let b = r as usize;
    let mut max = g.emp[e].money.get() / g.emp[b].bondprice;
    let left = (g.emp[b].maxbonds as i32 - g.emp[b].bonds as i32) as i16;
    if (left as i32) < max {
        max = left as i32;
    }
    kout!(g; col(9), " * ", cin(g), cs(&g.emp[b].bondname), cn(g), b'\n');
    hline(g, col(1), 0x28);
    kout!(g; cn(g));
    kout!(g; "These bonds are available for ", cnum(g), g.emp[b].bondprice, cn(g), " credits.  They pay ", cnum(g),
          g.emp[b].bondpayoff, cn(g), " credits after either ", g.emp[b].bondturns, " turns or ", g.emp[b].bondhours,
          " (real) hours, whichever is later.\n");
    kout!(g; "How many would you like?  ");
    max1(g, max);
    set_defval(g, KIN, 0);
    set_maxval(g, KIN, max);
    let mut n = in_i32(g, KIN);
    if n <= 0 {
        return;
    }
    let year = (g.emp[e].year as i32 + g.emp[b].bondturns as i32) as i16;
    let name = cs(&g.emp[b].bondname);
    let mut ev = Event::bank(g, g.emp[e].id, &name, n.wrapping_mul(g.emp[b].bondpayoff), year, r);
    let hours = g.emp[b].bondhours;
    ev.set_due(g, 0, hours, 0);
    add_event(g, &mut ev);
    let price = g.emp[b].bondprice;
    let mut cost = n.wrapping_mul(price);
    let oldmoney = g.emp[b].money.get();
    g.emp[b].money.add(cost);
    if g.emp[b].money.get().wrapping_sub(cost) != oldmoney {
        // The issuer's treasury is full: the rest goes to its bank account.
        let rest = cost.wrapping_sub(g.emp[b].money.get().wrapping_sub(oldmoney));
        let oldbank = g.emp[b].bank.get();
        g.emp[b].bank.add(rest);
        if g.emp[b].bank.get().wrapping_sub(rest) != oldbank {
            // (The original measures the overflow against the buyer's bank.)
            let over = rest.wrapping_sub(g.emp[e].bank.get().wrapping_sub(oldbank));
            let back = (over + price - 1) / price;
            n -= back;
            cost -= back * price;
            g.emp[b].bank.sub(back * price - over);
            kout!(g; "Your financial team could only acquire ", n, " bonds at this time.\n");
        }
    }
    g.emp[e].money.sub(cost);
    g.emp[b].bonds = (g.emp[b].bonds as i32 + n) as i16;
    let year = (g.emp[e].year as i32 + g.emp[b].bondturns as i32) as i16;
    kout!(g; cn(g), "You will be notified when your ", cnum(g), n, cn(g), " bonds mature.  (Sometime after ", "year ",
          year, ".)\n");
}

// 1719:0b7d: set up the empire's own bonds.
fn configure_bonds(g: &mut Sre, e: usize) {
    if g.emp[e].bondprice == 0 {
        kout!(g; "You are not currently offering any bonds.\n");
        if !yes_no(g, 0, b"Do you wish to offer bonds") {
            return;
        }
        if g.emp[e].bondname[0] == 0 {
            let x = &mut g.emp[e];
            strcpy(&mut x.bondname, b"- Bonds");
            x.bondname[0] = (x.id as u8).wrapping_add(b'A');
            x.bondprice = 8500;
            x.bondpayoff = 10000;
            x.bondturns = 10;
            x.bondhours = 1;
            x.maxbonds = 1;
        }
    }
    kout!(g; "Your empire currently offers ", cin(g), cs(&g.emp[e].bondname), cn(g), " for ", cnum(g),
          g.emp[e].bondprice, cn(g), " credits, which pays ", cnum(g), g.emp[e].bondpayoff, cn(g),
          " credits when it matures.\n");
    kout!(g; "The bonds mature after after ", g.emp[e].bondturns, " turns and ", g.emp[e].bondhours,
          " (real) hours.\n");
    if yes_no(g, 0, b"Do you wish to change the name") {
        let mut buf = cs(&g.emp[e].bondname);
        kout!(g; "Enter a name for your bonds: (please make it plural)\n> ");
        set_maxlen(g, KIN, 0x1f);
        get_line(g, KIN, &mut buf); // edits the current name
        if !buf.is_empty() {
            strcpy(&mut g.emp[e].bondname, &buf);
        }
    }
    kout!(g; "How much should each bond cost?  ");
    set_maxval(g, KIN, 1000000);
    set_defval(g, KIN, g.emp[e].bondprice);
    g.emp[e].bondprice = in_i32(g, KIN);
    kout!(g; "How much should be paid when the bond matures?  ");
    set_maxval(g, KIN, 2000000);
    set_defval(g, KIN, g.emp[e].bondpayoff);
    g.emp[e].bondpayoff = in_i32(g, KIN);
    kout!(g; "You can set the maturity date in terms of game turns, (real) hours, or both.  Set 0 if you do not want one or the other.\n");
    loop {
        kout!(g; "What is the number of turns before maturity?  ");
        set_maxval(g, KIN, 100);
        set_defval(g, KIN, g.emp[e].bondturns as i32);
        g.emp[e].bondturns = in_i16(g, KIN);
        kout!(g; "What is the minimum number of hours before maturity?  ");
        set_maxval(g, KIN, 240);
        set_defval(g, KIN, g.emp[e].bondhours as i32);
        g.emp[e].bondhours = in_i16(g, KIN);
        if g.emp[e].bondturns != 0 || g.emp[e].bondhours != 0 {
            break;
        }
        kout!(g; "You must set ", col(12), "either", cn(g), " the number of turns or the numbers of hours.\n");
    }
    let x = &g.emp[e];
    let div = (x.bondturns as i32 * 3 + x.bondhours as i32) as i16 as i32;
    if 30i32.wrapping_mul((x.bondpayoff - x.bondprice) / div) > x.bondprice {
        kout!(g; "The Galactic Coordinator does not permit you to create such a bond.  It thinks your bonds are too profitable.  Try lowering your payoff and try again.\n");
        g.emp[e].bondprice = 0;
        return;
    }
    kout!(g; "How many bonds can be bought (maximum) at a time?  ");
    set_maxval(g, KIN, 1000);
    set_defval(g, KIN, g.emp[e].maxbonds as i32);
    g.emp[e].maxbonds = in_i16(g, KIN);
    if g.emp[e].bondhours == 0 && g.emp[e].bondturns == 0 {
        kout!(g; "You have specified 0 for both turns and hours.\n");
        g.emp[e].bondprice = 0;
    }
}

// 1719:10b8
fn investments(g: &mut Sre, e: usize) {
    if !g.grb(&R_PLAYERINVEST) {
        make_investment(g, e);
        return;
    }
    loop {
        let mut m = Menu::new(g, b"Investments", b"Investments", [0xf, 3, 2, 0xe, 0xa, 7, 7, 7, 7, 7]);
        m.prompt = Some(invest_prompt(e));
        m.add(g, b"List investments", b'S', 1);
        m.add(g, b"Make an investment", b'I', 2);
        m.add(g, b"Configure your bonds", b'C', 3);
        if g.emp[e].bondprice > 0 {
            m.add(g, b"Kancel bond offers", b'K', 4);
        }
        let r = m.run(g, true, -1);
        if r == -1 {
            return;
        }
        if r == 1 {
            list_investments(g, g.emp[e].id);
        }
        if r == 2 {
            make_investment(g, e);
        }
        if r == 3 {
            configure_bonds(g, e);
        }
        if r == 4 && yes_no(g, 0, b"Do you wish to stop offering bonds") {
            g.emp[e].bondprice = 0;
        }
    }
}

// 3e41:0000
pub fn bank(g: &mut Sre, _p: &mut Player, e: usize) {
    if g.system.regstate == 0 && g.door.port != 0 {
        // Shown on the sysop's screen only, with a beep.
        out!(g, KLOCAL; col(14), "You are not allowed to visit the bank in the unregistered versions!\n", cn(g));
        return;
    }
    kout!(g; cn(g), "\nWelcome to ", col(14), Untr(cs(&g.system.bank)));
    let mut c;
    loop {
        kout!(g; b'\n');
        let credit = {
            let x = &g.emp[e];
            let p = |t: usize| x.planets[t].count;
            35i32.wrapping_mul(x.year as i32)
                + 3 * x.soldiers
                + 8 * x.fighters
                + 10 * x.stations
                + 10 * x.heavycruisers
                + 13 * x.carriers
                + x.food / 31
                + 230 * p(P_TOURISM)
                + 500 * p(P_ORE)
                + 500 * p(P_FOOD)
                + 400 * p(P_SUPPLY)
                + 230 * p(P_EDUCATION)
                + 330 * p(P_GOVERNMENT)
                + 50 * p(P_RESEARCH)
                + 230 * p(P_URBAN)
                + 1100 * p(P_ANTIPOLLUTION)
                + 150000
        };
        kout!(g; cn(g), "You have ", cnum(g), g.emp[e].money.get(), cn(g), " credits, and ");
        if g.emp[e].bank.get() >= 0 {
            kout!(g; cnum(g), g.emp[e].bank.get(), cn(g), " credits in the bank.\n");
        } else {
            kout!(g; "you have a loan of ", cnum(g), g.emp[e].bank.get().wrapping_neg(), cn(g), " credits.\n");
        }
        if g.emp[e].bank.get() > 0 {
            key(g, b'W');
            kout!(g; "ithdraw, ");
        } else {
            kout!(g; "Take a ");
            key(g, b'L');
            kout!(g; "oan, ");
        }
        if g.grb(&R_INVESTMENTS) {
            key(g, b'I');
            kout!(g; "nvest, ");
            key(g, b'B');
            kout!(g; "onds, ");
        }
        key(g, b'D');
        kout!(g; "eposit, or ");
        key(g, b'Q');
        kout!(g; "uit?  ", cin(g));
        loop {
            c = get_choice(g, b"QDWLBIS", true);
            if c == b'W' && g.emp[e].bank.get() <= 0 {
                c = 0;
            }
            if c == b'L' && g.emp[e].bank.get() > 0 {
                c = 0;
            }
            if !g.grb(&R_INVESTMENTS) && (c == b'I' || c == b'B') {
                c = 0;
            }
            if c != 0 {
                break;
            }
        }
        if g.system.regstate == 0 {
            c = 0;
        }
        if c == b'S' {
            kout!(g; "See Investments", cn(g), b'\n');
            list_investments(g, g.emp[e].id);
        }
        if c == b'I' {
            kout!(g; "Invest", cn(g), b'\n');
            treasury_bonds(g, e);
        }
        if c == b'D' {
            kout!(g; "Deposit", cn(g), b'\n');
            let mut max = g.emp[e].money.get();
            if g.emp[e].bank.get() >= 0 && 2000000000 - g.emp[e].bank.get() < max {
                max = 2000000000 - g.emp[e].bank.get();
            }
            if g.emp[e].bank.get() >= 0 {
                kout!(g; "At a ", cnum(g), g.galaxy.interest, cn(g));
                kout!(g; "% daily interest rate,\nhow much do you wish to deposit?  ");
                max1(g, max);
            } else {
                kout!(g; "You have loaned ", cnum(g), g.emp[e].bank.get().wrapping_neg(), cn(g), " credits at a ");
                kout!(g; cnum(g), g.galaxy.interest, cn(g), "% annual interest rate.\n");
                kout!(g; "How much do you wish to pay back?  ");
            }
            set_defval(g, KIN, 0);
            set_maxval(g, KIN, max);
            let n = in_i32(g, KIN);
            g.emp[e].money.sub(n);
            g.emp[e].bank.add(n);
        }
        if c == b'B' {
            kout!(g; "Bonds", cn(g), b'\n');
            investments(g, e);
        }
        if c == b'W' && g.emp[e].bank.get() > 0 {
            kout!(g; "Withdraw", cn(g), b'\n');
            kout!(g; "How much do you wish to withdraw?  ");
            let b = g.emp[e].bank.get();
            max1(g, b);
            let mut max = g.emp[e].bank.get();
            if 2000000000 - g.emp[e].money.get() < g.emp[e].bank.get() {
                max = 2000000000 - g.emp[e].money.get();
            }
            set_defval(g, KIN, 0);
            set_maxval(g, KIN, max);
            let mut n = in_i32(g, KIN);
            if g.emp[e].money.get() > 0 && money_overflow(g.emp[e].money.get(), n) {
                kout!(g; "[SRE can only handle numbers up to ", 2000000000i32,
                      ", so some money will have to stay in the bank.]\n");
                while g.emp[e].money.get() < 1990000000 && n > 10000000 {
                    g.emp[e].money.add(10000000);
                    n -= 10000000;
                    g.emp[e].bank.sub(10000000);
                }
                while g.emp[e].money.get() < 2000000000 && n > 10000 {
                    g.emp[e].money.add(10000);
                    n -= 10000;
                    g.emp[e].bank.sub(10000);
                }
            } else {
                g.emp[e].bank.sub(n);
                g.emp[e].money.add(n);
            }
        }
        if c == b'L' && g.emp[e].bank.get() <= 0 {
            let mut payment: i32 = 0;
            kout!(g; "Loan", cn(g), b'\n');
            if credit.wrapping_add(g.emp[e].bank.get()) <= 0 {
                kout!(g; "Your credit is overdrawn.\n");
            } else {
                let mut avail = Money { v: 0 };
                avail.set(credit);
                avail.add(g.emp[e].bank.get());
                let x = &g.emp[e];
                if total_planets(g, e) + 8 * x.planets[P_PETROLEUM].count < 100 * x.planets[P_ANTIPOLLUTION].count {
                    kout!(g; "This bank rewards environmentalists who have set up anti-pollution planets; you have a better credit rating with us.\n");
                    // (The original raises a copy of the limit that is never used.)
                }
                kout!(g; "At a ", cnum(g), g.galaxy.interest, cn(g));
                kout!(g; "% annual interest rate,\nhow much do you wish to borrow?  ");
                max1(g, avail.get());
                set_defval(g, KIN, 0);
                set_maxval(g, KIN, avail.get());
                let mut n = in_i32(g, KIN);
                let interest = g.galaxy.interest as i32;
                if n > 0 {
                    kout!(g; "That will require a ", cnum(g), interest.wrapping_mul(n / 100), cn(g), " finance charge.\n");
                    payment = interest.wrapping_mul(n / 70) + 1;
                    kout!(g; "The yearly payment is ", cnum(g), payment, cn(g), " credits.\n");
                    if !yes_no(g, 0, b"Are you sure you want a loan") {
                        payment = 0;
                        n = 0;
                    }
                }
                if money_overflow(n, g.emp[e].money.get()) && g.emp[e].money.get() > 0 {
                    kout!(g; "The bank is unable to find the full sum of money for loan.\n");
                    while g.emp[e].money.get() < 1990000000 && n > 10000000 {
                        g.emp[e].money.add(10000000);
                        n -= 10000000;
                        g.emp[e].bank.sub(100000 * interest + 10000000);
                    }
                    while g.emp[e].money.get() < 2000000000 && n > 10000 {
                        g.emp[e].money.add(10000);
                        n -= 10000;
                        g.emp[e].bank.sub(100 * interest + 10000);
                    }
                } else {
                    g.emp[e].bank.sub(n.wrapping_add(n.wrapping_mul(interest) / 100));
                    g.emp[e].money.add(n);
                    g.emp[e].d1a4 += payment;
                }
            }
        }
        if c == b'Q' {
            break;
        }
    }
    kout!(g; "Quit", cn(g), "\n\n");
}
