// Raids on the pirate teams (segment 5fe0).
#include "sre.h"
#include "itemlist.h"

// 5fe0:0000
int16 ChoosePirates()
{
    int16 n = PirateCount();
    if (n == 0) {
        kout << "No pirate teams have been created.\n";
        return -1;
    }
    if (n > 9)
        n = 9;
    Menu m("Pirates", "Pirate Teams", 0xf, 6, 6, 0xf, 0xe, 7, 7, 7, 7, 7);
    for (int16 i = 1; i <= n; i++) {
        Pirate *pt = &g_pirates[i - 1];
        m.Add(PirateName(pt), (char)('0' + i), i);
        MenuItem *it = m.Find(i);
        uint8 c = PirateColor(pt);
        it->colors[0] = 6;
        it->colors[1] = 0xf;
        it->colors[2] = c;
        for (int16 k = 3; k < 8; k++)
            it->colors[k] = 7;
        it->LoadColors();
    }
    return m.Run(1, -1);
}

// 5fe0:018a
static int32 AskFleet(String name, int32 def, int32 max)
{
    kout << name << ":  ";
    if (def != max)
        kout << c_bracket << '[' << c_bold << "MAX=" << Color(3) << max << c_bracket << ']'
             << c_normal << "  ";
    int32 v;
    kin >> defval(def) >> maxval(max) >> v;
    return v;
}

static void Shrink(int32 *v, int32 by)
{
    *v += by;
    if (*v < 0)
        *v = 0;
}

// 5fe0:0297
int16 PirateRaid(Player *p, Empire *e)
{
    (void)p;
    int16 r = 0;
    int16 team = ChoosePirates();
    if (team == -1)
        return 0;
    Pirate *pt = &g_pirates[team - 1];
    kout << Color(3) << "Specify your attack fleet:\n" << c_normal;
    int32 credits = AskFleet("Credits", e->money.get() / 10, e->money.get());
    int16 chance;
    if (500 * NetWorth(e) < credits)
        chance = 1000;
    else
        chance = (int16)(2 * credits / NetWorth(e));
    if (chance < 200)
        chance = 200;
    int32 s = AskFleet("Soldiers", e->soldiers, e->soldiers);
    int32 sents = s;
    int32 f = AskFleet("Fighters", e->fighters, e->fighters);
    int32 sentf = f;
    int32 c = AskFleet("Cruisers", e->heavycruisers, e->heavycruisers);
    int32 sentc = c;
    if (s == 0 && f == 0 && c == 0)
        return 0;
    e->money.sub(credits);
    int32 ps = s + pt->soldiers;
    int32 pf = f + pt->fighters;
    int32 pc = c + pt->heavycruisers;
    kout << Color(13);
    for (;;) {
        if (Random(1100) < chance && pt->soldiers > 0)
            Shrink(&pt->soldiers, -ps / 80 - 1);
        if (Random(1300) < chance && pt->heavycruisers > 0)
            Shrink(&pt->heavycruisers, -pc / 80 - 1);
        if (Random(1200) < chance && pt->fighters > 0)
            Shrink(&pt->fighters, -pf / 80 - 1);
        if (Random(1200) < chance && pt->stations > 0)
            Shrink(&pt->stations, -pt->stations / 80 - 1);
        if (Random(500) < 80 && s > 0)
            s -= ps / 80 + 1;
        if (Random(500) < 80 && c > 0)
            c -= pc / 80 + 1;
        if (Random(500) < 80 && f > 0)
            f -= pf / 80 + 1;
        kout << (r % 2 ? '/' : '\\');
        r++;
        if (r > 25)
            break;
        if (!(sents / 2 < s || sentc / 2 < c || sentf / 2 < f))
            break;
        if (!(pt->soldiers > 0 || pt->heavycruisers > 0 || pt->fighters > 0))
            break;
        if ((ps + pc + pf) / 5 > s + c + f)
            break;
    }
    kout << c_normal << '\n';
    if (s < 0)
        s = 0;
    if (f < 0)
        f = 0;
    if (c < 0)
        c = 0;
    e->soldiers -= sents - s;
    e->heavycruisers -= sentc - c;
    e->fighters -= sentf - f;
    int16 score = 0;
    if (pt->fighters > 0)
        score--;
    if (pt->soldiers > 0)
        score--;
    if (pt->heavycruisers > 0)
        score--;
    if (s > 0)
        score++;
    if (c > 0)
        score++;
    if (f > 0)
        score++;
    if (pt->fighters > f)
        score--;
    if (pt->soldiers > s)
        score--;
    if (pt->heavycruisers > c)
        score--;
    if (pt->fighters < f)
        score++;
    if (pt->soldiers < s)
        score++;
    if (pt->heavycruisers < c)
        score++;
    if (2 * f < pt->fighters)
        score--;
    if (2 * s < pt->soldiers)
        score--;
    if (2 * c < pt->heavycruisers)
        score--;
    if (2 * pt->fighters < f)
        score++;
    if (2 * pt->soldiers < s)
        score++;
    if (2 * pt->heavycruisers < c)
        score++;
    if (2 * pt->fighters + pt->soldiers + 13 * pt->heavycruisers > 2 * f + s + 13 * c)
        score -= 5;
    else
        score += 5;
    ItemList lost(c_normal, c_number);
    lost.SkipZero();
    lost.Add(sents - s, "soldier");
    lost.Add(sentf - f, "fighter");
    lost.Add(sentc - c, "heavy cruiser");
    if (score > 0) {
        int32 total = 0;
        int16 div = 3;
        if (Random(13) == 0 && ShowScreen(0, "RaidWar", 0))
            Pause();
        if (e->year < 0)
            div += 3;
        if (e->score > 100000)
            div += 2;
        if (TotalPlanets(e) > 500)
            div += 2;
        if (NetWorth(e) > 100000)
            div += 4;
        if (score > 3)
            div--;
        if (score > 6)
            div--;
        if (score > 9)
            div--;
        if (div < 2)
            div = 2;
        for (int16 k = 0; k < 10; k++) {
            int16 take = (int16)(pt->planets[k] / div);
            if (take > 100)
                take = 100;
            if (take < 0)
                take = 0;
            Planets(e, k)->count += take;
            pt->planets[k] = (int16)(pt->planets[k] - take);
            if (pt->planets[k] < 0)
                pt->planets[k] = 0;
            total += take;
        }
        ItemList got(c_normal, Color(14));
        got.SkipZero();
        got.Add(total, "planet");
        got.Add(pt->food, "megatons of food", "");
        got.Add(pt->soldiers / div, "soldier");
        got.Add(pt->fighters / div, "fighter");
        got.Add(pt->stations / div, "defense station");
        got.Add(pt->heavycruisers / div, "heavy cruiser");
        got.Add(pt->carriers / div, "carrier");
        got.Add(pt->generals / div, "general");
        got.Add(pt->agents / div, "covert agent");
        got.Add(pt->money, "credit");
        e->soldiers += pt->soldiers / div;
        e->fighters += pt->fighters / div;
        e->generals += pt->generals / div;
        e->heavycruisers += pt->heavycruisers / div;
        e->carriers += pt->carriers / div;
        e->stations += pt->stations / div;
        e->agents += pt->agents / div;
        e->food += pt->food;
        e->money.add(pt->money);
        pt->hatred[e->id]++;
        Shrink(&pt->soldiers, -pt->soldiers / div);
        Shrink(&pt->fighters, -pt->fighters / div);
        Shrink(&pt->generals, -pt->generals / div);
        Shrink(&pt->heavycruisers, -pt->heavycruisers / div);
        Shrink(&pt->carriers, -pt->carriers / div);
        Shrink(&pt->stations, -pt->stations / div);
        Shrink(&pt->agents, -pt->agents / div);
        pt->food = 0;
        pt->money = 0;
        if (r_pirateraids.Bool()) {
            const char *name = PirateName(pt);
            g_log2 << c_empire << Untr(e->name) << c_normal << " raided " << Color(12) << name
                   << c_normal << ".\n";
        }
        if (Random(13) == 0 && ShowScreen(0, "RaidWar", 0))
            Pause();
        kout << "You lost " << lost;
        kout << ", and you ";
        if (score > 10)
            kout << "slaughtered";
        else if (score > 8)
            kout << "crushed";
        else if (score > 5)
            kout << "humiliated";
        else
            kout << "defeated";
        kout << " the ";
        PutPirate(kout, pt) << c_normal << ".  ";
        kout << "You captured " << got << ".\n";
    } else {
        if (r_pirateraids.Bool()) {
            const char *name = PirateName(pt);
            g_log2 << c_empire << Untr(e->name) << c_normal << " attempted to raid " << Color(12)
                   << name << c_normal << ", but failed.\n";
        }
        kout << "You lost " << lost;
        kout << "; you were unable to defeat the ";
        PutPirate(kout, pt) << c_normal << ".\n";
    }
    return 1;
}

// 5fe0:19d9
int16 SpyOnPirates()
{
    int16 n = PirateCount();
    Menu m("PirateSpy", "Spy Report on Pirate Teams", 7, 6, 7, 7, 7, 7, 7, 7, 7, 7);
    m.Add("Team", 0, 100);
    m.Find(100)->AddColumn("Planets");
    m.Find(100)->AddColumn("Net Worth");
    for (int16 i = 1; i <= n; i++) {
        Pirate *pt = &g_pirates[i - 1];
        m.Add(PirateName(pt), (char)('0' + i), i);
        MenuItem *it = m.Find(i);
        uint8 c = PirateColor(pt);
        it->colors[0] = (uint8)(c & 7);
        it->colors[1] = 0xf;
        it->colors[2] = c;
        it->colors[3] = 0xb;
        it->colors[4] = 0xd;
        it->colors[5] = 7;
        it->colors[6] = 7;
        it->colors[7] = 7;
        it->LoadColors();
        it->AddColumn(PiratePlanets(pt));
        it->AddColumn(PirateNetWorth(pt));
    }
    m.Show();
    return 0;
}
