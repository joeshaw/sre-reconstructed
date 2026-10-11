// Double agents planted by other empires (segment 432f).

use crate::covert::{opmask, set_opmask, OP_BRIBE};
use crate::prelude::*;

fn spies(g: &Sre, o: usize, e: usize) -> bool {
    g.emp[o].exists() && mask_test(opmask(g, o, OP_BRIBE), g.emp[e].id)
}

fn opmask_clear(g: &mut Sre, o: usize, op: i16, bit: i16) {
    let mut m = opmask(g, o, op);
    mask_clear(&mut m, bit);
    set_opmask(g, o, op, m);
}

// 432f:006f
pub fn double_agents(g: &mut Sre, p: &mut Player, e: usize) {
    let mut n = g.emp[e].agents;
    for i in 0..25 {
        if spies(g, i, e) {
            n -= 1;
        }
    }
    while n < 0 {
        let i = random(25) as usize;
        if spies(g, i, e) {
            let id = g.emp[e].id;
            opmask_clear(g, i, OP_BRIBE, id);
            n += 1;
        }
    }
    if g.emp[e].agents == n {
        return;
    }
    if random(3) != 0 {
        return;
    }
    if random(3) == 0 && show_screen(g, None, b"TwoTimer", false) {
        pause_prompt(g);
    }
    loop {
        let mut cost = [0i32; 4];
        let agents = g.emp[e].agents;
        cost[1] = 430i32.wrapping_mul(agents);
        cost[2] = agents.wrapping_add(100);
        cost[3] = 1037i32.wrapping_mul(agents);
        let mut m = Menu::new(g, b"DoubleSpy", b"ALERT!  Double Agents", [0xe, 5, 5, 0xe, 7, 0xf, 7, 7, 7, 7]);
        // 432f:0000
        m.prompt = Some(Box::new(move |g: &mut Sre| {
            kout!(g; "Your Intelligence Agency has determined that at least one covert agent ");
            kout!(g; "is actually a double-spy, spying for another empire.\n");
            kout!(g; "You have ", cnum(g), g.emp[e].money.get(), cn(g), " credits.\n");
            1
        }));
        m.add(g, b"Find a double-spy", b'1', 1);
        m.add(g, b"Eliminate all spies", b'2', 2);
        m.add(g, b"Bribe a double spy", b'3', 3);
        m.add(g, b"Ignore this threat", b'Q', 0x63);
        for i in 1..=3 {
            m.find(i).unwrap().add_column_num(cost[i as usize]);
        }
        if g.system.regstate != RS_UNREGISTERED {
            m.add(g, b"", b'V', 0x50);
        }
        let r = m.run(g, true, -1);
        if (1..=3).contains(&r) && cost[r as usize] > g.emp[e].money.get() {
            kout!(g; "You can not afford to do that!\n");
            continue;
        }
        match r {
            1 => {
                g.emp[e].money.sub(cost[1]);
                n = 0;
                let mut found: i32 = 0;
                for i in 0..25usize {
                    if spies(g, i, e) && random(3) != 0 {
                        mask_set(&mut found, i as i16);
                        let id = g.emp[e].id;
                        opmask_clear(g, i, OP_BRIBE, id);
                        n += 1;
                    }
                }
                for i in 0..25usize {
                    if mask_test(found, i as i16) {
                        kout!(g; "You found an agent from ", cemp(g), uname(g, i), cn(g), ".\n");
                    }
                }
                kout!(g; "You found a total of ", col(9), n, cn(g));
                if n == 1 {
                    kout!(g; " double-spy.\n");
                } else {
                    kout!(g; " double-spies.\n");
                }
                return;
            }
            2 => {
                if !yes_no(g, 0, b"Are you sure") {
                    continue;
                }
                g.emp[e].money.sub(cost[2]);
                // The count continues from the agents left after the
                // double-spies were taken out above.
                for i in 0..25usize {
                    if g.emp[i].exists() {
                        let id = g.emp[e].id;
                        opmask_clear(g, i, OP_BRIBE, id);
                        n += 1;
                    }
                }
                g.emp[e].agents = 0;
                kout!(g; "You killed ", cnum(g), n, cn(g));
                if n == 1 {
                    kout!(g; "double-spy");
                } else {
                    kout!(g; "double-spies");
                }
                kout!(g; ".\n");
                return;
            }
            3 => {
                g.emp[e].money.sub(cost[3]);
                let mut count: i16 = 0;
                let mut found: i32 = 0;
                for i in 0..25usize {
                    if spies(g, i, e) && random(3) != 0 {
                        mask_set(&mut found, i as i16);
                        count += 1;
                    }
                }
                let mut pick = random(count);
                for i in 0..25usize {
                    if !mask_test(found, i as i16) {
                        continue;
                    }
                    if pick > 0 {
                        pick -= 1;
                        continue;
                    }
                    let id = g.emp[e].id;
                    opmask_clear(g, i, OP_BRIBE, id);
                    let oid = g.emp[i].id;
                    let mut m = opmask(g, e, OP_BRIBE);
                    mask_set(&mut m, oid);
                    set_opmask(g, e, OP_BRIBE, m);
                    kout!(g; "You hired an agent from ", cemp(g), uname(g, i), cn(g), ".\n");
                    break;
                }
                return;
            }
            0x50 => {
                crate::bank::bank(g, p, e);
                continue;
            }
            0x63 => return,
            _ => {}
        }
    }
}
