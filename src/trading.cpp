// Trading fleets between empires (segment 6e3f).
#include "sre.h"
#include "streamfile.h"
#include "itemlist.h"
#include "messages.h"
#include "timeutil.h"

static const char *TRADE_FILE = "Data/Trading";     // DS:aaa4
static const char *TRADE_CLASS = "TradingFleet";    // DS:aab1
Resource r_trading("Trading", (int32)1);             // aa0a
static Resource r_tradeexpire("Trade.Expire", (int32)4);  // aa16

#pragma pack(push, 1)
// What a fleet carries (6e3f:0152).
struct Goods {
    int32 soldiers;     // 00
    int32 fighters;     // 04
    int32 cruisers;     // 08
    int32 generals;     // 0c
    int32 carriers;     // 10
    int32 credits;      // 14
    int32 food;         // 18
    int32 agents;       // 1c
};
struct TradeData {
    int32 created;      // 00
    int32 expires;      // 04
    int16 to;           // 08
    int16 from;         // 0a
    int16 state;        // 0c 0 done, 1 sent, 2 accepted, 3 rejected, 4 returning
    Goods offer;        // 0e
    Goods demand;       // 2e
    int32 carriers;     // 4e carriers carrying the fleet
    int16 hasmsg;       // 52 a Message follows in the file
};
#pragma pack(pop)
static_assert(sizeof(TradeData) == 0x54, "TradeData");

// The items of the trading menus, by number (DS:aa5e, aa36, aa4a).
static const char *const ITEM[10] = { 0,      "Soldiers", "Fighters", 0,        "Generals",
                                      "Money", "Food",     "Agents",   "Cruisers", "Carriers" };

static int32 *EmpireItem(Empire *e, int16 i)
{
    switch (i) {
    case 1: return &e->soldiers;
    case 2: return &e->fighters;
    case 4: return &e->generals;
    case 6: return &e->food;
    case 7: return &e->agents;
    case 8: return &e->heavycruisers;
    case 9: return &e->carriers;
    }
    return 0;
}

static int32 *GoodsItem(Goods *g, int16 i)
{
    switch (i) {
    case 1: return &g->soldiers;
    case 2: return &g->fighters;
    case 4: return &g->generals;
    case 5: return &g->credits;
    case 6: return &g->food;
    case 7: return &g->agents;
    case 8: return &g->cruisers;
    case 9: return &g->carriers;
    }
    return 0;
}

// 6e3f:01c5
static int32 GoodsValue(Goods *g)
{
    return g->credits / 100000 + 7 * g->agents + g->soldiers + g->fighters + 3 * g->cruisers +
           3 * g->carriers + 2 * g->generals;
}

// 6e3f:0271
static void GoodsList(Goods *g, ItemList *l)
{
    l->SkipZero();
    l->Add(g->food, "megatons of food");
    l->Add(g->soldiers, "soldier");
    l->Add(g->fighters, "fighter");
    l->Add(g->cruisers, "heavy cruiser");
    l->Add(g->generals, "general");
    l->Add(g->carriers, "carrier");
    l->Add(g->agents, "covert agent");
    l->Add(g->credits, "credit");
}

// 6e3f:03da: carriers needed besides those traded.
static int32 CarriersNeeded(Goods *g)
{
    int32 n = (g->fighters + g->generals + 3 * g->agents + 149) / 150 + (g->soldiers + 499) / 500 +
              (g->food + 9999) / 10000;
    n -= g->carriers;
    if (n < 0)
        n = 0;
    return n;
}

static void TradeInit(TradeData *t)
{
    memset(&t->offer, 0, sizeof t->offer);
    memset(&t->demand, 0, sizeof t->demand);
    t->carriers = 0;
    t->to = -1;
    t->from = -1;
    t->state = 0;
    t->created = g_now;
    t->expires = t->created + 86400 * r_tradeexpire.Long();
}

// 6e3f:05fe
static int32 FleetCarriers(TradeData *t)
{
    int32 n = CarriersNeeded(&t->offer);
    if (CarriersNeeded(&t->demand) > n)
        n = CarriersNeeded(&t->demand);
    return n;
}

// Class "TradingFleet" (vtable b009).
class TradingFleet : public Streamable {
public:
    TradeData d;
    TradingFleet() { TradeInit(&d); d.hasmsg = 0; }         // 6e3f:0000
    const char *ClassName() { return TRADE_CLASS; }         // 6e3f:0058
    int16 Size() { return 0x56; }                           // 6e3f:0066
    uint8 *Data() { return (uint8 *)&d; }
    int16 Expired() { return d.state == 0; }                // 6e3f:0070
    void OnEmpireDeleted(int16 id);                         // 6e3f:0088
};

static Streamable *NewTradingFleet() { return new TradingFleet(); }
static struct TradeReg {
    ClassEntry e;
    TradeReg() { RegisterClass(&e, TRADE_CLASS, NewTradingFleet); }    // 6e3f:27a1
} s_tradereg;

// 6e3f:0088: a fleet to a deleted empire turns back; one from it is lost.
void TradingFleet::OnEmpireDeleted(int16 id)
{
    if (d.to == id && d.state == 1) {
        d.to = d.from;
        d.from = id;
        d.state = 4;
        if (d.to != -1 && !Exists(g_empires[d.to]))
            d.state = 0;
    }
    if (d.from == id) {
        if (d.state == 1)
            d.state = 0;
        else if (d.state != 0 && d.to != -1 && Exists(g_empires[d.to]))
            d.state = 4;
    }
}

// 6e3f:0658
static void AskAmount(const char *name, int32 *v, int32 max, int16 demand)
{
    kout << "New amount of " << c_number << name << c_normal;
    if (!demand) {
        kout << " to send?  " << c_bracket << '[' << c_bold << "MAX=" << max << c_bracket << ']'
             << c_normal << "  ";
        kin >> maxval(max);
    } else {
        kout << " to demand?  ";
    }
    kin >> defval(*v);
    if (!demand)
        kin >> maxval(max);
    kin >> *v;
}

// 6e3f:0791
struct FleetPrompt : MenuHook {
    Empire *e;
    TradeData *t;
    FleetPrompt(Empire *x, TradeData *y) : e(x), t(y) {}
    int16 Call()
    {
        int32 n = FleetCarriers(t);
        kout << "Current fleet requires " << c_number << n << c_normal << " carriers.\n";
        int32 more = n + t->offer.carriers - e->carriers;
        if (more > 0)
            kout << "This is " << Color(14) << more << c_normal << " more carriers than you have.\n";
        return 1;
    }
};

// 6e3f:087f: set up the offer and the demands.
static int16 MakeOffer(Empire *e, TradeData *t)
{
    int16 r;
    do {
        Menu m("Trading.Offer", "Your Offer", 0xe, 6, 4, 0xe, 7, 0xa, 0xf, 7, 7, 7);
        m.prompt = new FleetPrompt(e, t);
        m.Add("Item", 0, 100);
        MenuItem *h = m.Find(100);
        for (int16 i = 0; i < 8; i++)
            h->colors[i] = 7;
        h->LoadColors();
        m.Find(100)->AddColumn("Sending");
        m.Find(100)->AddColumn("Owned");
        for (int16 i = 1; i <= 9; i++) {
            if (!GoodsItem(&t->offer, i))
                continue;
            m.Add(ITEM[i], (char)('0' + i), i);
            int32 sending = *GoodsItem(&t->offer, i);
            int32 owned = i == 5 ? e->money.get() : *EmpireItem(e, i);
            owned -= sending;
            m.Find(i)->AddColumn(sending);
            m.Find(i)->AddColumn(owned);
        }
        r = m.Run(1, -1);
        if (r >= 1 && r <= 9 && r != 3) {
            if (r == 5)
                AskAmount(ITEM[5], &t->offer.credits, e->money.get(), 0);
            else
                AskAmount(ITEM[r], GoodsItem(&t->offer, r), *EmpireItem(e, r), 0);
        }
    } while (r != -1);
    if (FleetCarriers(t) + t->offer.carriers > e->carriers) {
        kout << "You don't have enough carriers to send the fleet.\n";
        return 0;
    }
    do {
        Menu m("Trading.Demand", "Your Demands", 0xe, 6, 4, 0xe, 7, 0xc, 7, 7, 7, 7);
        m.prompt = new FleetPrompt(e, t);
        m.Add("Item", 0, 100);
        MenuItem *h = m.Find(100);
        for (int16 i = 0; i < 8; i++)
            h->colors[i] = 7;
        h->LoadColors();
        m.Find(100)->AddColumn("Demand");
        for (int16 i = 1; i <= 9; i++) {
            if (!GoodsItem(&t->demand, i))
                continue;
            m.Add(ITEM[i], (char)('0' + i), i);
            m.Find(i)->AddColumn(*GoodsItem(&t->demand, i));
        }
        r = m.Run(1, -1);
        if (r >= 1 && r <= 9 && r != 3) {
            if (r == 5)
                AskAmount(ITEM[5], &t->demand.credits, 0, 1);
            else
                AskAmount(ITEM[r], GoodsItem(&t->demand, r), 0, 1);
        }
    } while (r != -1);
    if (FleetCarriers(t) + t->offer.carriers > e->carriers) {
        kout << "You don't have enough carriers to send the fleet.\n";
        return 0;
    }
    t->carriers = FleetCarriers(t);
    return 1;
}

// 6e3f:0fd8: the empires one has relations with.
static int16 ChoosePartner(Empire *e)
{
    int32 allowed = EmpireMask() & ~ProtectedMask();
    for (int16 i = 0; i < 25; i++) {
        // (The original tests the trading empire itself, not empire i.)
        if (e->treaty[i].type() == 0 || !Exists(*e) || TotalPlanets(e) <= 0 ||
            e->population <= 0)
            MaskClear(&allowed, i);
    }
    return ChooseEmpire(allowed);
}

// 6e3f:108f
static int16 SendTrade(Empire *e)
{
    TradeData t;
    TradeInit(&t);
    int16 to = ChoosePartner(e);
    if (to == -1)
        return 0;
    if (!MakeOffer(e, &t))
        return 1;
    if (!YesNo(1, "Send this trade deal"))
        return 1;
    for (int16 i = 1; i <= 9; i++)
        if (EmpireItem(e, i))
            *EmpireItem(e, i) -= *GoodsItem(&t.offer, i);
    e->carriers -= t.carriers;
    e->money.sub(t.offer.credits);
    e->tradegoods += GoodsValue(&t.offer);
    t.to = to;
    t.from = e->id;
    t.state = 1;
    TradingFleet fleet;
    fleet.d = t;
    fleet.d.hasmsg = 0;
    Streamable *msg = 0;
    if (r_messages.Bool() && YesNo(0, "Attach a message"))
        msg = ComposeMessage(e, to);
    if (msg)
        fleet.d.hasmsg = 1;
    AppendObject(TRADE_FILE, &fleet);
    if (msg)
        AppendObject(TRADE_FILE, msg);
    kout << "Trading fleet sent.\n";
    return 1;
}

// 6e3f:1290: can the empire give what is demanded?
static int16 CanMeet(TradeData *t, Empire *e)
{
    for (int16 i = 1; i <= 9; i++) {
        if (!EmpireItem(e, i))
            continue;
        if (*GoodsItem(&t->demand, i) - *GoodsItem(&t->offer, i) > *EmpireItem(e, i))
            return 0;
    }
    Money m = e->money;
    m.add(e->bank.get());
    return t->demand.credits - t->offer.credits <= m.get();
}

static void Key(char c)
{
    kout << c_bracket << '[' << c_bold << c << c_bracket << ']' << c_normal;
}

// 6e3f:1394: a fleet reaches the empire; returns 1 to keep it.
static int16 ArriveTrade(Empire *e, TradeData *t)
{
    if (!r_trading.Bool())
        return 0;
    switch (t->state) {
    case 0:
        return 0;
    case 1: {
        Empire *o = &g_empires[t->from];
        kout << c_empire << Untr(o->name) << c_normal << " has sent a trading deal:\n";
        kout << "In return for ";
        {
            ItemList l(c_normal, c_number);
            GoodsList(&t->offer, &l);
            kout << l << ", you must send back ";
        }
        {
            ItemList l(c_normal, c_number);
            GoodsList(&t->demand, &l);
            kout << l << ".\n";
        }
        t->state = 3;
        char c;
        if (e->year < 0) {
            kout << "However, since you are in protection, you cannot trade with other empires.\n";
            c = 'N';
        } else if (!CanMeet(t, e)) {
            kout << "However, you can not meet the demands right now.\n";
            kout << "Do you wish to ";
            Key('K');
            kout << "eep for later or ";
            Key('R');
            kout << "eject?  " << c_input;
            c = GetChoice("KR", 1);
            if (c == 'K') {
                kout << "Keep for later";
                c = 'I';
            } else if (c == 'R') {
                kout << "Reject";
                c = 'N';
            }
            kout << c_normal << '\n';
        } else if (e->treaty[t->from].type() == 0) {
            kout << "However, you no longer have any relations with that empire.\n";
            c = 'N';
            t->state = 4;
        } else {
            kout << "Accept this deal?  ";
            Key('Y');
            kout << "es, ";
            Key('N');
            kout << "o, or ";
            Key('I');
            kout << "gnore for now:  " << c_input;
            c = GetChoice("YNI", 1);
            if (c == 'Y')
                kout << "Accept\n";
            else if (c == 'N')
                kout << "Reject Offer\n";
            else if (c == 'I')
                kout << "Ignore for now\n";
        }
        if (c == 'N') {
            t->to = t->from;
            t->from = e->id;
            return 1;
        }
        if (c == 'I') {
            t->state = 1;
            t->created += 2;
            return 1;
        }
        for (int16 i = 1; i <= 9; i++)
            if (EmpireItem(e, i))
                *EmpireItem(e, i) -= *GoodsItem(&t->demand, i) - *GoodsItem(&t->offer, i);
        o->tradegoods -= GoodsValue(&t->offer);
        e->tradegoods += GoodsValue(&t->demand);
        e->money.sub(t->demand.credits - t->offer.credits);
        if (e->money.get() - t->demand.credits + t->offer.credits < 0) {
            e->bank.add(e->money.get());
            e->money.set(0);
        }
        t->state = 2;
        t->to = t->from;
        t->from = e->id;
        return 1;
    }
    case 2: {
        // The fleet returns with what was demanded.
        Empire *o = &g_empires[t->from];
        if (Exists(*o))
            kout << c_number << Untr(o->name) << c_normal << " accepted your trade agreement.\n";
        else
            kout << "An unknown empire accepted your trade agreement.\n";
        for (int16 i = 1; i <= 9; i++)
            if (EmpireItem(e, i))
                *EmpireItem(e, i) += *GoodsItem(&t->demand, i);
        while (MoneyOverflow(e->money.get(), t->demand.credits)) {
            e->bank.add(1000000);
            t->demand.credits -= 1000000;
        }
        e->money.add(t->demand.credits);
        e->carriers += t->carriers;
        o->tradegoods -= GoodsValue(&t->demand);
        return 0;
    }
    case 3:
    case 4: {
        // The offer comes back.
        Empire *o = &g_empires[t->from];
        if (Exists(*o) && t->state == 3)
            kout << c_number << Untr(o->name) << c_normal << " rejected your trade agreement.\n";
        else
            kout << "A trading fleet was not able to continue, and has returned.\n";
        for (int16 i = 1; i <= 9; i++)
            if (EmpireItem(e, i))
                *EmpireItem(e, i) += *GoodsItem(&t->offer, i);
        while (MoneyOverflow(e->money.get(), t->offer.credits)) {
            e->bank.add(1000000);
            t->offer.credits -= 1000000;
        }
        e->money.add(t->offer.credits);
        e->carriers += t->carriers;
        e->tradegoods -= GoodsValue(&t->offer);
        return 0;
    }
    }
    return 0;
}

static int16 IsClass(Streamable *o, const char *name)
{
    return StrEqI(String(name), String(o->ClassName()));
}

// 6e3f:1f14: fleets that have reached the empire, one at a time.
int16 TradeArrivals(Player *p, Empire *e)
{
    (void)p;
    int32 now = b_time(0);
    int16 n = 0;
    int32 last = 0;
    for (;;) {
        int16 done = 0;
        int32 idx = 0;
        {
            StreamFile f(TRADE_FILE);
            for (;;) {
                Streamable *o = f.Read();
                idx++;
                if (!o)
                    break;
                if (!IsClass(o, TRADE_CLASS)) {
                    PutTime(g_errlog, g_now) << " Error@1: read a " << o->ClassName()
                                             << " from the trading file.\n";
                    break;
                }
                TradingFleet *t = (TradingFleet *)o;
                Message *msg = 0;
                if (t->d.hasmsg) {
                    Streamable *m = f.Read();
                    if (m && !IsClass(m, MESSAGE_CLASS)) {
                        PutTime(g_errlog, now) << " Error@1: read a " << m->ClassName()
                                               << " from the trading file (expecting a Message).\n";
                        continue;   // (both objects are lost)
                    }
                    msg = (Message *)m;
                }
                if (!done && last < idx && t->d.to == e->id && now > t->d.created) {
                    n++;
                    done = 1;
                    last = idx;
                    if (msg) {
                        msg->Header(n, 6, 0xe);
                        msg->ShowLines(6, 0xe, 0xc, 0xf);
                    } else {
                        kout << Color(6) << "\xc4\xc4[" << Color(14) << n << Color(6)
                             << "]\xc4\xc4\n" << c_normal;
                    }
                    if (ArriveTrade(e, &t->d)) {
                        t->d.hasmsg = 0;
                        f.Keep(t);
                    }
                } else {
                    if (!msg)
                        t->d.hasmsg = 0;
                    f.Keep(t);
                    if (t->d.hasmsg)
                        f.Keep(msg);
                }
                delete msg;
                delete t;
            }
        }
        if (!done)
            return n;
    }
}

// 6e3f:2429
void Trade(Player *p, Empire *e)
{
    (void)p;
    if (!r_trading.Bool())
        return;
    if (e->year < 0)
        return;
    while (SendTrade(e))
        ;
}

// 6e3f:245d
void ClearTrades()
{
    StreamFile f(TRADE_FILE);
}

// 6e3f:2497
int16 TradesEmpireDeleted(int16 id)
{
    StreamFile f(TRADE_FILE);
    for (;;) {
        Streamable *o = f.Read();
        if (!o)
            break;
        if (!IsClass(o, TRADE_CLASS)) {
            PutTime(g_errlog, g_now) << " Error@2: read a " << o->ClassName()
                                     << " from the trading file.\n";
            break;
        }
        TradingFleet *t = (TradingFleet *)o;
        Streamable *msg = 0;
        if (t->d.hasmsg) {
            msg = f.Read();
            if (msg && !IsClass(msg, MESSAGE_CLASS)) {
                PutTime(g_errlog, g_now) << " Error@2: read a " << msg->ClassName()
                                         << " from the trading file (expecting a Message).\n";
                continue;   // (both objects are lost)
            }
        }
        t->OnEmpireDeleted(id);
        f.Keep(t);
        if (t->d.hasmsg)
            f.Keep(msg);
        delete msg;
        delete t;
    }
    return 0;
}
