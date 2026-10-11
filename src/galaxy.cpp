// The galaxy (segment 5222).
#include "sre.h"

// 5222:0112
void GalaxyInit(Galaxy *g)
{
    g->name[0] = 0;
    g->created = g_today;
    g->lasthourly = g_today;
    g->startdate = g_today;
    g->lastregen = g_now;
    g->joindeadline = NeverDate();
    g->enddate = NeverDate();
    g->inflation = 0;
    g->localonly = 0;
    g->interest = 3;
    for (int16 i = 0; i < MAXEMPIRES; i++)
        g->empire[i].clear();
    for (int16 i = 1; i <= MAXPIRATES; i++)
        g->pirate[i - 1].clear();
    g->weaponsmaster = -1;
    g->planetmaster = -1;
    g->solarmaster = -1;
    g->grantturns = 5;
    g->protection = 20;
    g->w3a2 = 0;
    g->lotterytickets = 0;
    g->lotterywinner = -1;
    g->jackpot = 0;
    g->coordfund = 10000;
    g->coordtax = 15;
    g->nukelimit = 300;
    g->nukepercent = 20;
    g->psionicprice = 10000000;
    g->newspaper = 0;
    g->daylength = 1;
    g->marketfood = 7000000;
    g->regenseconds = 3;
    g->marketplanets = 100;
    g->planetsperturn = -1;
    g->oilprice = 0;
}

// 5222:0339
void GalaxyReset(Galaxy *g)
{
    GalaxyInit(g);
    for (int16 i = 0; i < MAXEMPIRES; i++)
        EmpireInit(&g_empires[i]);
    SystemSaveAll();
}

// 5222:037e
void GalaxySetName(Galaxy *g, const String &s)
{
    strncpy(g->name, s.c_str(), 0x3f);
    g->name[0x3f] = 0;
}

// 5222:03b4: the price of petroleum follows supply and demand.
void GalaxyOilPrice(Galaxy *g)
{
    if (!g_open)
        return;
    int32 planets = 0, petroleum = 0;
    for (int16 i = 0; i < MAXEMPIRES; i++) {
        Empire *e = &g_empires[i];
        planets += TotalPlanets(e);
        petroleum += e->planets[P_PETROLEUM].count;
    }
    g->oilprice = (300 * planets) / (petroleum + 10);
    if (g->oilprice < 700)
        g->oilprice = 700;
    if (g->oilprice > 100000)
        g->oilprice = 100000;
}

// 5222:04a8
void GalaxyOpen()
{
    if (g_open++ == 0)
        g_open = SystemLoadData();
}

// 5222:04c2
void GalaxyClose()
{
    if (!g_open)
        return;
    if (--g_open != 0)
        return;
    SystemSaveAll();
    g_current = 0;
}

// 5222:04f9
void ScoreListAll(Galaxy *g)
{
    kout << '\n';
    ScoreList(g, EmpireMask(), 0);
}

static void ShowMaster(const char *what, Color c, int16 idx)
{
    kout << c_normal << what << c;
    PutUntranslated(kout, String(g_empires[idx].name));
    kout << c_normal << ".\n";
}

// 5222:052e
void ShowMasters(Galaxy *g)
{
    GalaxyOpen();
    if (g->planetmaster != -1)
        ShowMaster("Today's Planet Master is ", Color(11), g->planetmaster);
    if (g->solarmaster != -1)
        ShowMaster("Today's Solar Master is ", Color(14), g->solarmaster);
    if (g->weaponsmaster != -1)
        ShowMaster("Today's Weapons Master is ", Color(13), g->weaponsmaster);
    GalaxyClose();
}

static bool Alive(Empire *e)
{
    return Exists(*e) && TotalPlanets(e) > 0 && e->population > 0;
}

static char Letter(Empire *e)
{
    return e->id == -1 ? 0 : (char)((uint8)e->id + 'A');
}

// 14f6:01ed
int16 MaskTest(int32 mask, int16 bit)
{
    if ((uint16)bit > 0x18)
        return 0;
    return (int16)(((uint32)mask >> bit) & 1);
}

// 14f6:0213
void MaskSet(int32 *mask, int16 bit)
{
    if ((uint16)bit > 0x18)
        return;
    *mask |= (int32)(1UL << bit);
}

// 14f6:0237
void MaskClear(int32 *mask, int16 bit)
{
    if ((uint16)bit > 0x18)
        return;
    *mask &= ~(int32)(1UL << bit);
}

// 5222:07c8: the list of players and scores; empires in mask are marked.
void ScoreList(Galaxy *g, int32 mask, OStream *o)
{
    int16 ctitle = 0xe, cline = 8, ckey = 0xe, cbracket = 2;
    int16 c0 = 0xf, c1 = 0xb, c2 = 0xe, c3 = 0xd, c4 = 0xa;
    if (!g_open) {
        g_errlog << "Galaxy not open.\n";
        return;
    }
    {
        Resource rt("Scores.Colors.Title", (int32)ctitle);
        Resource rl("Scores.Colors.Line", (int32)cline);
        Resource rk("Scores.Colors.Key", (int32)ckey);
        Resource rb("Scores.Colors.Brackets", (int32)cbracket);
        Resource r0("Scores.Colors.Col0", (int32)c0);
        Resource r1("Scores.Colors.Col1", (int32)c1);
        Resource r2("Scores.Colors.Col2", (int32)c2);
        Resource r3("Scores.Colors.Col3", (int32)c3);
        Resource r4("Scores.Colors.Col4", (int32)c4);
        ctitle = (int16)rt.Long();
        ckey = (int16)rk.Long();
        cline = (int16)rl.Long();
        cbracket = (int16)rb.Long();
        c0 = (int16)r0.Long();
        c1 = (int16)r1.Long();
        c2 = (int16)r2.Long();
        c3 = (int16)r3.Long();
        c4 = (int16)r4.Long();
    }
    if (!o)
        o = &kout;
    int16 t;
    *o << Color(ctitle) << "List of Players/Scores:\n";
    *o << setw(4) << Color(ckey) << "ID";
    *o << setw(0x20) << Color(c0) << "Empire";
    *o << setwr(11) << Color(c1) << "Planets";
    t = g->enddate < NeverDate();
    *o << setwr(11 - t) << Color(c2) << "Score";
    t = g->enddate < NeverDate();
    *o << setwr(13 - t) << Color(c3) << "Net Worth";
    if (g->enddate < NeverDate())
        *o << setwr(7) << Color(c4) << "Points";
    *o << setw(0) << '\n' << Color(cline);
    for (int16 k = 0; k < (g->enddate < NeverDate()) * 5 + 0x47; k++)
        *o << '\xc4';
    *o << '\n';
    for (int16 i = 0; i < MAXEMPIRES; i++) {
        Empire *e = &g_empires[i];
        if (!Exists(*e))
            continue;
        if (o == &kout && kin.KeyReady())
            return;
        if (MaskTest(mask, i)) {
            *o << Color(cbracket) << '<' << Color(ckey) << Letter(e) << Color(cbracket) << '>';
        } else if (Alive(e)) {
            *o << Color(7) << ' ' << Letter(e) << ' ';
        } else {
            *o << Color(4) << ' ' << Letter(e) << ' ';
        }
        *o << ' ' << Color(c0) << setw(0x20);
        PutUntranslated(*o, String(e->name));
        *o << setw(0);
        if (!Alive(e)) {
            *o << setwr(0x16) << Color(12) << "DEAD!" << setw(0) << '\n';
            continue;
        }
        *o << Color(c1) << setw(11) << TotalPlanets(e) << setw(0);
        *o << Color(c2);
        t = g->enddate < NeverDate();
        *o << setw(11 - t) << e->score << setw(0);
        *o << Color(c3);
        t = g->enddate < NeverDate();
        *o << setw(13 - t) << NetWorth(e) << setw(0);
        if (g->enddate < NeverDate())
            *o << Color(c4) << setw(7) << e->points << setw(0);
        *o << c_normal << '\n';
    }
    *o << Color(cline);
    for (int16 k = 0; k < (g->enddate < NeverDate()) * 5 + 0x47; k++)
        *o << '\xc4';
    *o << c_normal << '\n';
}

// 5222:13bb
int16 GalaxyPlayable(Galaxy *g)
{
    if (g_today < g->startdate) {
        kout << "This game cannot be played until " << c_number;
        PutTime(kout, g->startdate) << c_normal << '\n';
        return 0;
    }
    if (g_today > g->enddate) {
        kout << "This game has ended.\n";
        return 0;
    }
    return 1;
}

// 5222:14d3
int32 EmpireMask()
{
    GalaxyOpen();
    int32 m = 0;
    for (int16 i = 0; i < MAXEMPIRES; i++)
        if (Exists(g_empires[i]))
            MaskSet(&m, i);
    GalaxyClose();
    return m;
}

// 5222:1539
int32 ProtectedMask()
{
    GalaxyOpen();
    int32 m = 0;
    for (int16 i = 0; i < MAXEMPIRES; i++)
        if (Exists(g_empires[i]) && g_empires[i].year < 0)
            MaskSet(&m, i);
    GalaxyClose();
    return m;
}

// 5222:2283
Resource r_inactivetimeout("Game.InactiveTimeOut", (int32)10);     // 3eaa
Resource r_failedcovert("News.FailedCovertOps", (int32)1);         // 3eb6
Resource r_attackvictim("News.AttackVictim", (int32)1);            // 3ec2
Resource r_pirateraids("News.PirateRaids", (int32)0);              // 3ece
Resource r_civilwar("News.CivilWar", (int32)1);                    // 3eda

void NameNormalize(String &s, int16 level);         // 7458:01de

// 5222:15b5: how well a search string matches a name (0 = not at all,
// else 1 + the normalization level needed).
static int16 NameMatch(String search, String name)
{
    for (int16 level = 0; level <= 3; level++) {
        NameNormalize(search, level);
        NameNormalize(name, level);
        if (name.find(search, 0) >= 0)
            return (int16)(level + 1);
    }
    return 0;
}

// 5222:1666: the empire whose name matches best, or -1 (after listing
// the matches if there are several).
int16 SearchEmpire(String search, int32 allowed)
{
    int16 score[MAXEMPIRES];
    int16 best = 0, count = 0;
    for (int16 i = 0; i < MAXEMPIRES; i++) {
        if (!MaskTest(allowed, i)) {
            score[i] = 0;
            continue;
        }
        score[i] = NameMatch(search, String(g_empires[i].name));
        if (score[i] > best) {
            best = score[i];
            count = 1;
        } else if (score[i] == best) {
            count++;
        }
    }
    if (count == 1) {
        for (int16 i = 0; i < MAXEMPIRES; i++)
            if (score[i] == best)
                return i;
    }
    if (count == 0 || best == 0) {
        kout << c_normal << "No match found.\n";
        return -1;
    }
    Resource rkey("Scores.Colors.Key", (int32)14);
    Resource rbrk("Scores.Colors.Brackets", (int32)4);
    Resource rcol("Scores.Colors.Col0", (int32)15);
    kout << c_normal << "Your string matches:\n";
    for (int16 i = 0; i < MAXEMPIRES; i++) {
        if (score[i] != best)
            continue;
        kout << Color((int16)rbrk.Long()) << " (" << Color((int16)rkey.Long())
             << (char)(g_empires[i].id + 'A') << Color((int16)rbrk.Long()) << ") "
             << Color((int16)rcol.Long()) << Untr(g_empires[i].name) << '\n';
    }
    return -1;
}

// 5222:19df: choose a set of empires by letter (Z toggles all); the
// result is in *chosen.  Returns 1 when done, 0 on ESC, or the key in
// exits that ended the choice.
int16 ChooseEmpires(int32 allowed, int32 *chosen, String exits, void (*list)(int32),
                    const char *prompt)
{
    String p(prompt ? prompt : "Select empires >> ");
    char sel[0x28];
    int16 n = 0;
    kout << c_normal << p << c_input;
    for (int16 i = 0; i < MAXEMPIRES; i++) {
        if (MaskTest(*chosen, i)) {
            sel[n++] = (char)(i + 'A');
            kout << (char)(i + 'A');
        }
    }
    for (;;) {
        char c;
        kout << c_input;
        kin >> c;
        if (c == '?') {
            kout << c_normal << '\n';
            list(allowed);
            kout << c_normal << p << c_input;
            for (int16 j = 0; j < n; j++)
                kout << sel[j];
        }
        if (c == '/') {
            char buf[0x100];
            kout << c_normal << "\nEmpire search: ";
            kin >> buf;
            if (buf[0]) {
                int16 k = SearchEmpire(String(buf), allowed);
                if (k != -1)
                    c = (char)(k + 'A');
            }
            kout << c_normal << p << c_input;
            for (int16 j = 0; j < n; j++)
                kout << sel[j];
        }
        if (c == 27 || exits.findch(c, 0) != -1) {
            kout << c_normal;
            for (int16 j = 0; j < n; j++)
                kout << "\b \b";
            for (int16 j = 0; DisplayLen(p) > j; j++)
                kout << "\b \b";
            if (c == 27)
                return 0;
            return (int16)c;
        }
        if (c == 8 && n > 0) {
            n--;
            kout << c_normal << "\b \b";
        }
        if (ctype_signed(c) & (CT_UPP | CT_LOW)) {
            c = (char)b_toupper((int)(int8)c);
            int16 all = c == 'Z';
            for (char x = all ? 'A' : c; x < (all ? 'Z' : c + 1); x++) {
                int16 found = 0;
                for (int16 j = 0; j < n; j++) {
                    if (sel[j] != x)
                        continue;
                    kout << c_normal << "\b ";
                    for (int16 k = j; k < n; k++)
                        kout << '\b';
                    n--;
                    for (int16 k = j; k < n; k++) {
                        sel[k] = sel[k + 1];
                        kout << c_input << sel[k];
                    }
                    found = 1;
                    break;
                }
                if (!found && MaskTest(allowed, (int16)(x - 'A'))) {
                    sel[n++] = x;
                    kout << c_input << x;
                }
            }
        }
        if (c == '\r' || c == '\n') {
            *chosen = 0;
            for (int16 j = 0; j < n; j++)
                MaskSet(chosen, (int16)(sel[j] - 'A'));
            kout << c_normal << '\n';
            return 1;
        }
    }
}

// 5222:1ecd: choose one empire; -1 for none.
int16 ChooseEmpire(int32 allowed)
{
    static String prompt("Choose an empire: ");     // DS:3ee8
    kout << c_normal << prompt << " (/ to search) ";
    for (;;) {
        char c;
        kout << c_input;
        kin >> c;
        if (c == '?') {
            kout << "List Empires" << c_normal << '\n';
            ScoreList(&g_galaxy, allowed, 0);
            kout << c_normal << prompt << c_input;
        }
        if (c == '/') {
            char buf[0x100];
            kout << c_normal << "Search: ";
            kin >> buf;
            if (buf[0]) {
                int16 k = SearchEmpire(String(buf), allowed);
                if (k != -1)
                    c = (char)(k + 'A');
            }
            kout << c_normal << prompt << c_input;
        }
        if (c == 27) {
            kout << "Abort" << c_normal << '\n';
            return -1;
        }
        if (c == '\r' || c == '\n') {
            kout << "None" << c_normal << '\n';
            return -1;
        }
        if (!(ctype_signed(c) & (CT_UPP | CT_LOW)))
            continue;
        c = (char)b_toupper((int)(int8)c);
        int16 id = (int16)(c - 'A');
        Resource rkey("Scores.Colors.Key", (int32)14);
        Resource rbrk("Scores.Colors.Brackets", (int32)4);
        if (!MaskTest(allowed, id))
            continue;
        kout << Color((int16)rbrk.Long()) << '(' << Color((int16)rkey.Long()) << c
             << Color((int16)rbrk.Long()) << ") " << c_empire << Untr(g_empires[id].name)
             << c_normal << '\n';
        return id;
    }
}
