// Lotteries (segment 55e7).

use crate::prelude::*;

// "[MAX=n]  " after a numeric prompt.
pub fn put_max(g: &mut Sre, max: i32) {
    kout!(g; cbr(g), b'[', cbo(g), "MAX=", max, cbr(g), b']', cn(g), "  ");
}

// 55e7:0000
pub fn jackpot(_g: &Sre, tickets: i32) -> i32 {
    11000i32.wrapping_mul(tickets).wrapping_add(3000000)
}

// 55e7:0713
pub fn buy_tickets(g: &mut Sre, _p: &mut Player, e: usize) {
    let mut max = g.emp[e].money.get() / 10000;
    let lim = |v: i32| v as i16 as i32;
    if lim(32000 - g.galaxy.lotterytickets as i32) < max {
        max = lim(32000 - g.galaxy.lotterytickets as i32);
    }
    if lim(2000 - g.emp[e].tickets as i32) < max {
        max = lim(2000 - g.emp[e].tickets as i32);
    }
    if g.emp[e].year < 0 && lim(100 - g.emp[e].tickets as i32) < max {
        max = lim(100 - g.emp[e].tickets as i32);
    }
    kout!(g; "You currently have ", cnum(g), g.emp[e].tickets, cn(g), " tickets.\n");
    let jp = jackpot(g, g.galaxy.lotterytickets as i32);
    kout!(g; "The jackpot is currently ", cnum(g), jp, cn(g), " credits.\n");
    kout!(g; "At ", 10000i32, " credits each, how many tickets would you like to buy?  \n");
    put_max(g, max);
    set_defval(g, KIN, 0);
    set_maxval(g, KIN, max);
    let n = in_i16(g, KIN);
    if n <= 0 {
        return;
    }
    g.emp[e].money.sub(10000 * n as i32);
    g.emp[e].tickets = g.emp[e].tickets.wrapping_add(n);
    g.galaxy.lotterytickets = g.galaxy.lotterytickets.wrapping_add(n);
    kout!(g; "You have bought ", cnum(g), n, cn(g), " tickets for tomorrow's lottery.\n");
}

// 55e7:001d
pub fn lottery(g: &mut Sre, p: &mut Player, e: usize) {
    if g.galaxy.lotterywinner == g.emp[e].id {
        kout!(g; cn(g), "\nYou've won the ", col(14), "Super Daily Lottery", cn(g), "!!!\n");
        kout!(g; cnum(g), g.galaxy.jackpot, cn(g), " credits has been placed in your bank account.\n\n");
    } else if g.galaxy.lotterywinner != -1 && g.emp[g.galaxy.lotterywinner as usize].exists() {
        let w = g.galaxy.lotterywinner as usize;
        kout!(g; b'\n', cnum(g));
        let n = ename(g, w);
        put_untranslated(g, KOUT, &n);
        kout!(g; cn(g));
        kout!(g; " has won the Super Daily Lottery today!\n");
        kout!(g; "(", cnum(g), g.galaxy.jackpot, cn(g), " credits)\n");
    }
    if g.emp[e].money.get() < 2000 {
        return;
    }
    if g.system.regstate == RS_UNREGISTERED && !g.grb(&R_UNREGISTERED) {
        return;
    }
    kout!(g; col(14), "\nPick-5 Lottery\n", cn(g));
    kout!(g; "Would you like to buy a lottery ticket for ", cnum(g), 2000i16, cn(g));
    if yes_no(g, 0, b" credits") {
        let mut pick = [0u8; 5];
        let mut matches: i16 = 0;
        let mut prize: i32 = -1;
        g.emp[e].money.sub(2000);
        kout!(g; "Enter the five digits you would like to pick:  ", cin(g));
        let mut i = 0;
        while i < 5 {
            pick[i] = in_char(g, KIN);
            if pick[i] == b'\r' {
                pick[i] = (random(10) as u8).wrapping_add(b'0');
            }
            if (pick[i] as i8) < b'0' as i8 || (pick[i] as i8) > b'9' as i8 {
                continue;
            }
            kout!(g; pick[i]);
            i += 1;
        }
        kout!(g; cn(g), "\nThe computer picks: ");
        for _ in 0..5 {
            kout!(g; col(2), 0xfau8);
            b_delay(100);
            kout!(g; col(10), b"\x08\xf9");
            b_delay(100);
            kout!(g; col(11), b"\x08\xfe");
            b_delay(100);
            let c = (random(10) as u8).wrapping_add(b'0');
            kout!(g; col(13));
            for j in 0..5 {
                if pick[j] == c {
                    pick[j] = 0;
                    matches += 1;
                    kout!(g; col(15));
                    break;
                }
            }
            kout!(g; 8u8, c);
        }
        kout!(g; cn(g), b'\n');
        match matches {
            0 => prize = 0,
            1 => prize = 500,
            2 => prize = 10000,
            3 => prize = 400000,
            4 => prize = 2500000,
            5 => prize = 10000000,
            _ => {}
        }
        if prize > 0 {
            kout!(g; col(14), "\nCongratulations!\n");
            kout!(g; cn(g), "You have won ", cnum(g), prize, cn(g), " credits.\n");
            kout!(g; "The money will be placed in your bank account.\n\n");
            if g.emp[e].bank.get() < 2000000000 {
                g.emp[e].bank.add(prize);
            } else {
                kout!(g; "The bank is full!\n");
                if g.emp[e].money.get() < 2000000000 {
                    kout!(g; "You receive cash.\n");
                    g.emp[e].money.add(prize);
                }
            }
        } else {
            kout!(g; col(12), "Sorry", cn(g));
            kout!(g; "\nYou didn't win this time.  Better luck next time!\n\n");
        }
    }
    if g.system.regstate != RS_UNREGISTERED && yes_no(g, 1, b"Would you like to buy Super Daily Lottery Tickets") {
        buy_tickets(g, p, e);
    }
}
