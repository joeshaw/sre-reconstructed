// Trading fleets between empires (segment 6e3f).

use crate::layout::record;
use crate::messages::{Message, MESSAGE_CLASS};
use crate::prelude::*;
use std::any::Any;

const TRADE_FILE: &[u8] = b"Data/Trading"; // DS:aaa4
pub const TRADE_CLASS: &[u8] = b"TradingFleet"; // DS:aab1
pub static R_TRADING: GRes = GRes::num(b"Trading", 1); // aa0a
static R_TRADEEXPIRE: GRes = GRes::num(b"Trade.Expire", 4); // aa16

record! {
    // What a fleet carries (6e3f:0152).
    #[derive(Copy)]
    pub struct Goods {
        pub soldiers: i32,  // 00
        pub fighters: i32,  // 04
        pub cruisers: i32,  // 08
        pub generals: i32,  // 0c
        pub carriers: i32,  // 10
        pub credits: i32,   // 14
        pub food: i32,      // 18
        pub agents: i32,    // 1c
    }
}

// TradeData::state
pub const FS_DONE: i16 = 0;
pub const FS_SENT: i16 = 1;
pub const FS_ACCEPTED: i16 = 2;
pub const FS_REJECTED: i16 = 3;
pub const FS_RETURNING: i16 = 4;

record! {
    pub struct TradeData {
        pub created: i32,   // 00
        pub expires: i32,   // 04
        pub to: i16,        // 08
        pub from: i16,      // 0a
        pub state: i16,     // 0c FS_*
        pub offer: Goods,   // 0e
        pub demand: Goods,  // 2e
        pub carriers: i32,  // 4e carriers carrying the fleet
        pub hasmsg: i16,    // 52 a Message follows in the file
    }
}
const _: () = assert!(<TradeData as Le>::SIZE == 0x54);

// The items of the trading menus, by number (DS:aa5e, aa36, aa4a).
const ITEM: [&str; 10] = ["", "Soldiers", "Fighters", "", "Generals", "Money", "Food", "Agents", "Cruisers", "Carriers"];

fn empire_item(x: &mut Empire, i: i16) -> Option<&mut i32> {
    match i {
        1 => Some(&mut x.soldiers),
        2 => Some(&mut x.fighters),
        4 => Some(&mut x.generals),
        6 => Some(&mut x.food),
        7 => Some(&mut x.agents),
        8 => Some(&mut x.heavycruisers),
        9 => Some(&mut x.carriers),
        _ => None,
    }
}

fn goods_item(gd: &mut Goods, i: i16) -> Option<&mut i32> {
    match i {
        1 => Some(&mut gd.soldiers),
        2 => Some(&mut gd.fighters),
        4 => Some(&mut gd.generals),
        5 => Some(&mut gd.credits),
        6 => Some(&mut gd.food),
        7 => Some(&mut gd.agents),
        8 => Some(&mut gd.cruisers),
        9 => Some(&mut gd.carriers),
        _ => None,
    }
}

fn gi(gd: &Goods, i: i16) -> i32 {
    let mut c = *gd;
    goods_item(&mut c, i).map(|v| *v).unwrap_or(0)
}

// 6e3f:01c5
fn goods_value(gd: &Goods) -> i32 {
    gd.credits / 100000 + 7 * gd.agents + gd.soldiers + gd.fighters + 3 * gd.cruisers + 3 * gd.carriers + 2 * gd.generals
}

// 6e3f:0271
fn goods_list(gd: &Goods, l: &mut ItemList) {
    l.skip_zero();
    l.add(gd.food, b"megatons of food");
    l.add(gd.soldiers, b"soldier");
    l.add(gd.fighters, b"fighter");
    l.add(gd.cruisers, b"heavy cruiser");
    l.add(gd.generals, b"general");
    l.add(gd.carriers, b"carrier");
    l.add(gd.agents, b"covert agent");
    l.add(gd.credits, b"credit");
}

// 6e3f:03da: carriers needed besides those traded.
fn carriers_needed(gd: &Goods) -> i32 {
    let mut n = (gd.fighters + gd.generals + 3 * gd.agents + 149) / 150 + (gd.soldiers + 499) / 500 + (gd.food + 9999) / 10000;
    n -= gd.carriers;
    if n < 0 {
        n = 0;
    }
    n
}

fn trade_init(g: &mut Sre, t: &mut TradeData) {
    t.offer = Goods::default();
    t.demand = Goods::default();
    t.carriers = 0;
    t.to = -1;
    t.from = -1;
    t.state = FS_DONE;
    t.created = g.time.now;
    t.expires = t.created.wrapping_add(86400i32.wrapping_mul(g.grl(&R_TRADEEXPIRE)));
}

// 6e3f:05fe
fn fleet_carriers(t: &TradeData) -> i32 {
    let mut n = carriers_needed(&t.offer);
    if carriers_needed(&t.demand) > n {
        n = carriers_needed(&t.demand);
    }
    n
}

// Class "TradingFleet" (vtable b009).
pub struct TradingFleet {
    pub d: TradeData,
}

pub fn new_trading_fleet() -> Box<dyn Streamable> {
    Box::new(TradingFleet { d: TradeData::default() })
}

impl Streamable for TradingFleet {
    // 6e3f:0058
    fn class_name(&self) -> &'static [u8] {
        TRADE_CLASS
    }
    // 6e3f:0066
    fn size(&self) -> i16 {
        0x56
    }
    // 6e3f:0070
    fn expired(&mut self, _g: &mut Sre) -> bool {
        self.d.state == FS_DONE
    }
    // 6e3f:0088: a fleet to a deleted empire turns back; one from it is lost.
    fn on_empire_deleted(&mut self, g: &mut Sre, id: i16) {
        let d = &mut self.d;
        if d.to == id && d.state == FS_SENT {
            d.to = d.from;
            d.from = id;
            d.state = FS_RETURNING;
            if d.to != -1 && !g.emp[d.to as usize].exists() {
                d.state = FS_DONE;
            }
        }
        if d.from == id {
            if d.state == FS_SENT {
                d.state = FS_DONE;
            } else if d.state != FS_DONE && d.to != -1 && g.emp[d.to as usize].exists() {
                d.state = FS_RETURNING;
            }
        }
    }
    fn data(&self) -> Vec<u8> {
        self.d.to_bytes()
    }
    fn set_data(&mut self, b: &[u8]) {
        self.d = TradeData::rd(b);
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

// 6e3f:0658
fn ask_amount(g: &mut Sre, name: &str, v: &mut i32, max: i32, demand: bool) {
    kout!(g; "New amount of ", cnum(g), name, cn(g));
    if !demand {
        kout!(g; " to send?  ", cbr(g), b'[', cbo(g), "MAX=", max, cbr(g), b']', cn(g), "  ");
        set_maxval(g, KIN, max);
    } else {
        kout!(g; " to demand?  ");
    }
    set_defval(g, KIN, *v);
    if !demand {
        set_maxval(g, KIN, max);
    }
    *v = in_i32(g, KIN);
}

// 6e3f:0791
fn fleet_prompt(e: usize, t: TradeData) -> MenuHook {
    Box::new(move |g: &mut Sre| {
        let n = fleet_carriers(&t);
        kout!(g; "Current fleet requires ", cnum(g), n, cn(g), " carriers.\n");
        let more = n + t.offer.carriers - g.emp[e].carriers;
        if more > 0 {
            kout!(g; "This is ", col(14), more, cn(g), " more carriers than you have.\n");
        }
        1
    })
}

fn item_header(g: &mut Sre, m: &mut Menu) {
    m.add(g, b"Item", 0, 100);
    let h = m.find(100).unwrap();
    h.colors = [7; 8];
    h.load_colors(g);
}

// 6e3f:087f: set up the offer and the demands.
fn make_offer(g: &mut Sre, e: usize, t: &mut TradeData) -> bool {
    loop {
        let mut m = Menu::new(g, b"Trading.Offer", b"Your Offer", [0xe, 6, 4, 0xe, 7, 0xa, 0xf, 7, 7, 7]);
        m.prompt = Some(fleet_prompt(e, t.clone()));
        item_header(g, &mut m);
        m.find(100).unwrap().add_column_str(b"Sending");
        m.find(100).unwrap().add_column_str(b"Owned");
        for i in 1..=9i16 {
            if goods_item(&mut t.offer, i).is_none() {
                continue;
            }
            m.add(g, ITEM[i as usize].as_bytes(), b'0' + i as u8, i);
            let sending = gi(&t.offer, i);
            let mut owned = if i == 5 { g.emp[e].money.get() } else { *empire_item(&mut g.emp[e], i).unwrap() };
            owned = owned.wrapping_sub(sending);
            let it = m.find(i).unwrap();
            it.add_column_num(sending);
            it.add_column_num(owned);
        }
        let r = m.run(g, true, -1);
        if (1..=9).contains(&r) && r != 3 {
            if r == 5 {
                let max = g.emp[e].money.get();
                let mut v = t.offer.credits;
                ask_amount(g, ITEM[5], &mut v, max, false);
                t.offer.credits = v;
            } else {
                let max = *empire_item(&mut g.emp[e], r).unwrap();
                let mut v = gi(&t.offer, r);
                ask_amount(g, ITEM[r as usize], &mut v, max, false);
                *goods_item(&mut t.offer, r).unwrap() = v;
            }
        }
        if r == -1 {
            break;
        }
    }
    if fleet_carriers(t) + t.offer.carriers > g.emp[e].carriers {
        kout!(g; "You don't have enough carriers to send the fleet.\n");
        return false;
    }
    loop {
        let mut m = Menu::new(g, b"Trading.Demand", b"Your Demands", [0xe, 6, 4, 0xe, 7, 0xc, 7, 7, 7, 7]);
        m.prompt = Some(fleet_prompt(e, t.clone()));
        item_header(g, &mut m);
        m.find(100).unwrap().add_column_str(b"Demand");
        for i in 1..=9i16 {
            if goods_item(&mut t.demand, i).is_none() {
                continue;
            }
            m.add(g, ITEM[i as usize].as_bytes(), b'0' + i as u8, i);
            let v = gi(&t.demand, i);
            m.find(i).unwrap().add_column_num(v);
        }
        let r = m.run(g, true, -1);
        if (1..=9).contains(&r) && r != 3 {
            let mut v = gi(&t.demand, r);
            ask_amount(g, ITEM[r as usize], &mut v, 0, true);
            *goods_item(&mut t.demand, r).unwrap() = v;
        }
        if r == -1 {
            break;
        }
    }
    if fleet_carriers(t) + t.offer.carriers > g.emp[e].carriers {
        kout!(g; "You don't have enough carriers to send the fleet.\n");
        return false;
    }
    t.carriers = fleet_carriers(t);
    true
}

// 6e3f:0fd8: the empires one has relations with.
fn choose_partner(g: &mut Sre, e: usize) -> i16 {
    let mut allowed = empire_mask(g) & !protected_mask(g);
    for i in 0..25 {
        // (The original tests the trading empire itself, not empire i.)
        if g.emp[e].treaty[i].typ() == TT_NONE || !g.emp[e].exists() || total_planets(g, e) <= 0 || g.emp[e].population <= 0 {
            mask_clear(&mut allowed, i as i16);
        }
    }
    choose_empire(g, allowed)
}

// 6e3f:108f
fn send_trade(g: &mut Sre, e: usize) -> bool {
    let mut t = TradeData::default();
    trade_init(g, &mut t);
    let to = choose_partner(g, e);
    if to == -1 {
        return false;
    }
    if !make_offer(g, e, &mut t) {
        return true;
    }
    if !yes_no(g, 1, b"Send this trade deal") {
        return true;
    }
    for i in 1..=9i16 {
        let o = gi(&t.offer, i);
        if let Some(v) = empire_item(&mut g.emp[e], i) {
            *v = v.wrapping_sub(o);
        }
    }
    g.emp[e].carriers -= t.carriers;
    g.emp[e].money.sub(t.offer.credits);
    g.emp[e].tradegoods += goods_value(&t.offer);
    t.to = to;
    t.from = g.emp[e].id;
    t.state = FS_SENT;
    let mut fleet = TradingFleet { d: t };
    fleet.d.hasmsg = 0;
    let mut msg = false;
    if g.grb(&crate::messages::R_MESSAGES) && yes_no(g, 0, b"Attach a message") {
        msg = crate::messages::compose_message(g, e, to);
    }
    if msg {
        fleet.d.hasmsg = 1;
    }
    append_object(g, TRADE_FILE, &mut fleet);
    if msg {
        let mut m = g.m.message.take().unwrap();
        append_object(g, TRADE_FILE, m.as_mut());
        g.m.message = Some(m);
    }
    kout!(g; "Trading fleet sent.\n");
    true
}

// 6e3f:1290: can the empire give what is demanded?
fn can_meet(g: &mut Sre, t: &TradeData, e: usize) -> bool {
    for i in 1..=9i16 {
        let need = gi(&t.demand, i).wrapping_sub(gi(&t.offer, i));
        if let Some(v) = empire_item(&mut g.emp[e], i) {
            if need > *v {
                return false;
            }
        }
    }
    let mut m = g.emp[e].money;
    m.add(g.emp[e].bank.get());
    t.demand.credits.wrapping_sub(t.offer.credits) <= m.get()
}

fn key(g: &mut Sre, c: u8) {
    kout!(g; cbr(g), b'[', cbo(g), c, cbr(g), b']', cn(g));
}

// 6e3f:1394: a fleet reaches the empire; returns true to keep it.
fn arrive_trade(g: &mut Sre, e: usize, t: &mut TradeData) -> bool {
    if !g.grb(&R_TRADING) {
        return false;
    }
    match t.state {
        FS_DONE => false,
        FS_SENT => {
            let o = t.from as usize;
            kout!(g; cemp(g), uname(g, o), cn(g), " has sent a trading deal:\n");
            kout!(g; "In return for ");
            {
                let mut l = ItemList::new(cn(g), cnum(g));
                goods_list(&t.offer, &mut l);
                put_item_list(g, KOUT, &l);
                kout!(g; ", you must send back ");
            }
            {
                let mut l = ItemList::new(cn(g), cnum(g));
                goods_list(&t.demand, &mut l);
                put_item_list(g, KOUT, &l);
                kout!(g; ".\n");
            }
            t.state = FS_REJECTED;
            let mut c: u8;
            if g.emp[e].year < 0 {
                kout!(g; "However, since you are in protection, you cannot trade with other empires.\n");
                c = b'N';
            } else if !can_meet(g, t, e) {
                kout!(g; "However, you can not meet the demands right now.\n");
                kout!(g; "Do you wish to ");
                key(g, b'K');
                kout!(g; "eep for later or ");
                key(g, b'R');
                kout!(g; "eject?  ", cin(g));
                c = get_choice(g, b"KR", true);
                if c == b'K' {
                    kout!(g; "Keep for later");
                    c = b'I';
                } else if c == b'R' {
                    kout!(g; "Reject");
                    c = b'N';
                }
                kout!(g; cn(g), b'\n');
            } else if g.emp[e].treaty[t.from as usize].typ() == TT_NONE {
                kout!(g; "However, you no longer have any relations with that empire.\n");
                c = b'N';
                t.state = FS_RETURNING;
            } else {
                kout!(g; "Accept this deal?  ");
                key(g, b'Y');
                kout!(g; "es, ");
                key(g, b'N');
                kout!(g; "o, or ");
                key(g, b'I');
                kout!(g; "gnore for now:  ", cin(g));
                c = get_choice(g, b"YNI", true);
                if c == b'Y' {
                    kout!(g; "Accept\n");
                } else if c == b'N' {
                    kout!(g; "Reject Offer\n");
                } else if c == b'I' {
                    kout!(g; "Ignore for now\n");
                }
            }
            if c == b'N' {
                t.to = t.from;
                t.from = g.emp[e].id;
                return true;
            }
            if c == b'I' {
                t.state = FS_SENT;
                t.created = t.created.wrapping_add(2);
                return true;
            }
            for i in 1..=9i16 {
                let d = gi(&t.demand, i).wrapping_sub(gi(&t.offer, i));
                if let Some(v) = empire_item(&mut g.emp[e], i) {
                    *v = v.wrapping_sub(d);
                }
            }
            g.emp[o].tradegoods -= goods_value(&t.offer);
            g.emp[e].tradegoods += goods_value(&t.demand);
            g.emp[e].money.sub(t.demand.credits.wrapping_sub(t.offer.credits));
            if g.emp[e].money.get().wrapping_sub(t.demand.credits).wrapping_add(t.offer.credits) < 0 {
                let m = g.emp[e].money.get();
                g.emp[e].bank.add(m);
                g.emp[e].money.set(0);
            }
            t.state = FS_ACCEPTED;
            t.to = t.from;
            t.from = g.emp[e].id;
            true
        }
        FS_ACCEPTED => {
            // The fleet returns with what was demanded.
            let o = t.from as usize;
            if g.emp[o].exists() {
                kout!(g; cnum(g), uname(g, o), cn(g), " accepted your trade agreement.\n");
            } else {
                kout!(g; "An unknown empire accepted your trade agreement.\n");
            }
            for i in 1..=9i16 {
                let d = gi(&t.demand, i);
                if let Some(v) = empire_item(&mut g.emp[e], i) {
                    *v = v.wrapping_add(d);
                }
            }
            while money_overflow(g.emp[e].money.get(), t.demand.credits) {
                g.emp[e].bank.add(1000000);
                t.demand.credits -= 1000000;
            }
            g.emp[e].money.add(t.demand.credits);
            g.emp[e].carriers += t.carriers;
            g.emp[o].tradegoods -= goods_value(&t.demand);
            false
        }
        FS_REJECTED | FS_RETURNING => {
            // The offer comes back.
            let o = t.from as usize;
            if g.emp[o].exists() && t.state == FS_REJECTED {
                kout!(g; cnum(g), uname(g, o), cn(g), " rejected your trade agreement.\n");
            } else {
                kout!(g; "A trading fleet was not able to continue, and has returned.\n");
            }
            for i in 1..=9i16 {
                let d = gi(&t.offer, i);
                if let Some(v) = empire_item(&mut g.emp[e], i) {
                    *v = v.wrapping_add(d);
                }
            }
            while money_overflow(g.emp[e].money.get(), t.offer.credits) {
                g.emp[e].bank.add(1000000);
                t.offer.credits -= 1000000;
            }
            g.emp[e].money.add(t.offer.credits);
            g.emp[e].carriers += t.carriers;
            g.emp[e].tradegoods -= goods_value(&t.offer);
            false
        }
        _ => false,
    }
}

fn is_class(o: &dyn Streamable, name: &[u8]) -> bool {
    str_eq_i2(name, o.class_name())
}

// 6e3f:1f14: fleets that have reached the empire, one at a time.
pub fn trade_arrivals(g: &mut Sre, _p: &mut Player, e: usize) -> i16 {
    let now = b_time();
    let mut n: i16 = 0;
    let mut last: i32 = 0;
    loop {
        let mut done = false;
        let mut idx: i32 = 0;
        {
            let mut f = StreamFile::new(g, TRADE_FILE);
            loop {
                let o = f.read(g);
                idx += 1;
                let mut o = match o {
                    Some(o) => o,
                    None => break,
                };
                if !is_class(o.as_ref(), TRADE_CLASS) {
                    let gn = g.time.now;
                    put_time(g, ERRLOG, gn);
                    out!(g, ERRLOG; " Error@1: read a ", o.class_name(), " from the trading file.\n");
                    break;
                }
                let mut msg: Option<Box<dyn Streamable>> = None;
                let hasmsg = o.as_any().downcast_mut::<TradingFleet>().unwrap().d.hasmsg;
                if hasmsg != 0 {
                    let m = f.read(g);
                    if let Some(m) = &m {
                        if !is_class(m.as_ref(), MESSAGE_CLASS) {
                            put_time(g, ERRLOG, now);
                            out!(g, ERRLOG; " Error@1: read a ", m.class_name(), " from the trading file (expecting a Message).\n");
                            continue; // (both objects are lost)
                        }
                    }
                    msg = m;
                }
                let t = o.as_any().downcast_mut::<TradingFleet>().unwrap();
                if !done && last < idx && t.d.to == g.emp[e].id && now > t.d.created {
                    n += 1;
                    done = true;
                    last = idx;
                    match msg.as_mut() {
                        Some(m) => {
                            let m = m.as_any().downcast_mut::<Message>().unwrap();
                            m.header(g, n, 6, 0xe);
                            m.show_lines(g, 6, 0xe, 0xc, 0xf);
                        }
                        None => kout!(g; col(6), b"\xc4\xc4[", col(14), n, col(6), b"]\xc4\xc4\n", cn(g)),
                    }
                    if arrive_trade(g, e, &mut t.d) {
                        t.d.hasmsg = 0;
                        f.keep(g, t);
                    }
                } else {
                    if msg.is_none() {
                        t.d.hasmsg = 0;
                    }
                    f.keep(g, t);
                    if t.d.hasmsg != 0 {
                        if let Some(m) = msg.as_mut() {
                            f.keep(g, m.as_mut());
                        }
                    }
                }
            }
            f.close(g);
        }
        if !done {
            return n;
        }
    }
}

// 6e3f:2429
pub fn trade(g: &mut Sre, _p: &mut Player, e: usize) {
    if !g.grb(&R_TRADING) {
        return;
    }
    if g.emp[e].year < 0 {
        return;
    }
    while send_trade(g, e) {}
}

// 6e3f:245d
pub fn clear_trades(g: &mut Sre) {
    let f = StreamFile::new(g, TRADE_FILE);
    f.close(g);
}

// 6e3f:2497
pub fn trades_empire_deleted(g: &mut Sre, id: i16) -> i16 {
    let mut f = StreamFile::new(g, TRADE_FILE);
    loop {
        let mut o = match f.read(g) {
            Some(o) => o,
            None => break,
        };
        if !is_class(o.as_ref(), TRADE_CLASS) {
            let gn = g.time.now;
            put_time(g, ERRLOG, gn);
            out!(g, ERRLOG; " Error@2: read a ", o.class_name(), " from the trading file.\n");
            break;
        }
        let mut msg: Option<Box<dyn Streamable>> = None;
        let hasmsg = o.as_any().downcast_mut::<TradingFleet>().unwrap().d.hasmsg;
        if hasmsg != 0 {
            msg = f.read(g);
            if let Some(m) = &msg {
                if !is_class(m.as_ref(), MESSAGE_CLASS) {
                    let gn = g.time.now;
                    put_time(g, ERRLOG, gn);
                    out!(g, ERRLOG; " Error@2: read a ", m.class_name(), " from the trading file (expecting a Message).\n");
                    continue; // (both objects are lost)
                }
            }
        }
        let t = o.as_any().downcast_mut::<TradingFleet>().unwrap();
        t.on_empire_deleted(g, id);
        f.keep(g, t);
        if t.d.hasmsg != 0 {
            if let Some(m) = msg.as_mut() {
                f.keep(g, m.as_mut());
            }
        }
    }
    f.close(g);
    0
}
