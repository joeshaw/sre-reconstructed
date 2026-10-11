// Battle reports and attack news (segment 3f62).

use crate::prelude::*;

pub const BATTLE_FILE: &[u8] = b"Data/Battles"; // DS:0d5c

// A report about attacks (vtable 0f63).
#[allow(dead_code)] // unused, as in the original
pub fn battle_report(g: &mut Sre) -> Sid {
    report_out(g, Some(BATTLE_FILE)) // 3f62:0000
}
pub fn battle_report_to(g: &mut Sre, id: i16) -> Sid {
    report_out_to(g, id, Some(BATTLE_FILE)) // 3f62:0047
}

// 3f62:00cb: what the victim of an attack is told.
pub fn attack_message(g: &mut Sre, o: Sid, attacker: &[u8], severity: i16) {
    if severity > 0 {
        out!(g, o; col(14));
    } else {
        out!(g, o; cemp(g));
    }
    out!(g, o; attacker.to_vec(), cn(g), b' ');
    let s = if severity > 10 {
        "demolished your empire and spread terror throughout the land!"
    } else if severity > 4 {
        "invaded your empire and caused devastation!"
    } else if severity > 2 {
        "caused considerable damage to your land in a fierce attack."
    } else if severity > 0 {
        "attacked you and defeated your forces."
    } else if severity > -3 {
        "attacked but was repelled by your forces."
    } else if severity > -6 {
        "attempted an invasion but was unable to penetrate your defenses."
    } else {
        "made a feeble attempt to invade your lands."
    };
    out!(g, o; s);
}

// 3f62:01c1
pub fn show_battle_reports(g: &mut Sre, p: &Player, e: usize) -> i16 {
    let id = g.emp[e].id;
    show_reports(g, id, BATTLE_FILE, has_pref(p.prefs2, PF2_REPORTDATES), 4, 12)
}

// 3f62:01f5
pub fn clear_battle_reports(g: &mut Sre) {
    clear_stream_file(g, Some(BATTLE_FILE));
}

// 3f62:0209: the news of an attack.
pub fn attack_news(g: &mut Sre, a: usize, d: usize, success: bool, severity: i16) {
    if success {
        out!(g, NEWS; cemp(g), uname(g, a), cn(g));
        if g.emp[d].exists() && total_planets(g, d) > 0 && g.emp[d].population > 0 {
            let s = if severity > 10 {
                " pulverized"
            } else if severity > 8 {
                " devastated"
            } else if severity > 4 {
                " demolished"
            } else if severity > 2 {
                " invaded"
            } else {
                " defeated"
            };
            out!(g, NEWS; s);
        } else {
            out!(g, NEWS; " completely destroyed");
        }
        if !g.grb(&R_ATTACKVICTIM) {
            out!(g, NEWS; " a");
        }
    } else {
        out!(g, NEWS; cemp(g), uname(g, a), cn(g), " unsuccessfully attacked");
        if !g.grb(&R_ATTACKVICTIM) {
            out!(g, NEWS; " a");
        }
    }
    if g.grb(&R_ATTACKVICTIM) {
        out!(g, NEWS; b' ');
        if success {
            out!(g, NEWS; col(9));
        } else {
            out!(g, NEWS; col(11));
        }
        out!(g, NEWS; uname(g, d), cn(g), ".\n");
        return;
    }
    if total_planets(g, a) > total_planets(g, d) && net_worth(g, a) > net_worth(g, d) {
        out!(g, NEWS; " smaller");
    } else if total_planets(g, a) < total_planets(g, d) && net_worth(g, a) < net_worth(g, d) {
        out!(g, NEWS; " larger");
    } else {
        out!(g, NEWS; "nother");
    }
    out!(g, NEWS; " empire.\n");
}
