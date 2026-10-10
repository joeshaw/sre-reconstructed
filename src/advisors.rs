// The advisors' reports (segment 3ab7).

use crate::help::show_help_topic2;
use crate::prelude::*;

const C_ADVISOR: Color = col(10); // DS:0094 (3ab7:0c22)

fn advisor(g: &mut Sre, title: &str) {
    kout!(g; "Your ", C_ADVISOR, title, cn(g));
}

// 3ab7:0000
pub fn advisors(g: &mut Sre, _p: &mut Player, e: usize) {
    let hint = b"hint";
    kout!(g; b'\n');
    let need = crate::foodmarket::food_needed(g, e, 0);
    let grown = g.emp[e].planets[P_FOOD].count * g.emp[e].planets[P_FOOD].b;
    let mut food = g.emp[e].food;
    advisor(g, "Agricultural Advisor");
    kout!(g; " reports that your food planets produce ", cnum(g), grown, cn(g),
          " megatons of food, and your empire requires ", cnum(g), need, cn(g), " megatons.");
    let mut years: i16 = 0;
    while food > 0 && years < 30 {
        years += 1;
        food = food.wrapping_add(grown.wrapping_sub(need));
    }
    if years > 0 && years < 30 {
        kout!(g; "  Your food will last for ", cnum(g), years, cn(g), " years.");
    }
    kout!(g; "\n\n");
    if random(10) == 5 {
        advisor(g, "Economic Advisor");
        kout!(g; " is not in her office.  Her employees say that she is out playing tennis, and she will be back shortly.\n\n");
    } else {
        advisor(g, "Economic Advisor");
        kout!(g; "'s report states that your empire has an efficiency of ", cnum(g), g.emp[e].d1b4, cn(g),
              " credits per planet (profit).");
        let d = g.emp[e].d1b4;
        if d > 2000 {
            kout!(g; "  Congratulations!");
        } else if d > 1500 {
            kout!(g; "  That is a respectable amount of profit.  You are a wise ruler!");
        } else if d < 0 {
            kout!(g; "  This is not very good for your empire!!!");
        }
        let x = &g.emp[e];
        let money = x.planets[P_TOURISM].count
            + x.planets[P_PETROLEUM].count
            + x.planets[P_ORE].count
            + x.planets[P_URBAN].count
            + x.planets[P_EDUCATION].count / 2;
        if 2 * money < total_planets(g, e) {
            kout!(g; "  In her opinion, you have too few money-producing planets.");
        }
        let r1 = random(10);
        let r2 = random(10);
        if r1 + r2 + 30 < g.emp[e].taxrate as i16 {
            kout!(g; "  She thinks that you tax your people too much.");
        }
        let r1 = random(5);
        let r2 = random(5);
        if r1 + r2 > g.emp[e].taxrate as i16 {
            kout!(g; "  She thinks that you should tax your people to raise revenue for the empire.");
        }
        kout!(g; "\n\n");
    }
    if g.emp[e].year < 0 {
        // Advice for empires still under protection.
        let small = b"hint-small-empire";
        let x = &g.emp[e];
        if total_planets(g, e) < 100 {
            show_help_topic2(g, small, b"no-planets");
        } else if x.year > -5 {
            show_help_topic2(g, small, b"almost-ready");
        } else if total_planets(g, e) / 2
            > x.planets[P_TOURISM].count + x.planets[P_ORE].count + x.planets[P_PETROLEUM].count + x.planets[P_URBAN].count
        {
            show_help_topic2(g, small, b"no-money");
        } else if x.cmdship == 0 {
            show_help_topic2(g, small, b"command-ship");
        } else if x.agents == 0 {
            show_help_topic2(g, small, b"covert-agents");
        }
        return;
    }
    let sick = g.emp[e].year % 10 == 3;
    if !sick && (g.emp[e].soldiers + 49) / 50 > g.emp[e].generals {
        advisor(g, "Chief of Military Maneuvers");
        kout!(g; " asks you to buy more generals to command your forces.  You have ", cnum(g), g.emp[e].soldiers, cn(g),
              " soldiers and your generals can lead ", cnum(g), 50i32.wrapping_mul(g.emp[e].generals), cn(g), " to battle.\n\n");
    }
    if !sick && g.emp[e].carriers + (((g.emp[e].weapons >> 3) & 1) as i32) < (g.emp[e].fighters + 99) / 100 {
        advisor(g, "Chief of Military Maneuvers");
        kout!(g; " asks you to buy more carriers to carry your fighters.  You have ", cnum(g), g.emp[e].fighters, cn(g),
              " fighters and your carriers can take ", cnum(g), 100i32.wrapping_mul(g.emp[e].carriers), cn(g), " to battle.\n\n");
    }
    if sick {
        advisor(g, "Chief of Military Maneuvers");
        kout!(g; " asks you if you have any asprin.  He seems to be ill.\n\n");
    }
    if g.emp[e].agents > 0 {
        let tp = total_planets(g, e);
        let ag = g.emp[e].agents;
        let mut a = (ag + tp / 2) / (tp + 1);
        let mut b = (tp + ag / 2) / (ag + 1);
        if a < 1 {
            a = 1;
        }
        if b < 1 {
            b = 1;
        }
        advisor(g, "Chief of Intelligence");
        kout!(g; " reports that your covert agent / planet ratio is ", cnum(g), a, cn(g), b':', cnum(g), b, cn(g), ".\n\n");
    }
    if g.emp[e].d1b0 > 100 || g.emp[e].d1b0 < -50 {
        let un = if g.emp[e].d1b0 < 0 { "un" } else { "" };
        advisor(g, "Chief of Relations");
        kout!(g; " says that the Galactic Coordinator looks ", un, "favorably upon you.\n\n");
    }
    if g.emp[e].insurgency > 0 {
        show_help_topic2(g, hint, b"insurgency");
    }
    if g.emp[e].taxrate == 23 {
        show_help_topic2(g, hint, b"tax-rate");
    }
    if g.emp[e].soldiers == 0 {
        show_help_topic2(g, hint, b"soldiers");
    }
    if g.emp[e].stations == 0 {
        show_help_topic2(g, hint, b"defense-stations");
    }
    if g.emp[e].d188 != 0 {
        show_help_topic2(g, hint, b"pollution");
    }
    if g.emp[e].w172 == 0 {
        show_help_topic2(g, hint, b"lottery");
    }
    if g.emp[e].d293 == 0 {
        show_help_topic2(g, hint, b"research");
    }
    pause_prompt(g);
}
