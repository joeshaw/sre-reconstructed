// Paying for the empire's upkeep at the start of a turn (segment 5e48).
#include "sre.h"


struct Costs {
    int32 military;     // 00
    int32 effect;       // 04 raising the army's effectiveness
    int32 violence;     // 08 reducing insurgency
    int32 loan;         // 0c loan payment
    int32 planets;      // 10 planet maintenance
    int32 donation;     // 14 to the Galactic Coordinator
};

// The planets neglected this turn (e+0x1c7, ten longs).
int32 Neglected(Empire *e, int16 i)
{
    int32 v;
    memcpy(&v, e->u_1c7 + 4 * i, 4);
    return v;
}

void SetNeglected(Empire *e, int16 i, int32 v)
{
    memcpy(e->u_1c7 + 4 * i, &v, 4);
}

// 5e48:0000
static void ComputeCosts(Empire *e, Costs *c)
{
    c->military = 2 * e->soldiers + 4 * e->fighters + 12 * e->heavycruisers +
                  10 * e->lightcruisers + 5 * e->stations;
    int32 x = (30 * e->fighters + 15 * e->soldiers + 75 * e->generals + 50 * e->heavycruisers +
               30 * e->lightcruisers) / 10 + 17;
    int16 k = (int16)(100 - e->effectiveness);
    if (k > 20)
        k = 20;
    c->effect = (int32)k * (x / 100);
    c->violence = (int32)((int16)(e->insurgency + 1) / 2) * (TotalPlanets(e) + 2 * e->population);
    c->loan = 0;
    if (e->bank.get() < 0) {
        c->loan = e->d1a4;
        if (c->loan + e->bank.get() > 0)
            c->loan = -e->bank.get();
    }
    if (g_galaxy.w384 == 0) {
        // Upkeep per planet grows with the logarithm of the size.
        static const int16 FRACTION[8] = { 0, 86, 266, 422, 533, 619, 689, 748 };
        int32 per = -1400;
        int32 v = TotalPlanets(e) + 500;
        while (v >= 8) {
            per += 800;
            v /= 8;
        }
        per += FRACTION[v];
        c->planets = per * TotalPlanets(e);
    } else {
        c->planets = TotalPlanets(e) * ((int32)e->year + 1000);
    }
    if (c->planets < 0)
        c->planets = 0;
    c->donation = e->m1bc.get() * (int32)g_galaxy.w3b0;
    if (g_galaxy.d3ac > 1500000000)
        c->donation = 0;
}

// 5e48:03f6: the consequences of what was paid.
static void Consequences(Empire *e, Costs *c, Costs *paid)
{
    if (paid->military < c->military) {
        e->b1c5 = (int8)((c->military - paid->military) / (c->military / 100 + 1));
        e->b1c6 = (int8)(e->b1c5 + e->effectiveness);
        e->b1c4 |= 8;
    }
    int32 per = c->effect;
    if (e->effectiveness < 100) {
        int16 k = (int16)(100 - e->effectiveness);
        if (k > 20)
            k = 20;
        per = c->effect / k;
    }
    if (c->effect + c->effect / 2 < paid->effect)
        paid->effect = c->effect + c->effect / 2;
    while (paid->effect >= per && per > 0) {
        int8 v = (int8)(e->effectiveness + 1);
        e->effectiveness = v;
        if (e->effectiveness < 0)
            e->effectiveness = 0;
        if (e->effectiveness > 100)
            e->effectiveness = 100;
        paid->effect -= per;
    }
    if (paid->violence >= c->violence - c->violence / 10 && e->insurgency > 0)
        e->insurgency--;
    e->bank.add(paid->loan);
    if (e->bank.get() >= 0)
        e->d1a4 = 0;
    if (paid->planets < c->planets) {
        int32 n = ((c->planets - paid->planets) / (c->planets / 1000 + 1)) *
                  (TotalPlanets(e) - 3 * e->planets[P_GOVERNMENT].count) / 2000;
        if (n > 0) {
            int32 k = TotalPlanets(e) / n;
            int16 every = (int16)(k > 10000 ? 10000 : k < 1 ? 1 : k);
            e->b1c4 |= 1;
            int32 total = 0;
            for (int16 i = 0; i < 10; i++) {
                SetNeglected(e, i, Planets(e, i)->count / every);
                total += Neglected(e, i);
            }
            for (int16 j = 0; j < 100 && total < n; j++) {
                int16 t = (int16)(((int32)10 * b_rand()) / 0x8000);
                if (Planets(e, t)->count > Neglected(e, t)) {
                    SetNeglected(e, t, Neglected(e, t) + 1);
                    total++;
                }
            }
        }
    }
    if (paid->donation > c->donation) {
        int32 x = (paid->donation - c->donation) / (e->m1bc.get() + 1);
        if (x > 1000)
            x -= 1000;
        if (x > 0)
            e->d1b0 += (int16)x;
    }
    g_galaxy.d3ac += paid->donation / 1000;
    if (TotalPlanets(e) > 0 && e->m1bc.get() < 2000000)
        e->d1b4 = (1000 * e->m1bc.get() - c->military - c->effect - c->violence - c->loan -
                   c->planets - c->donation) /
                  TotalPlanets(e);
}

// 5e48:09e3: ask how much to pay of an amount.
static int32 AskPay(const char *what, int32 amount, Money *avail, int32 min)
{
    if (amount <= 0)
        return 0;
    int32 def = amount;
    if (avail->get() < def)
        def = avail->get();
    kout << c_normal;
    if (avail->get() < min)
        min = avail->get();
    kout << '\n' << what << ' ' << c_number << amount << c_normal << " credits.\n";
    int32 v;
    do {
        kout << "Pay how much?  ";
        kin >> maxval(avail->get()) >> defval(def) >> v;
        kout << c_normal;
        if (v < min)
            kout << "You must pay at least " << c_number << min << c_normal << " credits.\n";
    } while (v < min);
    avail->sub(v);
    return v;
}

// 5e48:0b9f
void PayUpkeep(Player *p, Empire *e, Galaxy *g)
{
    Costs c, paid;
    ComputeCosts(e, &c);
    for (;;) {
        int32 total = c.planets + c.loan + c.military + c.violence + c.donation;
        kout << "Your empire requires " << c_number << total << c_normal
             << " credits this year.  You have " << c_number << e->money.get() << c_normal
             << " credits.\n";
        if (e->money.get() >= total) {
            int16 autopay = (p->prefs2 >> 1) & 1;
            if (p->prefs2 & 1)
                autopay = YesNo(autopay, "Pay everything automatically");
            if (autopay) {
                paid = c;
                e->money.sub(total);
                break;
            }
        }
        if (g_system.regstate && YesNo(0, "Do you wish to visit the bank"))
            Bank(p, e, g);
        Money avail = e->money;
        int16 shortfall = 0;
        paid.planets = AskPay("Maintaining your planets requires", c.planets, &avail, 0);
        shortfall |= paid.planets < c.planets;
        paid.loan = AskPay("For your loan, the bank requires that you pay back", c.loan, &avail, c.loan);
        paid.military = AskPay("Your military requires", c.military, &avail, 0);
        shortfall |= paid.military < c.military;
        paid.violence = AskPay("Violence within your empire can be reduced by spending", c.violence,
                               &avail, 0);
        shortfall |= paid.violence < c.violence;
        paid.effect = AskPay("Your Army's effectiveness can be raised by spending", c.effect,
                             &avail, 0);
        shortfall |= paid.effect < c.effect;
        paid.donation = AskPay("The Galactic Coordinator asks that you donate", c.donation, &avail,
                               c.donation);
        if (shortfall) {
            kout << "\nNot paying for maintenance of your empire may lead to DISASTEROUS consequences!  ";
            if (YesNo(1, "Do you wish to reconsider"))
                continue;
        }
        e->money.set(avail.get());
        break;
    }
    Consequences(e, &c, &paid);
}
