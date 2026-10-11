// Empires (segments 48cc and 1c59).
#include "sre.h"

static const char PLANET_CODES[] = "PETFROGUSA";        // DS:9d69
static const int16 PLANET_OFFSET[10] = { 0x2c, 0x38, 0x44, 0x50, 0x5c,
                                         0x68, 0x74, 0x80, 0x8c, 0x98 };  // DS:9d48 - 1

// 1c59:0099: planets of a type (index 0..9, or a code letter).
PlanetSet *Planets(Empire *e, int16 type)
{
    if (type > 9) {
        for (int16 i = 0; i < 10; i++) {
            if ((int8)PLANET_CODES[i] == type) {
                type = i;
                break;
            }
        }
    }
    if (type >= 0 && type <= 9)
        return (PlanetSet *)((uint8 *)e + PLANET_OFFSET[type]);
    kout << "FATAL ERROR: planet-index=" << type << ".\n";
    g_errlog << "FATAL ERROR @Empire::Planets&( " << type << " ).\n";
    exit(1);
    return (PlanetSet *)((uint8 *)e + 0xe0);
}

// 1c59:004b
int32 TotalPlanets(Empire *e)
{
    int32 n = 0;
    for (int16 i = 0; i < 10; i++)
        n += Planets(e, i)->count;
    return n;
}

// 1c59:0009
void InsurgencyUp(Empire *e)
{
    if (e->insurgency < 7)
        e->insurgency++;
}

// 1c59:002a
void InsurgencyDown(Empire *e)
{
    if (e->insurgency > 0)
        e->insurgency--;
}

// 1c59:028b
int32 NetWorth(Empire *e)
{
    int32 t = 0;
    for (int16 i = 0; i < 25; i++) {
        Treaty tr = e->treaty[i];
        if (tr.type() == TT_NONE)
            continue;
        t += ((int32)tr.days + 5) / 3;
        if (tr.type() == TT_FREETRADE)
            continue;
        if (tr.type() == TT_NEUTRALITY)
            continue;
        t += 1;
    }
    int32 planets = TotalPlanets(e);
    t = t + (10 - (int32)e->insurgency) * planets + e->money.get() / 100000 + 7 * e->agents +
        e->soldiers + e->fighters + 3 * e->heavycruisers + 3 * e->lightcruisers +
        (e->stations << 1) + 3 * e->carriers + (e->generals << 1) +
        e->population / ((int32)e->taxrate + 10) + (int32)e->cmdship + e->bank.get() / 110000 -
        e->pollution + e->tradegoods / 2;
    if (t < 0)
        t = 0;
    return t;
}

// 1c59:072c
int32 EmpireScore(Empire *e)
{
    return e->score;
}

// 1c59:0740
const char *InsurgencyName(Empire *e)
{
    switch (e->insurgency) {
    case 0: return "Peaceful";
    case 1: return "Mild Insurgencies";
    case 2: return "Occasional Riots";
    case 3: return "Violent Demonstrations";
    case 4: return "Political Conflicts";
    case 5: return "Internal Violence";
    case 6: return "Revolutionary Warfare";
    case 7: return "Under Coup";
    }
    return "(unknown)";
}

// 1c59:07a8
const char *PlanetName(int8 type)
{
    if (type >= 0 && type < 10)
        type = PLANET_CODES[(int)type];
    switch (type) {
    case 'P': return "Petroleum";
    case 'E': return "Education";
    case 'T': return "Tourism";
    case 'F': return "Food";
    case 'R': return "Research";
    case 'O': return "Ore";
    case 'G': return "Government";
    case 'U': return "Urban";
    case 'S': return "Supply";
    case 'A': return "Antipollution";
    }
    return "(unknown planet)";
}

// ---------------------------------------------------------------------
// 48cc:11be: clear what went short in the last turn (Empire+0x1c7).
void ClearShortages(Empire *e)
{
    memset(e->neglected, 0, 0x60);
}

static int8 Clamp100(int16 v)
{
    if (v < 0)
        v = 0;
    if (v > 100)
        v = 100;
    return (int8)v;
}

static void SetPlanets(Empire *e, int16 type, int32 count, int32 target, int32 output)
{
    PlanetSet *p = Planets(e, type);
    p->count = 0;
    p->target = 0;
    p->output = 0;
    Planets(e, type)->target = target;
    Planets(e, type)->output = output;
    Planets(e, type)->count = count;
}

// 48cc:0254
void EmpireInit(Empire *e)
{
    e->id = -1;
    e->name[0] = 0;
    e->player = -1;
    e->lastplay = g_now;
    e->created = g_now;
    e->inactivedays = 4;
    SetPlanets(e, P_PETROLEUM, 0, 1900, 1900);
    SetPlanets(e, P_EDUCATION, 3, 17, 39);
    SetPlanets(e, P_TOURISM, 9, 2650, 3100);
    SetPlanets(e, P_FOOD, 1, 46, 53);
    SetPlanets(e, P_RESEARCH, 1, 9, 11);
    SetPlanets(e, P_ORE, 9, 2200, 2500);
    SetPlanets(e, P_GOVERNMENT, 1, 2, 3);
    SetPlanets(e, P_URBAN, 3, 200, 210);
    SetPlanets(e, P_SUPPLY, 0, 11, 15);
    SetPlanets(e, P_ANTIPOLLUTION, 0, 10, 10);
    for (int16 i = 0; i < 26; i++) {
        Treaty &t = e->treaty[i];
        t.settype(TT_NONE);
        t.days = 0;
        t.setsoldierpct(0);
        t.setcruiserpct(0);
        t.setstatus(TS_NONE);
    }
    for (int16 i = 0; i < 25; i++)
        e->intercepts[i] = 0;
    e->lightcruisers = 0;
    e->generals = 0;
    e->carriers = 0;
    e->heavycruisers = 0;
    e->stations = 0;
    e->fighters = 0;
    e->soldiers = 0;
    e->supplyrates[0] = Clamp100(20);
    e->supplyrates[1] = Clamp100(20);
    e->supplyrates[2] = Clamp100(20);
    e->supplyrates[3] = Clamp100(20);
    e->supplyrates[4] = Clamp100(10);
    e->supplyrates[5] = Clamp100(10);
    e->tradegoods = 0;
    e->tickets = 0;
    e->localplay = 0;
    e->insurgency = 0;
    e->effectiveness = Clamp100(100);
    e->cmdship = Clamp100(0);
    e->money.set(10000);
    e->score = 0;
    e->points = 0;
    e->population = 40;
    e->pollution = 0;
    e->food = 0;
    e->efficiency = 0;
    e->d29d = 0;
    if (g_open) {
        e->year = -g_galaxy.protection;
        e->turnsused = 0;
        e->turnsperday = g_galaxy.grantturns;
    } else {
        e->year = -20;
        e->turnsused = 0;
        e->turnsperday = 5;
    }
    e->bonusyear = e->year;
    e->bank.set(0);
    e->loanpayment = 0;
    e->w1a8 = 0;
    e->bonds = 0;
    e->maxbonds = 0;
    e->bondpayoff = 0;
    e->bondprice = 0;
    e->bondhours = 0;
    e->bondturns = 0;
    e->bondname[0] = 0;
    e->taxrate = Clamp100(23);
    e->w1ab = 0;
    e->draftrate = Clamp100(5);
    e->w1ae = 0;
    e->favor = 0;
    e->phase = PH_START;
    e->income.set(0);
    e->shortfalls &= ~(SF_NEGLECT | SF_STARVED | SF_UNUSED | SF_ARMY);
    ClearShortages(e);
    e->effnext = 0;
    e->effchange = 0;
    e->weapons &= ~(W_NUCLEAR | W_CHEMICAL | W_PSIONIC | W_MOTHERSHIP);
    e->weapons &= 0x0f;
    e->troubles = 0;
    e->bonuses = 0;
    e->attacks = 0;
    e->agents = 0;
    e->aidmask = 0;
    e->dissensionmask = 0;
    e->demoralizemask = 0;
    e->bombmask = 0;
    e->hostagemask = 0;
    e->bribemask = 0;
    e->setupmask = 0;
    for (int16 i = 0; i < 9; i++)
        e->w26f[i] = 0;
    e->researchpoints = 0;
    e->planetsbought = 0;
    e->lastnews = 0;
}

// 48cc:112c
void EmpireInitId(Empire *e, int16 id)
{
    EmpireInit(e);
    e->id = id;
}

// 48cc:1146
void EmpireSetName(Empire *e, const String &s)
{
    strncpy(e->name, s.c_str(), 0x1f);
    e->name[0x1f] = 0;
}

// 48cc:1196
int32 OilPrice()
{
    if (!g_open)
        return 1900;
    GalaxyOilPrice(&g_galaxy);
    return g_galaxy.oilprice;
}

// 48cc:124f: the empire of a player, or -1.
int16 EmpireOfPlayer(int16 player)
{
    if (player == -1)
        return -1;
    GalaxyOpen();
    for (int16 i = 0; i < MAXEMPIRES; i++) {
        Empire *e = &g_empires[i];
        if (Exists(*e) && e->player == player)
            return i;
    }
    return -1;
}

// 48cc:12b9: continue on a new, indented line if past column n.
void WrapAt(int16 n)
{
    if (kout.line.len > n)
        kout << "\n" << setw(12) << "" << setw(0);
}

// 48cc:12fc
void StatusItem(int32 v, const char *name, int16 flag)
{
    if (v <= 0)
        return;
    WrapAt((int16)(0x48 - strlen(name) - flag * 8));
    kout << Color(8) << '[' << c_normal << name;
    if (!flag)
        kout << (v == 1 ? "=" : "s=") << c_number << v << c_normal;
    kout << Color(8) << "] " << c_normal;
}

// 48cc:1400
static void PlanetItem(int16 type, PlanetSet *p)
{
    if (p->count <= 0)
        return;
    WrapAt((int16)(0x46 - strlen(PlanetName((int8)type))));
    kout << Color(8) << '[' << c_normal << PlanetName((int8)type) << '=' << c_number << p->count
         << c_normal << Color(8) << "] " << c_normal;
}

// 48cc:14ec
void EmpireStatus(Empire *e, int16 full)
{
    int16 sb = g_sbcolor;
    kout << Color(14) << "\nEmpire Status\n";
    HLine(Color(1), 0x4b);
    kout << Color(sb) << "-" << Color(sb + 8) << "*" << Color(15) << e->name << Color(sb + 8) << "*"
         << Color(sb) << "-" << c_normal << "\n";
    if (full) {
        kout << setw(12) << "Score:" << setw(0) << c_number << e->score << c_normal << '\n';
        kout << setw(12) << "Turns Left:" << setw(0) << c_number
             << (int16)(e->turnsperday - e->turnsused - 1) << c_normal << '\n';
    }
    kout << setw(12) << "Money:" << setw(0) << c_number << e->money.get() << c_normal;
    if (full) {
        if (e->bank.get() > 0)
            kout << Color(8) << "   (" << c_normal << "Bank Savings=" << c_number << e->bank.get()
                 << Color(8) << ')' << c_normal;
        else if (e->bank.get() < 0)
            kout << Color(8) << "   (" << c_normal << "Bank Loan=" << c_number << -e->bank.get()
                 << Color(8) << ')' << c_normal;
    }
    kout << '\n';
    kout << setw(12) << "Pop:" << setw(0) << c_number << e->population << c_normal << " Million   "
         << Color(8) << '(' << c_normal << "Tax Rate=" << (int16)e->taxrate << '%' << Color(8)
         << ")\n" << c_normal;
    kout << setw(12) << "Food:" << setw(0) << c_number << e->food << c_normal << " Megatons\n";
    if (full)
        kout << setw(12) << "Covert:" << setw(0) << c_number << e->agents << c_normal << " Agents\n";
    kout << setw(12) << "Insurgency:" << setw(0) << c_number << InsurgencyName(e) << c_normal << '\n';
    kout << setw(12) << "Cmd. Ship:" << setw(0) << c_number << (int16)e->cmdship << c_normal
         << "% completed\n";
    kout << setw(12) << "Military:" << setw(0);
    StatusItem(e->soldiers, "Soldier", 0);
    StatusItem(e->fighters, "Fighter", 0);
    StatusItem(e->stations, "Defense Station", 0);
    StatusItem(e->generals, "General", 0);
    StatusItem(e->heavycruisers, "Heavy Cruiser", 0);
    StatusItem(e->lightcruisers, "Light Cruiser", 0);
    StatusItem(e->carriers, "Carrier", 0);
    StatusItem(HasWeapon(e, W_MOTHERSHIP), "Mothership", 0);
    StatusItem(HasWeapon(e, W_NUCLEAR), "Nuclear Bomb", 1);
    StatusItem(HasWeapon(e, W_CHEMICAL), "Chemical Bomb", 1);
    StatusItem(HasWeapon(e, W_PSIONIC), "Psionic Bomb", 1);
    WrapAt(0x34);
    kout << Color(8) << '[' << c_normal << "Effectiveness=" << c_number << (int16)e->effectiveness
         << c_normal << "%" << Color(8) << "] " << c_normal;
    WrapAt(0x38);
    int32 nw = NetWorth(e);
    kout << Color(8) << '(' << c_normal << "Net Worth=" << Color(15) << nw << c_normal << Color(8)
         << ")\n" << c_normal;
    kout << setw(12) << "Planets:" << setw(0);
    for (int16 i = 0; i < 10; i++) {
        PlanetSet p = *Planets(e, i);
        PlanetItem(i, &p);
    }
    WrapAt(0x32);
    int32 tot = TotalPlanets(e);
    kout << Color(8) << '(' << c_normal << "Total=" << Color(15) << tot << c_normal << Color(8)
         << ")\n" << c_normal;
    if (e->troubles) {
        kout << setw(12) << "Troubles:" << setw(0);
        if (e->troubles & TR_PLAGUE)
            StatusItem(1, "Plague", 1);
        if (e->troubles & TR_FAMINE)
            StatusItem(1, "Famine", 1);
        if (e->troubles & TR_SPACESTORMS)
            StatusItem(1, "Space Storms", 1);
        if (e->troubles & TR_DUSTCLOUD)
            StatusItem(1, "Dust Cloud", 1);
        if (e->troubles & TR_GASCLOUD)
            StatusItem(1, "Gas Cloud", 1);
        if (e->troubles & TR_BANKFAILURE)
            StatusItem(1, "Bank Failure", 1);
        kout << '\n';
    }
    if (e->bonuses) {
        kout << setw(12) << "Bonuses:" << setw(0);
        StatusItem(e->bonuses & B_CLOAKING, "Cloaking Device", 1);
        StatusItem(e->bonuses & B_SCANNER, "Long Range Scanner", 1);
        StatusItem(e->bonuses & B_GENEFOOD, "Genetically Engineered Food", 1);
        StatusItem(e->bonuses & B_SHIELD, "Planetary Shield", 1);
        StatusItem(e->bonuses & B_ACCURA, "Accura Weapons", 1);
        StatusItem(e->bonuses & B_SPYING, "Enhanced Spying", 1);
        kout << '\n';
    }
    if (full) {
        if (e->year < -1) {
            kout << "You have " << c_number << (int16)-e->year << c_normal << " turn";
            if (e->year != -1)
                kout << 's';
            kout << " of protection left.  ";
        } else if (e->year == -1) {
            kout << "You are in the " << Color(15) << "LAST" << c_normal << " turn of protection!  ";
        } else {
            kout << "It is year " << c_number << e->year << c_normal << " after the end of protection.  ";
        }
        kout << "You have " << c_number << (int16)(e->turnsperday - e->turnsused - 1) << c_normal
             << " turns left today.\n";
    }
    HLine(Color(1), 0x4b);
}

// 48cc:232d: the Coordinator's fund pays a bonus to protected empires.
int32 ProtectionBonus(Empire *e)
{
    if (e->year < e->bonusyear)
        return 0;
    int32 div = 100;
    if (e->year <= 0)
        div = 300;
    if (e->year < -25)
        div = 400;
    int32 v = g_galaxy.coordfund / div;
    if (v < 0)
        v = 0;
    e->money.add(1000 * v);
    g_galaxy.coordfund -= v;
    e->bonusyear = (int16)(e->year + g_galaxy.grantturns);
    return v;
}

// 48cc:240a
void ShowStatus(Empire *e)
{
    EmpireStatus(e, 1);
}

// 48cc:2420
void NextPhase(Empire *e)
{
    if (e->phase < PH_CONTINUE) {
        e->phase++;
    } else {
        e->phase = PH_START;
        e->turnsused++;
        e->year++;
    }
}

// 48cc:2454
void PrevPhase(Empire *e)
{
    if (e->phase >= 0) {
        e->phase--;
    } else {
        e->phase = PH_CONTINUE;
        e->turnsused--;
        e->year--;
    }
}

// 48cc:2488: the command ship grows by itself once started.
void BuildCmdShip(Empire *e)
{
    if (e->cmdship > 0) {
        int8 step;
        if (e->cmdship < 90)
            step = 5;
        else if (e->cmdship < 95)
            step = 3;
        else
            step = 1;
        e->cmdship = Clamp100((int16)(e->cmdship + step));
    }
    if (e->cmdship > 100)
        e->cmdship = Clamp100(100);
    if (e->cmdship < 0)
        e->cmdship = Clamp100(0);
}

// 48cc:2598: bank interest on loans and savings.
void BankInterest(Empire *e)
{
    int32 b = e->bank.get();
    if (b < 0 && b >= -500000000) {
        int32 v = (int32)g_galaxy.interest * (b / 100);
        e->bank.add(v);
    }
    b = e->bank.get();
    if (b > 0 && b <= 1000000000) {
        int32 v = (int32)g_galaxy.interest * (b / 500);
        e->bank.add(v);
    }
}

// 14f6:02a6: move b a step towards a.
void MoveToward(PlanetSet *p)
{
    int32 diff = (p->target >= 0) ? p->target - p->output : 0;
    if (diff >= 30 || diff <= -30) {
        p->output += diff / 30;
    } else if (p->target > p->output) {
        p->output += 1;
    } else if (p->target < p->output) {
        p->output -= 1;
    }
}

// 48cc:267c
void PlanetsMoveToward(Empire *e)
{
    for (int16 i = 0; i < 10; i++)
        MoveToward(Planets(e, i));
}

// 48cc:26b0
void UpdateScore(Empire *e)
{
    e->score -= e->score / 19;
    e->score += TotalPlanets(e) / 6;
    e->score += e->population / ((int32)e->taxrate + 10);
}

// 48cc:2731: the Coordinator's favor fades, and grows with anti-pollution planets.
void DriftFavor(Empire *e)
{
    if (e->favor < 0)
        e->favor += 1;
    if (e->favor > 0)
        e->favor -= 2;
    if ((TotalPlanets(e) - e->planets[P_PETROLEUM].count) / 10 < e->planets[P_ANTIPOLLUTION].count)
        e->favor += 1;
}

// 48cc:27c6: spare capacity of government planets, in agents.
int32 GovCapacity(Empire *e)
{
    return 300 * e->planets[P_GOVERNMENT].count - 6 * e->generals - e->agents;
}

// 48cc:282f: spare capacity of government planets, in generals.
int32 GovCapacityGenerals(Empire *e)
{
    return (300 * e->planets[P_GOVERNMENT].count - 6 * e->generals - e->agents) / 6;
}

// Set a treaty's type and its default terms (inlined in the original).
void SetTreaty(Treaty &t, int16 type, uint8 days)
{
    t.settype(type);
    t.days = days;
    t.setsoldierpct(0);
    t.setcruiserpct(0);
    if (t.type() == TT_MINORALLIANCE) {
        t.setsoldierpct(0x14);
        t.setcruiserpct(0x14);
    }
    if (t.type() == TT_ARMEDDEFENSE)
        t.setsoldierpct(0x3c);
    if (t.type() == TT_CRUISERPROTECTION)
        t.setcruiserpct(0x3c);
    if (t.type() == TT_TOTALDEFENSE) {
        t.setsoldierpct(0x28);
        t.setcruiserpct(0x28);
    }
}
