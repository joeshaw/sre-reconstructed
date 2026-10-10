// Hourly and daily maintenance, and the newspaper (segment 5681).

use crate::prelude::*;

static R_DELETEDEAD: GRes = GRes::num(b"DeleteDeadPlayers", 1); // 5544
static R_PTSOLAR: GRes = GRes::num(b"Points.SolarMaster", 5); // 5550
static R_PTWEAPONS: GRes = GRes::num(b"Points.WeaponsMaster", 1); // 555c
static R_PTPLANET: GRes = GRes::num(b"Points.PlanetMaster", 1); // 5568

// 5681:0000
fn status(g: &mut Sre, msg: &[u8]) {
    put_untranslated(g, KOUT, msg);
}

// 5681:004a: erase a status message.
fn status_end(g: &mut Sre, msg: &[u8]) {
    for _ in 0..msg.len() {
        kout!(g; "\x08 \x08");
    }
}

// 5681:0079
fn backup(g: &mut Sre, from: &[u8], to: &[u8]) {
    use std::io::{Read, Write};
    let mut inp = match std::fs::File::open(host_path(from)) {
        Ok(f) => f,
        Err(_) => return,
    };
    let mut outp = match std::fs::File::create(host_path(to)) {
        Ok(f) => f,
        Err(_) => {
            out!(g, ERRLOG; "Could not generate backup file for ", from.to_vec(), ".\n");
            return;
        }
    };
    let mut buf = [0u8; 0x400];
    loop {
        let n = match inp.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        if outp.write_all(&buf[..n]).is_err() {
            out!(g, ERRLOG; "Error while backing up ", from.to_vec(), ".\n");
        }
    }
}

// 5681:0160: one of three newspapers, with random headlines.
fn newspaper(g: &mut Sre) {
    let paper = g.galaxy.w3c0;
    let today = g.time.today;
    if paper == 2 {
        out!(g, NEWS; b'\\', col(14), "   Farmer Bob's Guide to the Galaxy        ", col(7), "Printed Annually\n");
        let mut bar = vec![0xc4u8; 75];
        bar.extend_from_slice(b"\n\n");
        out!(g, NEWS; b'\\', col(12), bar, cn(g));
        if random(3) == 0 {
            match random(3) {
                0 => out!(g, NEWS; "It looks like a good year for crops.\n"),
                1 => out!(g, NEWS; "The space storm passing through this part of the galaxy may lead to crop failures!\n"),
                2 => out!(g, NEWS; "Eat your broccoli!\n"),
                _ => {}
            }
        }
        if random(7) == 0 {
            match random(4) {
                0 => out!(g, NEWS; "This year will be great for Arcadian sheep farmers.\n"),
                1 => {
                    out!(g, NEWS; "Galactic tollway ");
                    let n = random(999) + 1;
                    out!(g, NEWS; n, "'s construction may disrupt Unkulian onyx grazing planets.\n");
                }
                2 => out!(g, NEWS; "Overpopulation of Regalian Redfoxes may lead to lower fur prices.\n"),
                3 => out!(g, NEWS; "No news is good gnus.\n"),
                _ => {}
            }
        }
        if random(10) == 0 {
            g.galaxy.w3c0 = 3;
            out!(g, NEWS; "Centurion Rahad has come out of retirement to take over this news agency.\n");
        }
    } else if paper == 3 {
        out!(g, NEWS; b'\\', col(14), "   Centurion Rahad's Galactic Times        ", col(7));
        put_time(g, NEWS, today);
        out!(g, NEWS; b'\n');
        let mut bar = vec![0xcdu8; 75];
        bar.extend_from_slice(b"\n\n");
        out!(g, NEWS; b'\\', col(10), bar, cn(g));
        if random(5) == 0 {
            match random(3) {
                0 => out!(g, NEWS; "The Yip have been improving their weapon systems, much to the dismay of the Galactic Coordinator.\n"),
                1 => {
                    let letter = (random(26) as u8).wrapping_add(b'A');
                    let num = random(9999) + 1;
                    out!(g, NEWS; "Science Station ", num, letter, " is being converted into a military outpost.\n");
                }
                2 => out!(g, NEWS; "Military hardware prices may go up as skirmishes in the inner Antarean sector heat up.\n"),
                _ => {}
            }
        }
        if random(5) == 0 {
            match random(4) {
                0 => {
                    if g.galaxy.planetmaster != -1 {
                        let m = g.galaxy.planetmaster as usize;
                        out!(g, NEWS; cemp(g), uname(g, m), cn(g), " appears to be a good military target.\n");
                    }
                }
                1 => {
                    if g.galaxy.weaponsmaster != -1 {
                        let m = g.galaxy.weaponsmaster as usize;
                        out!(g, NEWS; cemp(g), uname(g, m), cn(g), " would be a strong opponent.\n");
                    }
                }
                2 => {
                    if g.galaxy.solarmaster != -1 {
                        let m = g.galaxy.solarmaster as usize;
                        out!(g, NEWS; "Computer psychohistory models predict that ", cemp(g), uname(g, m), cn(g),
                             " can be defeated by a united offense.\n");
                    }
                }
                3 => out!(g, NEWS; "All is quiet, unfortunately.\n"),
                _ => {}
            }
        }
        if random(10) == 0 {
            g.galaxy.w3c0 = 1;
            out!(g, NEWS; "Centurion Rahad is considering retiring!\n");
        }
    } else {
        out!(g, NEWS; b'\\', col(11), "   The Galactic Star                                         ", col(7));
        put_time(g, NEWS, today);
        out!(g, NEWS; b'\n');
        out!(g, NEWS; b'\\', col(9), "___________________________________________________________________________\n\n", cn(g));
        if random(10) == 0 {
            if random(3) == 0 {
                out!(g, NEWS; "The Galactic Coordinator steals ", col(12), "Reko's", col(7), " car!\n");
            } else if random(3) == 0 {
                out!(g, NEWS; "Covert agents kidnap ", col(12), "Reko's", col(7), " daughter!\n");
            } else if random(3) == 0 {
                out!(g, NEWS; "Rumors spread that ", col(12), "Emperor Reko", col(7),
                     " has picked Abraham Lincoln\n     as heir to his throne!\n");
            } else if random(10) == 0 {
                out!(g, NEWS; col(14), "Ross Perot", col(7), " denounces the use of free trade agreements.\n");
            }
        }
        if random(10) == 0 {
            if random(3) == 0 {
                out!(g, NEWS; "A band of pirates terrorizes the galaxy, news at 11.\n");
            } else if random(3) == 0 {
                out!(g, NEWS; "Bank managers complain as the number of loans has taken a sharp turn downwards.\n");
            } else if random(3) == 0 {
                out!(g, NEWS; "Lottery officials were found 'fixing' the numbers!\n");
            } else if random(10) == 0 {
                out!(g, NEWS; "Rumors are that investments are a good buy.\n");
            }
        }
        if random(10) == 0 {
            g.galaxy.w3c0 = 2;
            out!(g, NEWS; "The Galactic Star is on the verge of bankrupcy.  Can another newspaper take over?\n");
        }
    }
}

// 1c59:0855: planets come back onto the market over time.
pub fn regenerate_planets(g: &mut Sre) {
    let now = b_time();
    let x = &mut g.galaxy;
    if x.d392 < 0 {
        x.d392 = 0;
    }
    if now > x.t48 && x.w38c > 0 {
        if x.d392 < 30000 {
            let n = now.wrapping_sub(x.t48) / (x.w38c as i32 + 1);
            if n > 0 {
                x.d392 += n;
                x.t48 = x.t48.wrapping_add(n.wrapping_mul(x.w38c as i32));
            }
        } else {
            x.t48 = now;
        }
    }
}

// 5681:1f8f: a new day for an empire.
fn daily_empire(g: &mut Sre, e: usize) {
    galaxy_open(g);
    let gt = g.galaxy.grantturns;
    {
        let x = &mut g.emp[e];
        x.turnsperday = gt;
        x.turnsused = 0;
        x.w2a1 = x.w2a1.wrapping_sub(1);
        x.w229 = 0;
        x.d22f[3] = 0;
        x.d22f[4] = 0;
        x.d22f[6] = 0;
        for i in 0..MAXEMPIRES {
            if x.treaty[i].a > 0 {
                x.treaty[i].a -= 1;
            }
        }
    }
    let alive = g.emp[e].exists() && total_planets(g, e) > 0 && g.emp[e].population > 0;
    if g.grb(&R_DELETEDEAD) && !alive {
        out!(g, NEWS; "Player ", cemp(g), uname(g, e), cn(g), " has ");
        if total_planets(g, e) == 0 && g.emp[e].population == 0 {
            out!(g, NEWS; "abdicated!\n");
        } else {
            out!(g, NEWS; "collapsed!\n");
        }
        crate::sysop::delete_empire(g, e);
        empire_init(g, e);
    } else if g.grb(&R_DELETEDEAD) && g.emp[e].w2a1 <= 0 {
        let days = g.grl(&R_INACTIVETIMEOUT);
        out!(g, NEWS; "Empire ", cemp(g), uname(g, e), cn(g), " erased - inactive for ", days, " days.\n");
        crate::sysop::delete_empire(g, e);
        empire_init(g, e);
    }
    galaxy_close(g);
}

// 5681:0c52
pub fn maintenance(g: &mut Sre) {
    galaxy_open(g);
    if g.time.today > g.galaxy.enddate {
        return;
    }
    regenerate_planets(g);
    let now = g.time.now;
    if now.wrapping_sub(g.galaxy.t3ba) >= 3600 {
        let msg = b"Running hourly maintenance...";
        status(g, msg);
        if midnight(g.galaxy.t3ba) < g.galaxy.created {
            g.galaxy.t3ba = g.galaxy.created;
        }
        while now.wrapping_sub(g.galaxy.t3ba) > 3600 {
            for i in 0..MAXPIRATES {
                crate::pirates::pirate_hourly(g, i);
            }
            g.galaxy.t3ba = g.galaxy.t3ba.wrapping_add(3600);
        }
        status_end(g, msg);
    }
    if g.time.today.wrapping_sub(midnight(g.galaxy.created)) < 86400i32.wrapping_mul(g.galaxy.grantdays as i32) {
        galaxy_close(g);
        return;
    }
    put_untranslated(g, KOUT, b"Running daily maintenance...\n");
    let msg = b"Making backups...";
    status(g, msg);
    host_mkdir(b"Backup");
    for f in [
        &b"System.II"[..],
        b"Empire.II",
        b"Galaxy.II",
        b"Player.II",
        b"Pirate.II",
        b"Battles.II",
        b"Trading.II",
        b"CovertOp.II",
        b"Message.II",
        b"Report.II",
        b"Event.II",
    ] {
        backup(g, &cat(&[b"Data/", f]), &cat(&[b"Backup/", f]));
    }
    status_end(g, msg);

    let msg = b"Updating news files...";
    status(g, msg);
    host_mkdir(b"News");
    host_remove(b"News/NEWS9.MSG");
    for i in (0..=8u8).rev() {
        let mut from = b"NEWS/NEWS_.MSG".to_vec();
        let mut to = b"NEWS/NEWS_.MSG".to_vec();
        from[9] = i + b'0';
        to[9] = i + b'1';
        host_rename(&from, &to);
    }
    for i in 0..=9 {
        news_files(g, i);
    }
    newspaper(g);
    status_end(g, msg);
    crate::events::process_events(g);

    let msg = b"Searching for illegal nuclear bomb traders...";
    status(g, msg);
    {
        let x = &mut g.galaxy;
        if x.w3b2 > 400 {
            x.w3b2 += 2;
        } else if x.w3b2 > 300 {
            x.w3b2 += 5;
        } else {
            x.w3b2 += 15;
        }
        if x.w3b4 < 30 {
            x.w3b4 += 1;
        }
        x.d3c2 = 15 * (x.d3c2 / 20);
        if x.d3c2 < 5000000 {
            x.d3c2 = 5000000;
        }
    }
    status_end(g, msg);

    let msg = b"Checking top scores and lottery...";
    status(g, msg);
    let mut bestscore: i32 = 10000;
    let mut bestplanets: i32 = 300;
    let mut bestworth: i32 = 50000;
    let oldsolar = g.galaxy.solarmaster;
    let oldplanet = g.galaxy.planetmaster;
    let oldweapons = g.galaxy.weaponsmaster;
    g.galaxy.solarmaster = -1;
    g.galaxy.weaponsmaster = -1;
    g.galaxy.planetmaster = -1;
    let tickets = g.galaxy.w3a4;
    let mut winner = ((tickets as i32 * b_rand() as i32) / 0x8000) as i16;
    if g.galaxy.w3a4 <= 0 {
        winner = -1;
        g.galaxy.w3a4 = 0;
    }
    for i in 0..MAXEMPIRES {
        if !g.emp[i].exists() {
            continue;
        }
        if g.emp[i].lastplay < g.galaxy.t44 {
            g.emp[i].lastplay = g.galaxy.t44;
        }
        daily_empire(g, i);
    }
    for i in 0..MAXEMPIRES {
        if !g.emp[i].exists() {
            continue;
        }
        if winner >= 0 && g.emp[i].w172 > winner {
            g.galaxy.w3a6 = g.emp[i].id;
            g.galaxy.d3a8 = crate::lottery::jackpot(g, g.galaxy.w3a4 as i32);
            let j = g.galaxy.d3a8;
            if g.emp[i].bank.get() < 2000000000 {
                g.emp[i].bank.add(j);
            } else if g.emp[i].money.get() < 2000000000 {
                g.emp[i].money.add(j);
            }
            g.galaxy.w3a4 = 0;
            winner = -1;
        }
        if winner >= 0 {
            winner = winner.wrapping_sub(g.emp[i].w172);
        }
        g.emp[i].w172 = 0;
        let sc = empire_score(g, i);
        if sc > 3 * bestscore / 2 {
            bestscore = sc;
            g.galaxy.solarmaster = g.emp[i].id;
        } else if sc >= bestscore {
            g.galaxy.solarmaster = -1;
            bestscore = sc;
        }
        let tp = total_planets(g, i);
        if tp > 3 * bestplanets / 2 {
            bestplanets = tp;
            g.galaxy.planetmaster = g.emp[i].id;
        } else if tp >= bestplanets {
            g.galaxy.planetmaster = -1;
            bestplanets = tp;
        }
        let nw = net_worth(g, i);
        if nw > 3 * bestworth / 2 {
            bestworth = nw;
            g.galaxy.weaponsmaster = g.emp[i].id;
        } else if nw >= bestworth {
            g.galaxy.weaponsmaster = -1;
            bestworth = nw;
        }
    }
    g.galaxy.w3a4 = 0;
    if g.galaxy.solarmaster != -1 {
        if g.galaxy.d3ac < 0 {
            g.galaxy.d3ac = 0;
        }
        let e = g.galaxy.solarmaster as usize;
        let pts = g.grl(&R_PTSOLAR);
        g.emp[e].d178 = g.emp[e].d178.wrapping_add(pts);
        out!(g, NEWS; cemp(g), uname(g, e), cn(g), " has been ");
        if g.galaxy.solarmaster == oldsolar {
            out!(g, NEWS; "once again ");
        }
        out!(g, NEWS; "chosen today's Solar Master, and has received a ");
        let bonus = 10i32.wrapping_mul(g.galaxy.d3ac);
        out!(g, NEWS; cnum(g), bonus, cn(g), " credit bonus!\n");
        g.emp[e].money.add(bonus);
        g.galaxy.d3ac -= g.galaxy.d3ac / 100;
    }
    if g.galaxy.planetmaster != -1 {
        let e = g.galaxy.planetmaster as usize;
        let pts = g.grl(&R_PTPLANET);
        g.emp[e].d178 = g.emp[e].d178.wrapping_add(pts);
        out!(g, NEWS; cemp(g), uname(g, e), cn(g));
        if g.galaxy.planetmaster == oldplanet {
            out!(g, NEWS; " is still the");
        } else {
            out!(g, NEWS; " has been chosen today's");
        }
        out!(g, NEWS; " Planet Master!\n");
    }
    if g.galaxy.weaponsmaster != -1 {
        let mut lost = false;
        let e = g.galaxy.weaponsmaster as usize;
        let pts = g.grl(&R_PTWEAPONS);
        g.emp[e].d178 = g.emp[e].d178.wrapping_add(pts);
        if g.galaxy.weaponsmaster != oldweapons {
            out!(g, NEWS; cemp(g), uname(g, e), cn(g), " has been chosen the new Weapons Master");
        }
        if total_planets(g, e) > 500 {
            for i in 0..MAXEMPIRES {
                if g.emp[e].treaty[i].e() > 0 || g.emp[e].treaty[i].d() > 0 {
                    let a = g.emp[e].treaty[i].a;
                    lost = true;
                    let eid = g.emp[e].id as usize;
                    set_treaty(&mut g.emp[i].treaty[eid], 1, a);
                    let oid = g.emp[i].id;
                    if oid >= 0 {
                        set_treaty(&mut g.emp[e].treaty[oid as usize], 1, a);
                    }
                }
            }
        }
        if g.galaxy.weaponsmaster != oldweapons {
            if lost {
                out!(g, NEWS; ", and lost some defense pacts");
            }
            out!(g, NEWS; "!\n");
        }
    }
    status_end(g, msg);
    if g.system.regstate != 0 {
        if g.galaxy.interest < 3 {
            g.galaxy.interest += 2;
            out!(g, NEWS; "Interest rates go up to ", g.galaxy.interest, "% as the galaxy's economy booms!\n");
        } else if g.galaxy.interest > 10 {
            g.galaxy.interest -= 3;
            out!(g, NEWS; "Interest rates fall sharply to ", g.galaxy.interest, "% as pirate raids increase!\n");
        } else if random(8) == 0 {
            g.galaxy.interest -= 2;
            out!(g, NEWS; "The galaxy's interest rates fall to ", g.galaxy.interest, "% in response to a weakening economy.\n");
        } else if random(5) == 0 {
            g.galaxy.interest -= 1;
            out!(g, NEWS; "Bank interest rates fall slightly to ", g.galaxy.interest, "%.\n");
        } else if random(3) == 0 {
            g.galaxy.interest += 1;
            out!(g, NEWS; "Bank interest rates rise to ", g.galaxy.interest, "%.\n");
        }
    }
    g.galaxy.created = g.time.today;
    g.galaxy.t3ba = g.time.today;
    galaxy_close(g);
}
