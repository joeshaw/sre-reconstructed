// The production of the planets at the start of a turn (segment 462a).
#include "sre.h"
#include "itemlist.h"

struct Income {
    int32 petroleum;    // 00
    int32 urban;        // 04
    int32 tourism;      // 08
    int32 ore;          // 0c
    int32 food;         // 10
    int32 pollution;    // 14 added by petroleum planets
    int32 cleaned;      // 18 removed by anti-pollution planets
    int32 soldiers;     // 1c
    int32 fighters;     // 20
    int32 stations;     // 24
    int32 heavycruisers;  // 28
    int32 carriers;     // 2c
    int32 generals;     // 30
    int32 credits;      // 34 from supply planets
    int32 taxes;        // 38
};

static inline PlanetSet &P(Empire *e, int16 type) { return e->planets[type]; }

// 462a:0000: produce everything; the total income (in thousands) is kept
// in Empire::income.
static void Produce(Empire *e, Income *r)
{
    memset(r, 0, sizeof *r);
    int32 total = 0;
    if (P(e, P_FOOD).count) {
        r->food = P(e, P_FOOD).count * P(e, P_FOOD).output - e->pollution / 10;
        if (r->food < 0)
            r->food = 0;
        e->food += r->food;
    }
    if (P(e, P_ORE).count) {
        r->ore = P(e, P_ORE).count * P(e, P_ORE).output;
        if (P(e, P_ORE).count < 100) {
            for (int32 i = 0; i < P(e, P_ORE).count; i++)
                r->ore += Random(100);
        }
        if (r->ore < 0)
            r->ore = 0;
        total += r->ore / 1000;
        e->money.add(r->ore);
    }
    if (P(e, P_SUPPLY).count) {
        int32 c = P(e, P_SUPPLY).count * P(e, P_SUPPLY).output;
        int32 soldiers = c * (int32)e->supplyrates[0] / 100;
        int32 fighters = c * (int32)e->supplyrates[1] / 130;
        int32 stations = c * (int32)e->supplyrates[2] / 170;
        int32 generals = c * (int32)e->supplyrates[5] / 700;
        int32 carriers = c * (int32)e->supplyrates[4] / 950;
        int32 cruisers = c * (int32)e->supplyrates[3] / 1200;
        int32 rest = c - (10 * soldiers + 13 * fighters + 17 * stations + 70 * generals +
                          95 * carriers + 120 * cruisers) / 10;
        int32 credits = 100 * rest;
        r->soldiers = soldiers;
        e->soldiers += soldiers;
        r->fighters = fighters;
        e->fighters += fighters;
        r->stations = stations;
        e->stations += stations;
        r->heavycruisers = cruisers;
        e->heavycruisers += cruisers;
        r->generals = generals;
        e->generals += generals;
        r->carriers = carriers;
        e->carriers += carriers;
        r->credits = credits;
        e->money.add(credits);
        total += credits / 1000;
    }
    if (P(e, P_TOURISM).count) {
        int32 up = Random(1000);
        int32 down = Random(1000);
        r->tourism = P(e, P_TOURISM).count *
                     (P(e, P_TOURISM).output + up - down - 350 * (int32)e->insurgency - e->pollution / 10);
        if (r->tourism < 0)
            r->tourism = 0;
        e->money.add(r->tourism);
        total += r->tourism / 1000;
    }
    if (P(e, P_URBAN).count) {
        r->urban = P(e, P_URBAN).count *
                   (P(e, P_URBAN).output - (int32)(int16)(e->insurgency * e->insurgency));
        if (r->urban < 0)
            r->urban = 0;
        e->money.add(r->urban);
        total += r->urban / 1000;
    }
    if (P(e, P_PETROLEUM).count) {
        r->petroleum = P(e, P_PETROLEUM).count * OilPrice();
        if (r->petroleum < 0)
            r->petroleum = 0;
        e->money.add(r->petroleum);
        total += r->petroleum / 1000;
        if (OilPrice() > 10000)
            total += r->petroleum / 3000;
        r->pollution = P(e, P_PETROLEUM).count;
        if (r->pollution < 0)
            r->pollution = 0;
        e->pollution += r->pollution;
    }
    if (P(e, P_ANTIPOLLUTION).count) {
        r->cleaned = P(e, P_ANTIPOLLUTION).output * P(e, P_ANTIPOLLUTION).count;
        if (r->cleaned < 0)
            r->cleaned = 0;
        if (e->pollution - r->cleaned < 0)
            r->cleaned = e->pollution;
        if (r->cleaned < 0)
            r->cleaned = 0;
        e->pollution -= r->cleaned;
        if (e->pollution < 0)
            e->pollution = 0;
    }
    if (P(e, P_RESEARCH).count) {
        int32 x = P(e, P_RESEARCH).count * P(e, P_RESEARCH).output;
        if (x < 0)
            x = 0;
        if (x + e->researchpoints > 100000)
            e->researchpoints = 100000;
        else
            e->researchpoints += x;
    }
    if (e->population > 0) {
        r->taxes = 3 * (e->population * (int32)e->taxrate) / ((int32)e->insurgency + 1);
        for (int16 i = 0; i < MAXEMPIRES; i++) {
            if (e->treaty[i].type() != TT_FREETRADE)
                continue;
            Empire *o = &g_empires[i];
            if (Exists(*o))
                r->taxes += o->population * Random(16);
        }
        if (r->taxes < 0)
            r->taxes = 0;
        e->money.add(r->taxes);
        total += r->taxes / 1000;
    }
    e->income.set(total);
}

// 462a:0b58: the report of the production.
void TurnReport(Player *p, Empire *e)
{
    Income r;
    Produce(e, &r);
    if (P(e, P_FOOD).count > 0 || r.food > 0) {
        if (r.food > 0)
            kout << c_number << r.food << c_normal << " megatons of food grown.\n";
        else
            kout << c_number << "NO" << c_normal
                 << " megatons of food were grown, due to excessive pollution.\n";
    }
    if (P(e, P_RESEARCH).count > 0) {
        int32 k = e->lightcruisers / 70 + 10;
        int32 n = 10 * P(e, P_RESEARCH).count / k;
        if (n > 0) {
            kout << c_number << n << c_normal
                 << " light cruisers were produced by your research planets.\n";
            e->lightcruisers += n;
        }
    }
    if (P(e, P_SUPPLY).count) {
        ItemList l(c_normal, c_number);
        l.SkipZero();
        l.Add(r.soldiers, "soldier");
        l.Add(r.fighters, "fighter");
        l.Add(r.stations, "defense station");
        l.Add(r.heavycruisers, "heavy cruiser");
        l.Add(r.generals, "general");
        l.Add(r.carriers, "carrier");
        l.Add(r.credits, "credit");
        kout << "Your supply planets produce " << l << ".\n";
    }
    if (P(e, P_ORE).count > 0 || r.ore > 0)
        kout << c_number << r.ore << c_normal << " credits worth of ore has been mined.\n";
    if (P(e, P_TOURISM).count > 0 || r.tourism > 0) {
        if (r.tourism > 0) {
            kout << c_number << r.tourism << c_normal
                 << " credits have been earned from tourist attractions.\n";
        } else {
            kout << c_number << "NO" << c_normal
                 << " credits have been earned from tourist attractions,\n";
            kout << "   possibly due to the poor conditions in your empire.\n";
        }
    }
    if (P(e, P_URBAN).count > 0 || r.urban > 0)
        kout << c_number << r.urban << c_normal
             << " credits were collected as trade taxes on urban planets.\n";
    if (P(e, P_PETROLEUM).count > 0 || r.petroleum > 0)
        kout << c_number << r.petroleum << c_normal
             << " credits have been earned from petroleum pumping.\n";
    int32 net = r.pollution - r.cleaned;
    const char *how = "released into";
    if (net < 0) {
        net = -net;
        how = "removed from";
    }
    if (net > 0)
        kout << c_number << net << c_normal << " exatons of pollutants have been " << how
             << " the galactic atmosphere.\n";
    if (r.taxes > 0)
        kout << "\nYou collect " << c_number << r.taxes << c_normal << " credits in taxes.\n";
}
