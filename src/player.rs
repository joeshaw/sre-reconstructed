// Players (segment 64d2): one record per BBS user in Data/Player.II.

use crate::prelude::*;

const PLAYER_FILE: &[u8] = b"Data/Player.II"; // DS:8094

// 64d2:0253
pub fn player_reset(g: &Sre, p: &mut Player) {
    p.index = -1;
    p.user[0] = 0;
    p.name[0] = 0;
    p.b42[0] = 0;
    p.male = 0;
    p.prefs1 &= !(0x03 | PF1_SHORTMENUS | PF1_TERSE);
    p.prefs1 |= PF1_ASKDEPOSIT | PF1_DEPOSIT | PF1_ASKDONATE | PF1_DONATE;
    p.prefs2 |= PF2_ASKPAY | PF2_AUTOPAY;
    p.prefs2 &= !(PF2_BATTLEMSG | PF2_REPORTDATES | PF2_ENTERSTAYS);
    p.prefs2 &= 0x1f;
    p.w6e = 0;
    p.lastplay = g.time.now;
    p.created = g.time.now;
    p.seenplanets = 0;
}

fn read_into(f: &mut DataFile, rec: i32, p: &mut Player) -> bool {
    let mut b = p.to_bytes();
    let ok = f.read(rec, &mut b, 1);
    *p = Player::from_bytes(&b);
    ok
}

// 64d2:0000
pub fn player_load(g: &mut Sre, p: &mut Player, index: i16) {
    p.created = b_time();
    p.lastplay = b_time();
    player_reset(g, p);
    if index == -1 {
        return;
    }
    let mut f = DataFile::new(g, PLAYER_FILE, 0x72, 0, 1);
    if index as i32 > f.numrecs() {
        out!(g, ERRLOG; "Tried to open non-existent player #", index, "!\n");
        player_reset(g, p);
        return;
    }
    if !read_into(&mut f, index as i32, p) {
        if index as i32 != f.numrecs() {
            out!(g, ERRLOG; "Error reading player #", index, "!\n");
        }
        player_reset(g, p);
        p.index = index;
    }
    if p.name[0] == 0 {
        player_reset(g, p);
        p.index = index;
    }
}

// 64d2:0183
pub fn player_save(g: &mut Sre, p: &mut Player) {
    if p.index == -1 {
        return;
    }
    let mut f = DataFile::new(g, PLAYER_FILE, 0x72, 0, 1);
    if p.index as i32 > f.numrecs() {
        out!(g, ERRLOG; "Error writing invalid player #", p.index, "!\n");
    } else {
        let b = p.to_bytes();
        f.write(g, p.index as i32, &b, 1);
    }
}

// 64d2:0320
pub fn player_set_name(p: &mut Player, s: &[u8]) {
    set_name(&mut p.name, s);
}

// 64d2:035a
pub fn player_set_user(p: &mut Player, s: &[u8]) {
    set_name(&mut p.user, s);
}

// 64d2:0394: he/him/his (type 0/1/2) for male, she/her/her otherwise.
pub fn pronoun(g: &mut Sre, typ: i16, male: i16) -> Str {
    let s: &[u8] = match typ {
        0 => {
            if male == 1 {
                b"he"
            } else {
                b"she"
            }
        }
        1 => {
            if male == 1 {
                b"him"
            } else {
                b"her"
            }
        }
        2 => {
            if male == 1 {
                b"his"
            } else {
                b"her"
            }
        }
        _ => {
            out!(g, ERRLOG; "Unknown pronountype in Pronoun(", typ, ")\n");
            b"??it??"
        }
    };
    s.to_vec()
}

// 64d2:041f
pub fn player_pronoun(g: &mut Sre, p: &Player, typ: i16) -> Str {
    pronoun(g, typ, p.male)
}

// 64d2:0443
pub fn random_pronoun(g: &mut Sre, typ: i16) -> Str {
    let m = (((b_rand() as i32) << 1) / 0x8000) as i16;
    pronoun(g, typ, m)
}

// 64d2:0479: the record number of a BBS user, or -1.
pub fn player_find(g: &mut Sre, user: &[u8]) -> i16 {
    let mut p = Player::default();
    player_load(g, &mut p, -1);
    let mut f = DataFile::new(g, PLAYER_FILE, 0x72, 0, 1);
    let n = f.numrecs();
    for i in 0..n.max(0) {
        read_into(&mut f, i, &mut p);
        if str_eq_i2(user, cstr(&p.user)) {
            return i as i16;
        }
    }
    -1
}

// 64d2:05d6: the first free record.
pub fn player_free(g: &mut Sre) -> i16 {
    let mut p = Player::default();
    player_load(g, &mut p, -1);
    let mut f = DataFile::new(g, PLAYER_FILE, 0x72, 0, 1);
    let n = f.numrecs();
    let mut i: i32 = 0;
    while i < n {
        read_into(&mut f, i, &mut p);
        if p.name[0] == 0 {
            break;
        }
        i += 1;
    }
    i as i16
}

// 64d2:069c: list all players (sysop).
#[allow(dead_code)] // unused, as in the original
pub fn player_list(g: &mut Sre) {
    let mut p = Player::default();
    player_load(g, &mut p, -1);
    let mut f = DataFile::new(g, PLAYER_FILE, 0x72, 0, 1);
    let n = f.numrecs();
    for i in 0..n.max(0) {
        read_into(&mut f, i, &mut p);
        if p.name[0] == 0 {
            continue;
        }
        kout!(g; setw(5), cnum(g), i + 1, setw(0), b' ');
        kout!(g; setw(0x20), col(14), cs(&p.user));
        kout!(g; setw(0x20), col(15), cs(&p.name), setw(0), b'\n');
    }
}

// 64d2:090c
pub fn player_init_index(g: &Sre, p: &mut Player, index: i16) {
    player_reset(g, p);
    p.index = index;
}

// 64d2:0926
pub fn player_who_am_i(g: &mut Sre, p: &mut Player) {
    let t: &[u8] = if p.male != 0 { b"Emperor" } else { b"Emperess" };
    kout!(g; cn(g), "You are ", t);
    kout!(g; b' ', col(15), cs(&p.name), cn(g), ".\n");
}

// 64d2:0835: the usernames of all the players, for the sysop's lists.
pub fn player_list_names(g: &mut Sre, l: &mut PickList) {
    let mut p = Player::default();
    player_load(g, &mut p, -1);
    let mut f = DataFile::new(g, PLAYER_FILE, 0x72, 0, 1);
    let n = f.numrecs();
    for i in 0..n.max(0) {
        read_into(&mut f, i, &mut p);
        if p.name[0] != 0 {
            l.add(cstr(&p.user));
        }
    }
}
