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
    int32 ps = s + pt->d98;
    int32 pf = f + pt->d9c;
    int32 pc = c + pt->da4;
    kout << Color(13);
    for (;;) {
        if (Random(1100) < chance && pt->d98 > 0)
            Shrink(&pt->d98, -ps / 80 - 1);
        if (Random(1300) < chance && pt->da4 > 0)
            Shrink(&pt->da4, -pc / 80 - 1);
        if (Random(1200) < chance && pt->d9c > 0)
            Shrink(&pt->d9c, -pf / 80 - 1);
        if (Random(1200) < chance && pt->da0 > 0)
            Shrink(&pt->da0, -pt->da0 / 80 - 1);
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
        if (!(pt->d98 > 0 || pt->da4 > 0 || pt->d9c > 0))
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
    if (pt->d9c > 0)
        score--;
    if (pt->d98 > 0)
        score--;
    if (pt->da4 > 0)
        score--;
    if (s > 0)
        score++;
    if (c > 0)
        score++;
    if (f > 0)
        score++;
    if (pt->d9c > f)
        score--;
    if (pt->d98 > s)
        score--;
    if (pt->da4 > c)
        score--;
    if (pt->d9c < f)
        score++;
    if (pt->d98 < s)
        score++;
    if (pt->da4 < c)
        score++;
    if (2 * f < pt->d9c)
        score--;
    if (2 * s < pt->d98)
        score--;
    if (2 * c < pt->da4)
        score--;
    if (2 * pt->d9c < f)
        score++;
    if (2 * pt->d98 < s)
        score++;
    if (2 * pt->da4 < c)
        score++;
    if (2 * pt->d9c + pt->d98 + 13 * pt->da4 > 2 * f + s + 13 * c)
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
            int16 take = (int16)(pt->wb4[k] / div);
            if (take > 100)
                take = 100;
            if (take < 0)
                take = 0;
            Planets(e, k)->count += take;
            pt->wb4[k] = (int16)(pt->wb4[k] - take);
            if (pt->wb4[k] < 0)
                pt->wb4[k] = 0;
            total += take;
        }
        ItemList got(c_normal, Color(14));
        got.SkipZero();
        got.Add(total, "planet");
        got.Add(pt->d94, "megatons of food", "");
        got.Add(pt->d98 / div, "soldier");
        got.Add(pt->d9c / div, "fighter");
        got.Add(pt->da0 / div, "defense station");
        got.Add(pt->da4 / div, "heavy cruiser");
        got.Add(pt->da8 / div, "carrier");
        got.Add(pt->dac / div, "general");
        got.Add(pt->db0 / div, "covert agent");
        got.Add(pt->money, "credit");
        e->soldiers += pt->d98 / div;
        e->fighters += pt->d9c / div;
        e->generals += pt->dac / div;
        e->heavycruisers += pt->da4 / div;
        e->carriers += pt->da8 / div;
        e->stations += pt->da0 / div;
        e->agents += pt->db0 / div;
        e->food += pt->d94;
        e->money.add(pt->money);
        pt->wd4[e->id]++;
        Shrink(&pt->d98, -pt->d98 / div);
        Shrink(&pt->d9c, -pt->d9c / div);
        Shrink(&pt->dac, -pt->dac / div);
        Shrink(&pt->da4, -pt->da4 / div);
        Shrink(&pt->da8, -pt->da8 / div);
        Shrink(&pt->da0, -pt->da0 / div);
        Shrink(&pt->db0, -pt->db0 / div);
        pt->d94 = 0;
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
