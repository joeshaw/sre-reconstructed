// Government spending: prices and the buy/sell menu (segment 1af6).
#include "sre.h"

// Prices.  The galaxy's inflation flag selects the formulas that grow with
// the empire's year and forces instead of only its net worth.

// 1af6:0006
int32 PriceSoldiers(Empire *e)
{
    if (g_galaxy.inflation == 0)
        return NetWorth(e) / 1000 + 180;
    return (int32)(int16)((e->year << 1) + 180) + e->soldiers / 1800 + e->fighters / 12400 +
           NetWorth(e) / 5000;
}

// 1af6:00b8
int32 PriceFighters(Empire *e)
{
    if (g_galaxy.inflation == 0)
        return NetWorth(e) / 800 + 240;
    return (int32)(int16)((int16)(e->year * 5) / 2 + 240) + e->soldiers / 7000 +
           e->fighters / 1690 + NetWorth(e) / 400;
}

// 1af6:0172
int32 PriceStations(Empire *e)
{
    if (g_galaxy.inflation == 0)
        return NetWorth(e) / 700 + 320;
    return (int32)(int16)(e->year * 3 + 320) + e->soldiers / 7500 + e->fighters / 9000 +
           e->stations / 1460 + NetWorth(e) / 300;
}

// 1af6:024c
int32 PriceGenerals(Empire *e)
{
    if (g_galaxy.inflation == 0)
        return NetWorth(e) / 300 + 670;
    return 14 * (int32)e->year + e->soldiers / 1790 + e->generals / 170 +
           TotalPlanets(e) / 10 + 670;
}

// 1af6:0307
int32 PriceCmdShip(Empire *e)
{
    if (g_galaxy.inflation == 0)
        return NetWorth(e) / 100 + 8800;
    return e->soldiers / 850 + e->fighters / 950 + 5000;
}

// 1af6:0379
int32 PricePlanet(Empire *e)
{
    if (g_galaxy.inflation == 0)
        return NetWorth(e) / 100 + TotalPlanets(e) + 13000;
    int32 k;
    if (e->year < 0)
        k = 3;
    else if (TotalPlanets(e) >= 300)
        k = 1;
    else
        k = 2;
    int32 x = TotalPlanets(e) + e->score / 2 + NetWorth(e) / 10 + (e->year < 0 ? 0 : e->year);
    return (97 * x + 50000) / k / 10;
}

// 1af6:0495
int32 PriceAgents(Empire *e)
{
    if (g_galaxy.inflation == 0)
        return NetWorth(e) / 200 + 4300;
    return e->agents + 18 * (int32)e->year + 7 * TotalPlanets(e) + e->score / 10 + 2218;
}

// 1af6:0542
int32 PriceHeavyCruisers(Empire *e)
{
    if (g_galaxy.inflation == 0)
        return NetWorth(e) / 600 + 1100;
    return 7 * (int32)e->year + e->fighters / 1220 + TotalPlanets(e) / 5 + e->stations / 2500 +
           NetWorth(e) / 150 + 1100;
}

// 1af6:0624
int32 PriceCarriers(Empire *e)
{
    if (g_galaxy.inflation == 0)
        return NetWorth(e) / 700 + 850;
    return 7 * (int32)e->year + TotalPlanets(e) / 10 + NetWorth(e) / 100 + 850;
}

// 61a7:010a: buy up to max (-1: as many as affordable) of an item.
static void Buy(Money *m, int32 price, int32 *have, String name, int32 max)
{
    if (price > 0 && (m->get() / price < max || max < 0))
        max = m->get() / price;
    if (m->get() < price || m->get() <= 0) {
        kout << "You can not afford any " << Color(10) << name << c_normal << "!\n";
        return;
    }
    if (price <= 0) {
        kout << "No " << c_number << name << c_normal << " are available to buy this year.\n";
        return;
    }
    kout << c_normal << "Buy how many " << Color(10) << name << c_normal << "?  ";
    PutMax(max);
    int32 n;
    kin >> defval(0) >> maxval(max) >> n;
    *have += n;
    m->sub(n * price);
}

// 61a7:0384
static void CmdShip(Empire *e, int32 price)
{
    if (e->cmdship >= 100) {
        kout << "You have already completed work on your command ship!\n";
        return;
    }
    if (e->money.get() < price) {
        kout << "You can't afford to start your command ship now.\n";
        return;
    }
    int16 v;
    if (e->cmdship > 0) {
        kout << "You added some strength to your command ship.\n";
        v = (int16)(e->cmdship + 5);
    } else {
        kout << "You started work on your command ship.  Your workers will automatically ";
        kout << "add 5% to the strength of the command ship each year.\n";
        v = 5;
    }
    e->cmdship = (int8)v;
    if (e->cmdship < 0)
        e->cmdship = 0;
    if (e->cmdship > 100)
        e->cmdship = 100;
    e->money.sub(price);
}

// 61a7:04d2
static void Sell(Money *m, int32 price, int32 *have, String name)
{
    if (*have <= 0) {
        kout << "You don't own any " << Color(10) << name << c_normal << "!\n";
        return;
    }
    if (price <= 0)
        price = 1;
    kout << Color(14) << "Sell" << c_normal << " how many " << Color(10) << name << c_normal
         << "?  ";
    PutMax(*have);
    int32 n;
    kin >> defval(0) >> maxval(*have) >> n;
    if ((2000000000 - m->get()) / price < n)
        n = (2000000000 - m->get()) / price;
    *have -= n;
    m->add(n * price);
}

// 61a7:0000: a menu of the planet types with the empire's counts.
static int16 PlanetMenu(Menu *m, int16 show, Empire *e)
{
    const char *letters = "FURPOSTAGE";
    const char *ids = "3740582961";
    int32 count[10];
    for (int16 i = 0; i < 10; i++)
        count[i] = Planets(e, i)->count;
    for (int16 i = 0; i < 10; i++) {
        int16 id = (int16)(ids[i] - '0');
        m->Add(PlanetName(letters[i]), letters[i], id);
        m->Find(id)->AddColumn(count[id]);
    }
    return m->Run(show, -1);
}

// 61a7:083d
static int32 HowMany(int32 max)
{
    kout << c_normal << "How many planets?  ";
    PutMax(max);
    int32 n;
    kin >> defval(0) >> maxval(max) >> n;
    return n;
}

// 61a7:0902
struct ReleasePrompt : MenuHook {
    int16 Call()
    {
        kout << "You receive NO money from releasing planets.\n";
        return 1;
    }
};

// 61a7:091b
static void ReleasePlanets(Player *p, Empire *e, Galaxy *g)
{
    int16 show = 1;
    while (TotalPlanets(e) > 0) {
        if (!(HasPref(p->prefs1, PF1_SHORTMENUS)))
            show = 1;
        Menu m("SellPlanets", "Release Planets", 7, 8, 1, 9, 7, 3, 7, 7, 7, 7);
        m.prompt = new ReleasePrompt;
        int16 k = PlanetMenu(&m, show, e);
        if (k == -1) {
            kout << '\n';
            return;
        }
        int32 n = HowMany(Planets(e, k)->count);
        e->planetsbought -= (int16)n;
        Planets(e, k)->count -= n;
        g->marketplanets += n;
        show = 0;
    }
}

// 61a7:071e
static int32 BuyPlanets(Money *m, int32 price, int32 *max, int32 *score)
{
    kout << c_normal << "How many planets?  ";
    PutMax(*max);
    int32 n;
    kin >> defval(0) >> maxval(*max) >> n;
    *max -= n;
    *score += 3 * n;
    m->sub(n * price);
    return n;
}

// 61a7:0aab
struct BuyPlanetsPrompt : MenuHook {
    Empire *e;
    explicit BuyPlanetsPrompt(Empire *x) : e(x) {}
    int16 Call()
    {
        kout << c_normal << "Current earnings:  [Ore = ";
        kout << c_number << e->planets[P_ORE].output << c_normal << "] [Tourism = ";
        kout << c_number << e->planets[P_TOURISM].output << c_normal << "] [Petroleum = ";
        int32 oil = OilPrice();
        kout << c_number << oil << c_normal << "].\n";
        return 1;
    }
};

// 61a7:0b7f
static void Colonize(Player *p, Empire *e, Galaxy *g, int32 price)
{
    int32 avail = g->marketplanets;
    if (e->year < 0 && 300 - TotalPlanets(e) > avail)
        avail = 300 - TotalPlanets(e);
    if (avail + TotalPlanets(e) > 30000)
        avail = 30000 - TotalPlanets(e);
    if (avail <= 0) {
        kout << "There are no planets available to be colonized.\n";
        return;
    }
    int32 max = e->money.get() / price;
    if (max > avail)
        max = avail;
    kout << "There are " << c_number << avail << c_normal << " planets available.\n";
    if (g_galaxy.planetsperturn >= 0 && g_galaxy.planetsperturn - e->planetsbought < max) {
        max = g_galaxy.planetsperturn - e->planetsbought;
        kout << "You may buy " << c_number << g_galaxy.planetsperturn << c_normal << " planets per turn.\n";
    }
    int16 show = 1;
    while (max > 0) {
        if (!(HasPref(p->prefs1, PF1_SHORTMENUS)))
            show = 1;
        Menu m("BuyPlanets", "Buy Planets", 7, 1, 1, 9, 0xf, 0xb, 7, 7, 7, 7);
        m.prompt = new BuyPlanetsPrompt(e);
        int16 k = PlanetMenu(&m, show, e);
        kout << c_normal;
        if (k == -1)
            return;
        show = 0;
        if (!(p->seenplanets & (1 << k))) {
            String s("Planet");
            s += (char)k;
            ShowScreen(0, s, 0);
            p->seenplanets |= (int16)(1 << k);
            ShowHelpTopic("planet", PlanetName((int8)k));
        }
        if (k == P_ANTIPOLLUTION && e->pollution > 0)
            kout << "There are " << c_number << e->pollution << c_normal
                 << " exatons of pollution in the atmosphere.\n";
        if (k == P_FOOD)
            kout << "Each food planet can produce " << c_number << e->planets[P_FOOD].output << c_normal
                 << " megatons of food each year.\n";
        if (k == P_SUPPLY)
            kout << "Each supply planet can train " << c_number << e->planets[P_SUPPLY].output << c_normal
                 << " military units per year.  Soldiers count as 1 unit, fighters 1.3, defense stations 1.7, generals 7, carriers 9.5, and cruisers 12.\n";
        int32 n = BuyPlanets(&e->money, price, &max, &e->score);
        e->planetsbought += (int16)n;
        Planets(e, k)->count += n;
        g->marketplanets -= n;
        if (g->marketplanets < 0)
            g->marketplanets = 0;
    }
}

// 1af6:06ba
static void SpendingHeader(Money *m, int16 turns)
{
    kout << c_normal << "You have " << c_number << m->get() << c_normal << " credits, and "
         << c_number << turns << c_normal << " turns.\n";
}

// 1af6:073d
struct BuyPrompt : MenuHook {
    Empire *e;
    explicit BuyPrompt(Empire *x) : e(x) {}
    int16 Call()
    {
        SpendingHeader(&e->money, (int16)(e->turnsperday - e->turnsused - 1));
        return 1;
    }
};

// 1af6:076a
struct SellPrompt : MenuHook {
    Empire *e;
    explicit SellPrompt(Empire *x) : e(x) {}
    int16 Call()
    {
        SpendingHeader(&e->money, (int16)(e->turnsperday - e->turnsused - 1));
        kout << Color(14) << "SELL" << c_normal << " which item?  ";
        return 0;
    }
};

static Resource r_military("Spending.Military", (int32)1);     // 9b1e

// 1af6:07dd
void Spending(Player *p, Empire *e, Galaxy *g)
{
    String cmdtext;
    int32 price[10], sellprice[10], owned[10];
    const char *names[10];
    price[1] = PriceSoldiers(e);      names[1] = "Soldiers";
    price[2] = PriceFighters(e);      names[2] = "Fighters";
    price[3] = PriceStations(e);      names[3] = "Defense Stations";
    price[4] = PriceGenerals(e);      names[4] = "Generals";
    price[5] = PriceCmdShip(e);       names[5] = "Command Ship";
    price[6] = PricePlanet(e);        names[6] = "Colonize Planets";
    price[7] = PriceAgents(e);        names[7] = "Covert Agents";
    price[8] = PriceHeavyCruisers(e); names[8] = "Heavy Cruisers";
    price[9] = PriceCarriers(e);      names[9] = "Carriers";
    for (int16 i = 1; i <= 9; i++)
        sellprice[i] = price[i] / 3;
    sellprice[6] = 0;           // released planets bring nothing
    int16 selling = 0;
    int16 show = 1;
    for (;;) {
        TimeCheck();
        Menu buy("Buying", "Government Spending", 0xb, 9, 2, 0xe, 0xa, 0xb, 0xf, 7, 7, 7);
        Menu sell("Selling", "Selling", 0xe, 4, 8, 0xf, 7, 0xc, 0xf, 7, 7, 7);
        buy.prompt = new BuyPrompt(e);
        sell.prompt = new SellPrompt(e);
        buy.Add("Item", 0, 100);
        MenuItem *head = buy.Find(100);
        for (int16 i = 0; i < 8; i++)
            head->colors[i] = 7;
        head->LoadColors();
        buy.Find(100)->AddColumn("Price");
        buy.Find(100)->AddColumn("Owned");
        int16 mil = r_military.Bool();
        for (int16 i = 1; i <= 9; i++) {
            char key = 0;
            if (mil || i == 5 || i == 6 || i == 7)
                key = (char)('0' + i);
            buy.Add(names[i], key, i);
            if (i == 5)
                continue;
            if (i == 6)
                sell.Add("Release Planets", '6', 6);
            else
                sell.Add(names[i], (char)('0' + i), i);
        }
        buy.Add("Sell", 'S', 10);
        buy.Add("Operations", '*', 11);
        sell.Add("Buy", 'B', 10);
        sell.Add("Operations", '*', 11);
        if (g_system.regstate) {
            buy.Add("", 'V', 0x50);
            sell.Add("", 'V', 0x50);
        }
        if (HasPref(p->prefs2, PF2_ENTERSTAYS))
            buy.Add("Quit", 'Q', 0x63);
        int32 *field[10] = { 0,          &e->soldiers, &e->fighters,      &e->stations, &e->generals,
                             0,          0,            &e->agents,        &e->heavycruisers,
                             &e->carriers };
        owned[1] = e->soldiers;
        owned[2] = e->fighters;
        owned[3] = e->stations;
        owned[4] = e->generals;
        owned[5] = e->cmdship;
        owned[6] = TotalPlanets(e);
        owned[7] = e->agents;
        owned[8] = e->heavycruisers;
        owned[9] = e->carriers;
        for (int16 i = 1; i <= 9; i++) {
            MenuItem *b = buy.Find(i);
            MenuItem *s = sell.Find(i);
            if (b) {
                if (b->key)
                    b->AddColumn(price[i]);
                else
                    b->AddColumn("");
            }
            if (i == 5) {
                cmdtext = IntStr(e->cmdship) + "%";
                if (b)
                    b->AddColumn(cmdtext.c_str());
                continue;
            }
            if (s)
                s->AddColumn(sellprice[i]);
            if (b)
                b->AddColumn(owned[i]);
            if (s)
                s->AddColumn(owned[i]);
        }
        if (!(HasPref(p->prefs1, PF1_SHORTMENUS)))
            show = 1;
        int16 r;
        if (selling) {
            r = sell.Run(show, -1);
            if (r >= 1 && r <= 9)
                r += 20;
            if (r == -1)
                r = 10;
        } else {
            r = buy.Run(show, -1);
        }
        if (r == -1 && !(HasPref(p->prefs2, PF2_ENTERSTAYS)))
            return;
        if (r == 0x63)
            return;
        show = 0;
        switch (r) {
        case 1: case 2: case 3: case 8: case 9:
            Buy(&e->money, price[r], field[r], names[r], -1);
            break;
        case 4: {
            int32 cap = GovCapacityGenerals(e);
            if (cap <= 0)
                kout << "One government planet is required for every 50 generals.  You don't have space on your planets for more generals.\n";
            else
                Buy(&e->money, price[4], &e->generals, names[4], cap);
            break;
        }
        case 5:
            CmdShip(e, price[5]);
            break;
        case 6:
            Colonize(p, e, g, price[6]);
            show = 1;
            break;
        case 7: {
            int32 cap = GovCapacity(e);
            if (cap <= 0)
                kout << "One government planet is required for every 300 agents.  You don't have space on your planets for more covert agents.\n";
            else
                Buy(&e->money, price[7], &e->agents, names[7], cap);
            break;
        }
        case 10:
            selling = !selling;
            show = 1;
            break;
        case 11:
            Operations(p, e, g);
            show = 1;
            break;
        case 21: case 22: case 23: case 24: case 27: case 28: case 29:
            Sell(&e->money, sellprice[r - 20], field[r - 20], names[r - 20]);
            break;
        case 26:
            ReleasePlanets(p, e, g);
            show = 1;
            break;
        case 0x50:
            Bank(p, e, g);
            show = 1;
            break;
        }
        if (!e->name[0] || TotalPlanets(e) <= 0 || e->population <= 0)
            return;
    }
}
