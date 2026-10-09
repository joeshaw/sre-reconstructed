// Attacks: the attack menu (segment 5090), conventional invasions (3b7b),
// guerilla ambushes (545e) and nuclear, chemical and psionic weapons
// (5c5c).
#include "sre.h"
#include "itemlist.h"
#include "report.h"
#include "battle.h"

static Resource r_maxattacks("Attacks.Maximum", (int32)-1);           // 384a
static Resource r_conventional("Attacks.Conventional", (int32)1);     // 3856
static Resource r_guerilla("Attacks.Guerilla", (int32)1);             // 3862
static Resource r_pirateraid("Attacks.PirateRaid", (int32)1);         // 386e
static Resource r_piratespy("Attacks.PirateSpy", (int32)1);           // 387a

int16 PirateRaid(Player *p, Empire *e);     // 5fe0:0297
int16 SpyOnPirates();                       // 5fe0:19d9

static int8 Clamp100(int16 v)
{
    int8 b = (int8)v;
    if (b < 0)
        b = 0;
    if (b > 100)
        b = 100;
    return b;
}

// The treaty an attack breaks (SetTreaty inlined, and the state reset).
static void BreakTreaty(Treaty &t)
{
    SetTreaty(t, 0, 0);
    t.setc(0);
}

static void BreakTreaties(Empire *a, Empire *b)
{
    BreakTreaty(a->treaty[b->id]);
    BreakTreaty(b->treaty[a->id]);
}

// 5090:0000: attacking ends the attacker's protection.
static int16 VoidProtection(Empire *e)
{
    if (e->year >= 0)
        return 1;
    const char *what = 0;
    if (e->stations < 200)
        what = "defense";
    else if (e->soldiers < 200)
        what = "soldiers";
    if (what) {
        kout << "You do not have enough " << what << " to be out of protection yet.\n";
        return 0;
    }
    if (TotalPlanets(e) < 200) {
        kout << "Your empire is too small" << " to be out of protection yet.\n";
        return 0;
    }
    if (!YesNo(0, "Are you sure you would like to void your protection by attacking"))
        return 0;
    if (e->year < 0) {
        e->year = 0;
        e->score -= (int32)e->year;
        if (e->score < 0)
            e->score = 0;
    }
    return 1;
}

// 5c5c:0000
static int16 CanAttack(Empire *me, Empire *t)
{
    if (t->year < 0) {
        kout << "You can not attack " << c_number << Untr(t->name) << c_normal
             << " because that empire is under protection!\n";
        return 0;
    }
    Treaty &tr = me->treaty[t->id];
    if (tr.type() != 0) {
        kout << "You have a " << Color(10) << TreatyName(tr.type()) << c_normal;
        kout << " with " << c_number << Untr(t->name) << c_normal << ".\n";
        if (tr.type() == 1) {
            kout << "Your Neutrality Treaty prevents you from attacking.\n";
            return 0;
        }
        if (tr.a != 0) {
            kout << "You cannot attack your allies unless you first break the treaty from the Diplomacy Menu.\n";
            return 0;
        }
        if (!YesNo(0, "Do you wish to continue"))
            return 0;
    }
    if (!Exists(*t) || TotalPlanets(t) <= 0 || t->population <= 0) {
        kout << "That empire has collapsed!\n";
        return 0;
    }
    if (!VoidProtection(me))
        return 0;
    BreakTreaties(me, t);
    return 1;
}

// 3b7b:0000
static void Lose(int32 *v, int32 n)
{
    *v -= n;
    if (*v < 0)
        *v = 0;
}

// 3b7b:0035: the first of the forces that is left takes the damage.
static void Hit(int32 *a, int32 da, int32 *b, int32 db, int32 *c, int32 dc, int32 *d, int32 dd)
{
    if (*a)
        Lose(a, da);
    else if (*b)
        Lose(b, db);
    else if (*c)
        Lose(c, dc);
    else
        Lose(d, dd);
}

// 3b7b:00a9: b * a / c, reduced to stay within range.
static int32 Proportion(int32 a, int32 b, int32 c)
{
    while (a > 10000 && c > 10000) {
        a /= 10;
        c /= 10;
    }
    while (b > 10000 && c > 10000) {
        b /= 10;
        c /= 10;
    }
    for (int16 k = 2; k <= 7; k++) {
        while (a % k == 0 && c % k == 0) {
            a /= k;
            c /= k;
        }
        while (b % k == 0 && c % k == 0) {
            b /= k;
            c /= k;
        }
    }
    return b * a / c;
}

// Lists kept between attacks (3b7b:2aa9).
static ItemList s_forces(Color(7), Color(11));     // DS:063e
static ItemList s_captured(Color(7), Color(14));   // DS:064e
static ItemList s_lost(Color(7), Color(12));       // DS:065e
static ItemList s_killed(Color(7), Color(11));     // DS:066e

// 3b7b:023b
static void ShowVictory(Color c, int32 credits, ItemList *planets)
{
    if (Random(5) == 0 && ShowScreen(0, "WinWar", 0))
        Pause();
    kout << "You have won the battle!\n\n";
    kout << "You capture " << c << credits << c_normal << " credits";
    if (planets->count > 0)
        kout << " and " << *planets << " planets";
    kout << ".\n\n";
}

// 3b7b:0325
static int16 Conventional(Player *p, Empire *me, Empire *t)
{
    (void)p;
    int32 heavy = me->heavycruisers;
    int32 light = me->lightcruisers;
    int32 stations = t->stations;           // defense stations fight like fighters
    int32 tlight = t->lightcruisers;
    int16 score = 0;
    if (!CanAttack(me, t))
        return 0;
    s_forces.count = 0;
    s_forces.skipzero = 0;
    s_forces.Add(me->fighters, "fighter");
    s_forces.Add(me->carriers, "carrier");
    s_forces.Add(me->soldiers, "soldier");
    s_forces.Add(me->generals, "general");
    s_forces.Add(me->heavycruisers, "heavy cruiser");
    s_forces.Add(me->lightcruisers, "light cruiser");
    if ((me->weapons >> 3) & 1)
        s_forces.Add(1, "mothership");
    kout << "You have " << s_forces << '.';
    // (Any carrier or mothership lets up to 100,000 fighters fly.)
    int32 fcap = (100 * me->carriers + ((me->weapons >> 3) & 1)) != 0 ? 100000 : 0;
    int32 scap = 50 * me->generals <= me->soldiers ? 50 * me->generals : me->soldiers;
    fcap = me->fighters < fcap ? me->fighters : fcap;
    int16 nogen = me->soldiers > scap, nocarrier = me->fighters > fcap;
    if (nogen || nocarrier)
        kout << "  You do not have enough";
    if (nogen)
        kout << " generals";
    if (nogen && nocarrier)
        kout << " and";
    if (nocarrier)
        kout << " carriers";
    if (nogen || nocarrier)
        kout << " to lead all your forces into battle.";
    kout << '\n';
    int32 sa, fa, la, ha;
    kout << "Take how many soldiers?  ";
    kin >> defval(scap) >> maxval(scap) >> sa;
    kout << "Take how many fighters?  ";
    kin >> defval(fcap) >> maxval(fcap) >> fa;
    kout << "Take how many light cruisers?  ";
    kin >> defval(light) >> maxval(light) >> la;
    kout << "Take how many heavy cruisers?  ";
    kin >> defval(heavy) >> maxval(heavy) >> ha;
    int32 sentf = fa, sents = sa, senth = ha, sentl = la;
    if (sa + fa + ha + la <= 0) {
        kout << Color(12) << "ABORTED" << c_normal << '\n';
        return 0;
    }
    BreakTreaties(me, t);
    // Allies of the defender send some of their forces.
    int32 allys[25], allyh[25];
    int32 sd = 0, hd = 0;
    for (int16 i = 0; i < 25; i++) {
        allyh[i] = 0;
        allys[i] = 0;
        Empire *a = &g_empires[i];
        if (!Exists(*a))
            continue;
        if (a->treaty[t->id].c() != 0)
            continue;
        allys[i] = (int32)a->treaty[t->id].e() * (a->soldiers / 100);
        allyh[i] = (int32)a->treaty[t->id].d() * (a->heavycruisers / 100);
        sd += allys[i];
        hd += allyh[i];
    }
    if (sd > 0 || hd > 0)
        kout << c_number << Untr(t->name) << c_normal << "'s allies send " << c_number << sd
             << c_normal << " soldiers and " << c_number << hd << c_normal << " heavy cruisers.  ";
    int32 drafted = (int32)t->draftrate * (t->population / 100);
    t->population -= drafted / 10;
    sd += drafted;
    if (drafted > 0)
        kout << c_number << drafted << c_normal << " soldiers are drafted.  ";
    kout << '\n';
    sd += t->soldiers;
    int32 sd0 = sd;
    hd += t->heavycruisers;
    int32 hd0 = hd;
    kout << Color(12);
    int32 ds = (sa + sd) / 50 + 1;
    int32 df = (fa + stations) / 50 + 1;
    int32 dh = (ha + hd) / 50 + 1;
    int32 dl = (la + tlight) / 50 + 1;
    int32 qa = (sa + fa + ha + la) / 10;
    int32 qd = (sd + stations + hd + tlight) / 10;
    int16 round = 0;
    while (sa + fa + ha + la > qa && sd + stations + hd + tlight > qd) {
        if (round >= 3 && Random(200) < t->effectiveness + 90)
            Hit(&sa, ds, &ha, dh, &fa, df, &la, dl);
        if (round >= 3 && Random(200) < me->effectiveness + 90)
            Hit(&sd, ds, &hd, dh, &stations, df, &tlight, dl);
        if (round >= 2 && Random(200) < 100)
            Hit(&fa, df, &sa, ds, &ha, dh, &la, dl);
        if (round >= 2 && Random(200) < 100)
            Hit(&stations, df, &sd, ds, &hd, dh, &tlight, dl);
        // (The original mixes in the defender's light cruisers here.)
        if (round >= 5 && Random(200) < t->cmdship / 2 + 50)
            Hit(&ha, dh, &tlight, dl, &fa, df, &sa, ds);
        if (round >= 5 && Random(200) < me->cmdship / 2 + 50)
            Hit(&hd, dh, &tlight, dl, &stations, df, &sd, ds);
        if (round >= 0 && Random(200) < 100)
            Hit(&la, dl, &ha, dh, &fa, df, &sa, ds);
        if (round >= 0 && Random(200) < 100)
            Hit(&tlight, dl, &hd, dh, &stations, df, &sd, ds);
        kout << '.';
        if (round++ > 0x4c)
            break;
    }
    kout << c_normal << '\n';
    // Score each kind of force.
    struct { int32 a, d; } f[4] = { { sa, sd }, { fa, stations }, { ha, hd }, { la, tlight } };
    for (int16 i = 0; i < 4; i++) {
        if (f[i].a > 0)
            score++;
        if (2 * f[i].d < f[i].a)
            score++;
        if (f[i].a < 0)
            score--;
        if (f[i].d > 0)
            score--;
        if (2 * f[i].a < f[i].d)
            score--;
        if (f[i].d < 0)
            score++;
    }
    Color cc(14);
    s_captured.count = 0;
    s_captured.skipzero = 0;
    s_captured.SkipZero();
    int16 won = 0;
    int32 credits = 0;
    if (score > 0) {
        won = 1;
        int32 total = 0;
        score *= 2;
        if (NetWorth(me) > NetWorth(t) && TotalPlanets(me) > TotalPlanets(t))
            score = score / 3 + 1;
        int16 div = (int16)(20 / score + 1);
        if (div < 3)
            div = 3;
        int16 take[10];
        for (int16 i = 0; i < 10; i++) {
            PlanetSet *ps = Planets(t, i);
            if (ps->count < 0)
                ps->count = 0;
            take[i] = (int16)(ps->count / div);
            if (take[i] == 0 && ps->count > 0)
                take[i] = 1;
            if (take[i] == 1 && ps->count > 1)
                take[i] = 2;
            if ((int32)take[i] > ps->count)
                g_errlog << "Player::ConventionalAttack() - pl/planets incorrect assignment.\n";
            total += take[i];
        }
        while (TotalPlanets(t) / div > total && TotalPlanets(t) > total) {
            int16 k = Random(10);
            if (Planets(t, k)->count > take[k]) {
                take[k]++;
                total++;
            }
        }
        credits = t->money.get() / div;
        me->effectiveness = Clamp100(me->effectiveness - 13);
        me->cmdship = Clamp100(me->cmdship - 7);
        t->effectiveness = Clamp100(t->effectiveness - 3);
        int8 r = (int8)(Random(5) + 2);
        t->cmdship = Clamp100(t->cmdship - r);
        for (int16 i = 0; i < 10; i++) {
            Planets(me, i)->count += take[i];
            Planets(t, i)->count -= take[i];
        }
        me->money.add(credits);
        me->score += 17 * total + t->score * score / 10;
        t->money.sub(credits);
        t->score -= 7 * total + t->score * score / 20;
        if (t->score < 0)
            t->score = 0;
        if (TotalPlanets(t) <= 0) {
            me->score += t->score;
            t->score = 0;
        }
        for (int16 i = 0; i < 10; i++)
            if (take[i] > 0)
                s_captured.Add(take[i], PlanetName((int8)i), "");
    } else {
        int16 r1 = Random(7);
        int16 r2 = Random(3);
        int8 r = (int8)(r1 + r2 + 3);
        me->cmdship = Clamp100(me->cmdship - r);
        int8 bonus = (int8)(score << 1);
        me->effectiveness = Clamp100(me->effectiveness + bonus);
        t->cmdship = Clamp100(t->cmdship - 5);
        t->effectiveness = Clamp100(t->effectiveness - 4);
        me->score -= Random(100) + 110;
        if (me->score < 0)
            me->score = 0;
    }
    AttackNews(me, t, score > 0, score);
    // A third of the destroyed forces were really lost.
    int32 lf = (sentf - fa) / 3;
    int32 ls = (sents - sa) / 3;
    int32 lh = (senth - ha) / 3;
    int32 ll = (sentl - la) / 3;
    int32 lg = (ls + 50) / 200;
    int32 lc = (lf + 200) / 1000;
    if (me->carriers < lc)
        lc = me->carriers;
    me->fighters -= lf;
    me->heavycruisers -= lh;
    me->lightcruisers -= ll;
    me->soldiers -= ls;
    me->carriers -= lc;
    me->generals -= lg;
    s_lost.count = 0;
    s_lost.skipzero = 0;
    s_lost.SkipZero();
    s_lost.Add(ls, "soldier");
    s_lost.Add(lf, "fighter");
    s_lost.Add(ll, "light cruiser");
    s_lost.Add(lh, "heavy cruiser");
    s_lost.Add(lg, "general");
    s_lost.Add(lc, "carrier");
    int32 kst = (t->stations - stations) / 3;
    int32 ks = (sd0 - sd) / 3;
    int32 kh = (hd0 - hd) / 3;
    int32 kl = (t->lightcruisers - tlight) / 3;
    s_killed.count = 0;
    s_killed.skipzero = 0;
    s_killed.SkipZero();
    s_killed.Add(ks, "soldier");
    s_killed.Add(kst, "defense station");
    s_killed.Add(kl, "light cruiser");
    s_killed.Add(kh, "heavy cruiser");
    for (int16 i = 0; i < 25; i++) {
        if (t->id == i)
            continue;
        if (allys[i] <= 0 && allyh[i] <= 0)
            continue;
        Empire *a = &g_empires[i];
        if (!Exists(*a))
            continue;
        int32 als = 0, alh = 0;
        if (sd0 > 0 && allys[i] > 0)
            als = Proportion(ks, allys[i], sd0);
        if (hd0 > 0 && allyh[i] > 0)
            alh = Proportion(kh, allyh[i], hd0);
        if (score < 0 && a->effectiveness > 0)
            a->effectiveness = Clamp100(a->effectiveness - 1);
        a->soldiers -= als;
        a->heavycruisers -= alh;
        if (score < 0)
            a->score -= t->score / 30;
        if (score > 0)
            a->score += me->score / 40;
        BattleReport r(i);
        r << "You defended " << c_empire << Untr(t->name) << c_normal << " against " << Color(14)
          << Untr(me->name) << c_normal << " and lost " << c_number << als << c_normal
          << " soldiers and " << c_number << alh << c_normal << " heavy cruisers.";
    }
    if (sd0 > 0)
        t->soldiers -= Proportion(ks, t->soldiers, sd0);
    if (hd0 > 0)
        t->heavycruisers -= Proportion(kh, t->heavycruisers, hd0);
    t->stations -= kst;
    t->lightcruisers -= kl;
    {
        BattleReport r(t->id);
        AttackMessage(r, me->name, score);
        me->w229++;
        if (won) {
            ShowVictory(cc, credits, &s_captured);
        } else {
            if (Random(5) == 0 && ShowScreen(0, "LoseWar", 0))
                Pause();
            kout << "You have lost the battle.\n";
        }
        kout << "You lost " << s_lost << ".  ";
        kout << c_empire << Untr(t->name) << c_normal << " lost " << s_killed << ".\n";
        if (won && TotalPlanets(t) <= 0)
            kout << Color(12) << '\n' << Untr(t->name) << Color(14)
                 << " is completely destroyed!!!\n" << c_normal;
    }
    return 1;
}

// 545e:0000
static int16 Guerilla(Player *p, Empire *me, Empire *t)
{
    (void)p;
    int32 theirs = t->soldiers;
    int16 score = 0, rounds = 0;
    if (!CanAttack(me, t))
        return 0;
    kout << "You have " << c_number << me->soldiers << c_normal << " soldiers.\n";
    int32 mine, retreat;
    kout << "How many soldiers to send?  ";
    kin >> defval(me->soldiers) >> maxval(me->soldiers) >> mine;
    kout << "Withdraw after loss of how many soldiers?  ";
    kin >> defval(mine) >> maxval(mine) >> retreat;
    if (mine == 0 || retreat == 0) {
        kout << Color(12) << "ABORTED" << c_normal << '\n';
        return 0;
    }
    int32 sent = mine;
    retreat = mine - retreat;
    if (retreat == 0)
        kout << "A battle to the death...\n";
    kout << Color(13);
    while (mine > retreat && theirs > 0 && rounds < 0x4e) {
        int16 a = Random((int16)(mine > 30000 ? 30000 : mine));
        int16 b = Random((int16)(theirs > 30000 ? 30000 : theirs));
        if (a < b)
            mine -= theirs / 300 + 1;
        a = Random((int16)(mine > 30000 ? 30000 : mine));
        b = Random((int16)(mine > 30000 ? 30000 : mine));
        if (a < b)
            mine -= mine / 500 + 1;
        a = Random((int16)(theirs > 30000 ? 30000 : theirs));
        b = Random((int16)(mine > 30000 ? 30000 : mine));
        if (a < b)
            theirs -= mine / 100 + 1;
        rounds++;
        kout << 'x';
    }
    kout << c_normal << '\n';
    if (mine > retreat)
        score++;
    if (mine < retreat)
        score--;
    if (theirs > 0)
        score--;
    if (theirs < 0)
        score++;
    if (Random(13) == 0 && ShowScreen(0, "Guerilla", 0))
        Pause();
    if (mine < 0)
        mine = 0;
    int32 lost = sent - mine;
    if (me->soldiers < lost)
        lost = me->soldiers;
    kout << "You lost " << c_number << lost << c_normal << " soldiers.\n";
    me->soldiers -= lost;
    me->w229++;
    int32 before = t->soldiers;
    t->soldiers -= (t->soldiers - theirs) / 3;
    if (t->soldiers < 0)
        t->soldiers = 0;
    int32 after = t->soldiers;
    int16 secret = 0, riot = 0;
    if (score > 0) {
        secret = 1;
        me->score += (int32)(int16)(score * 10);
        me->effectiveness = Clamp100(me->effectiveness - 10);
        if (me->effectiveness < 0)
            me->effectiveness = Clamp100(0);
        t->effectiveness = Clamp100(t->effectiveness - 5);
        if (t->insurgency < 7 && Random(2) == 0) {
            riot = 1;
            InsurgencyUp(t);
        }
    } else {
        int8 bonus = (int8)(score << 1);
        me->effectiveness = Clamp100(me->effectiveness + bonus);
        me->score -= (int32)(int16)(Random(200) + 0xdc - score * 0x28);
        if (me->score < 0)
            me->score = 0;
        BreakTreaties(me, t);
        BattleReport r(t->id);
        r << c_empire << Untr(me->name) << c_normal
          << " attempted a secret guerilla invasion of your empire!";
        r << "  You lost " << c_number << before - after << c_normal << " soldiers defending.";
        g_log2 << c_empire << Untr(me->name) << c_normal << " attempted a guerilla invasion of "
               << Color(9) << Untr(t->name) << c_normal << ".\n";
    }
    kout << "You lost " << c_number << lost << c_normal << " soldiers.\n";
    if (secret)
        kout << "You have weakened the enemy without revealing your identity.\n";
    else
        kout << "Your immoral acts have been discovered and reported to " << c_number
             << Untr(t->name) << c_normal << ".\n";
    if (riot)
        kout << "Riots break out in your opponent's empire due to your ambush.\n";
    return 1;
}

// 5c5c:060b
static int16 Nuclear(Player *p, Empire *me, Empire *t)
{
    int16 div = (int16)(100 / g_galaxy.w3b4 + 1);
    if (!(me->weapons & 1)) {
        kout << "You have no nuclear weapons!\n";
        return 0;
    }
    if (!CanAttack(me, t))
        return 0;
    if (!YesNo(0, "The Galactic Coordinator has outlawed nuclear weapons due to the environmental destruction they cause.  By using the bomb, you will be seriously angering It.\nAre you sure you wish to do this"))
        return 0;
    BreakTreaties(me, t);
    me->weapons &= ~1;
    me->w229++;
    if (g_galaxy.w3b2 > 50)
        g_galaxy.w3b2 -= 30;
    if (g_galaxy.w3b4 > 3)
        g_galaxy.w3b4--;
    if (Random(33) == 0) {
        // The bomb goes off at home.
        int32 tp = TotalPlanets(me);
        while (tp / div > g_galaxy.w3b2 && div < 100)
            div++;
        for (int16 i = 0; i < 10; i++)
            Planets(me, i)->count -= Planets(me, i)->count / div;
        int32 lost = tp - TotalPlanets(me);
        g_log2 << c_empire << Untr(me->name) << c_normal << " was hit by " << PlayerPronoun(p, 2)
               << " own nuclear bomb!\n";
        kout << "Your bomb was booby-trapped, and you completely destroyed " << lost
             << " of your own planets with the Nuclear Bomb!\n";
        return 1;
    }
    int32 tp = TotalPlanets(t);
    while (tp / div > g_galaxy.w3b2 && div < 100)
        div++;
    for (int16 i = 0; i < 10; i++)
        Planets(t, i)->count -= Planets(t, i)->count / div;
    int32 lost = tp - TotalPlanets(t);
    BattleReport r(t->id);
    r << "You lost " << c_number << lost << c_normal << " planets in a Nuclear Attack from "
      << c_empire << Untr(me->name) << c_normal << '!';
    me->d1b0 /= 2;
    g_log2 << Color(9) << Untr(t->name) << c_normal << " was hit by a nuclear bomb traced to "
           << c_empire << Untr(me->name) << c_normal << ".\n";
    if (Random(3) == 0 && ShowScreen(0, "NukeWar", 0))
        Pause();
    kout << "You completely destroyed " << lost << " planets with the Nuclear Bomb!\n";
    return 1;
}

// 5c5c:0f19
static int16 Chemical(Player *p, Empire *me, Empire *t)
{
    if (!((me->weapons >> 1) & 1)) {
        kout << "You have no chemical weapons!\n";
        return 0;
    }
    if (!CanAttack(me, t))
        return 0;
    if (!YesNo(0, "The Galactic Coordinator has outlawed chemical weapons.  By using the barrel of chemicals, you will be seriously angering It.  Are you sure you wish to do this"))
        return 0;
    me->d1b0 = 0;
    BreakTreaties(me, t);
    me->weapons &= ~2;
    me->w229++;
    if (Random(10) == 0 && t->agents > 0) {
        // The target's agents sabotage the weapon.
        int32 killed = me->population / 2;
        me->population -= killed;
        {
            ReportOut r(t->id, 0);
            r << c_empire << Untr(me->name) << c_normal
              << " attempted to use chemical weapons on you; your covert intelligence discovered their plot and sabotaged "
              << PlayerPronoun(p, 2) << " weapon!\n";
            g_log2 << "The ruler of " << c_empire << Untr(me->name) << c_normal << " killed "
                   << killed << " million of " << PlayerPronoun(p, 2)
                   << " own civilians with a chemical weapon!\n";
            Player tp;
            PlayerLoad(&tp, t->player);
            kout << "Your opponent sent " << PlayerPronoun(&tp, 2)
                 << " agents to sabotage your weapon.  You kill " << killed
                 << " million of your own civilians!\n";
            PlayerSave(&tp);
        }
        return 1;
    }
    int32 killed = t->population / 2;
    t->population -= killed;
    BattleReport r(t->id);
    r << c_number << killed << c_normal << " million civilians were killed by chemical attacks by "
      << c_empire << Untr(me->name) << c_normal << '.';
    g_log2 << c_empire << Untr(me->name) << c_normal
           << " used chemical weapons to kill innocent civilians in " << Color(9) << Untr(t->name)
           << c_normal << ".\n";
    if (Random(3) == 0 && ShowScreen(0, "ChemWar", 0))
        Pause();
    kout << "You kill " << killed << " million innocent civilians.\n";
    return 1;
}

// 5c5c:1862
static int16 Psionic(Player *p, Empire *me, Empire *t)
{
    (void)p;
    if (!((me->weapons >> 2) & 1)) {
        kout << "You have no psionic bomb!\n";
        return 0;
    }
    if (!CanAttack(me, t))
        return 0;
    if (!YesNo(0, "Are you sure you wish to attack with the psionic weapon"))
        return 0;
    BreakTreaties(me, t);
    me->weapons &= ~4;
    me->w229++;
    t->insurgency = 7;
    t->effectiveness = 0;
    BattleReport r(t->id);
    r << "A psionic weapon from " << c_empire << Untr(me->name) << c_normal
      << " devastated your population.\n";
    me->d1b0 /= 2;
    g_log2 << Color(9) << Untr(t->name) << c_normal << " was hit by a psionic bomb from "
           << c_empire << Untr(me->name) << c_normal << ".\n";
    kout << "You brought " << Untr(t->name) << " down to UNDER COUP.\n";
    return 1;
}

// 5090:0121: returns 1 when an attack was made.
int16 Attack(Player *p, Empire *e, char how)
{
    for (;;) {
        Menu m("Attack", "Attack Options", 0xf, 6, 6, 0xf, 0xe, 7, 7, 7, 7, 7);
        char key = '1';
        if (r_conventional.Bool())
            m.Add("Conventional Invasion", key++, 1);
        if (r_guerilla.Bool())
            m.Add("Guerilla Ambush", key++, 2);
        if (r_pirateraid.Bool())
            m.Add("Pirate Raid", key++, 3);
        if (r_piratespy.Bool())
            m.Add("Spy on Pirates", key++, 4);
        if (e->weapons & 1)
            m.Add("Nuclear Offensive", key++, 5);
        if ((e->weapons >> 1) & 1)
            m.Add("Chemical Warfare", key++, 6);
        if ((e->weapons >> 2) & 1)
            m.Add("Psionic Attack", key++, 7);
        if (g_system.regstate)
            m.Add("", 'V', 0x50);
        if (key == '1')
            return 0;
        int16 r = -1;
        for (int16 i = 1; i <= 7; i++) {
            MenuItem *it = m.Find(i);
            if (it && it->key == how) {
                r = it->id;
                how = 0;
            }
        }
        if (r == -1)
            r = m.Run(1, -1);
        if (r == -1)
            return 0;
        if (r == 0x50) {
            Bank(p, e, &g_galaxy);
            continue;
        }
        if (r == 3) {
            if (PirateRaid(p, e))
                return 1;
            continue;
        }
        if (r == 4) {
            SpyOnPirates();
            return 1;
        }
        if (r_maxattacks.Long() != -1 && (int32)e->w229 >= r_maxattacks.Long()) {
            kout << "The " << Color(15) << "Galactic Coordinator" << c_normal;
            kout << " will not let you attack more than " << c_number;
            kout << r_maxattacks.Long() << c_normal << " times per " << c_number;
            kout << g_galaxy.grantturns << c_normal << " turns.\n";
            return 0;
        }
        kout << Color(14) << "Attack Empire:\n";
        int32 allowed = EmpireMask() & ~ProtectedMask();
        MaskClear(&allowed, e->id);
        int16 target = ChooseEmpire(allowed);
        if (target == -1)
            continue;
        int16 done = 0;
        do {
            Empire *t = &g_empires[target];
            switch (r) {
            case 1: done = Conventional(p, e, t); break;
            case 2: done = Guerilla(p, e, t); break;
            case 5: done = Nuclear(p, e, t); break;
            case 6: done = Chemical(p, e, t); break;
            case 7: done = Psionic(p, e, t); break;
            }
            if (!done) {
                target = ChooseEmpire(allowed);
            } else if (TotalPlanets(t) <= 0) {
                ReportOut news(0);
                news << c_empire << Untr(t->name) << c_normal << " has been wiped out by "
                     << Color(14) << Untr(e->name) << c_normal << '.';
                news.rep.AddAll();
            }
        } while (!done && target != -1);
        if (done)
            return 1;
    }
}
