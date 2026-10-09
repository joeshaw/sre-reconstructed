// The operations menu, tax and draft rates, preferences and supply
// production (segment 6a9b).
#include "sre.h"

// 6a9b:06c1
static void TaxRates(Player *p, Empire *e)
{
    const char *tax = "The tax rate is ";
    int16 v;
    kout << "How much do want the tax rate to be (0%-100%)?  ";
    kin >> defval((int32)e->taxrate) >> maxval(100) >> v;
    e->taxrate = (int8)v;
    if (e->taxrate < 0)
        e->taxrate = 0;
    if (e->taxrate > 100)
        e->taxrate = 100;
    if (!((p->prefs1 >> 3) & 1))
        kout << tax << "now " << c_number << (int16)e->taxrate << c_normal << "%.\n";
    if (e->taxrate > 80 && Random(100) < e->taxrate)
        kout << "A high tax rate may cause violent revolts in your empire.\n";
    else if (e->taxrate > 50 && Random(90) < e->taxrate)
        kout << "A high tax rate will cause your citizens to leave the empire.\n";
    else if (e->taxrate > 30 && Random(50) < e->taxrate)
        kout << "The citizens are unhappy with the tax rate.\n";
    const char *draft = "The draft rate is ";
    kout << "How much do want the draft rate to be (0%-30%)?  ";
    kin >> defval((int32)e->draftrate) >> maxval(30) >> v;
    e->draftrate = (int8)v;
    if (e->draftrate < 0)
        e->draftrate = 0;
    if (e->draftrate > 100)
        e->draftrate = 100;
    if (!((p->prefs1 >> 3) & 1))
        kout << draft << "now " << c_number << (int16)e->draftrate << c_normal << "%.\n";
}

// 6a9b:09a8
void Preferences(Player *p)
{
    int16 r;
    do {
        Menu m("Preferences", "Player Preferences", 0xb, 3, 3, 0xf, 0xb, 7, 7, 7, 7, 7);
        m.Add((p->prefs1 >> 2) & 1 ? "Redisplay Menus" : "Don't redisplay Menus", '1', 1);
        m.Add("Automatic Payment", '2', 2);
        m.Add("Galactic Coordinator Donations", '3', 3);
        if (g_system.regstate)
            m.Add("Bank Deposits", '4', 4);
        m.Add("Report Dates", '6', 6);
        m.Add("Exiting the Spending Menu", '7', 7);
        m.Add("ANSI Color", '8', 8);
        m.Add("IBM Character Set", '9', 9);
        r = m.Run(1, -1);
        int16 ask, yes;
        switch (r) {
        case 1:
            kout << "Spending and Planet menus are now " << c_number;
            p->prefs1 ^= 4;
            if ((p->prefs1 >> 2) & 1)
                kout << "SHORT";
            else
                kout << "LONG";
            kout << c_normal << ".\n";
            break;
        case 2:
            kout << "SRE can pay your expenses automatically if you wish.  ";
            ask = YesNo(p->prefs2 & 1, "Do you want the system to ask about automatic payment");
            if (ask)
                yes = YesNo((p->prefs2 >> 1) & 1, "Do you want the default to be YES");
            else
                yes = YesNo((p->prefs2 >> 1) & 1,
                            "Do you want it to automatically pay whenever possible");
            p->prefs2 = (uint8)((p->prefs2 & ~1) | (ask & 1));
            p->prefs2 = (uint8)((p->prefs2 & ~2) | ((yes & 1) << 1));
            break;
        case 3:
            kout << "The Galactic Coordinator appreciates donations.  At the end of each turn, you have the option of donating a portion of your funds to It.  ";
            if (YesNo(!((p->prefs1 >> 6) & 1) && ((p->prefs1 >> 7) & 1),
                      "Do you want to always donate")) {
                ask = 0;
                yes = 1;
            } else if (YesNo(!((p->prefs1 >> 6) & 1) && !((p->prefs1 >> 7) & 1),
                             "Do you want to never donate")) {
                ask = 0;
                yes = 0;
            } else {
                ask = 1;
                yes = YesNo((p->prefs1 >> 7) & 1,
                            "The system will ask whether you want to donate.  Do you want the default to be YES");
            }
            p->prefs1 = (uint8)((p->prefs1 & ~0x40) | ((ask & 1) << 6));
            p->prefs1 = (uint8)((p->prefs1 & 0x7f) | ((yes & 1) << 7));
            break;
        case 4:
            kout << "At the end of each turn, you can deposit the rest of your money into the bank if you wish.  ";
            if (YesNo(!((p->prefs1 >> 4) & 1) && ((p->prefs1 >> 5) & 1),
                      "Do you want to always deposit")) {
                ask = 0;
                yes = 1;
            } else if (YesNo(!((p->prefs1 >> 4) & 1) && !((p->prefs1 >> 5) & 1),
                             "Do you want to never deposit")) {
                ask = 0;
                yes = 0;
            } else {
                ask = 1;
                yes = YesNo((p->prefs1 >> 5) & 1,
                            "The system will ask whether you want to deposit.  Do you want the default to be YES");
            }
            p->prefs1 = (uint8)((p->prefs1 & ~0x10) | ((ask & 1) << 4));
            p->prefs1 = (uint8)((p->prefs1 & ~0x20) | ((yes & 1) << 5));
            break;
        case 5:
            kout << "When you win a battle, SRE can attach a message to the other empire.  ";
            yes = YesNo((p->prefs2 >> 2) & 1, "Would you like to do this");
            p->prefs2 = (uint8)((p->prefs2 & ~4) | ((yes & 1) << 2));
            break;
        case 6:
            yes = YesNo((p->prefs2 >> 3) & 1, "Do you want the date to be displayed on reports");
            p->prefs2 = (uint8)((p->prefs2 & ~8) | ((yes & 1) << 3));
            break;
        case 7:
            yes = YesNo(!((p->prefs2 >> 4) & 1), "Do you want Enter to exit from the spending menu");
            p->prefs2 = (uint8)((p->prefs2 & ~0x10) | ((!yes & 1) << 4));
            break;
        case 8:
            g_ansi = YesNo(g_ansi, "Use ANSI color");
            break;
        case 9:
            g_ibm = YesNo(g_ibm, "Enable IBM (high ASCII) characters");
            break;
        }
    } while (r != -1);
}

// 6a9b:0f63: how supply planets divide their production.
static void SupplyProduction(Player *p, Empire *e)
{
    (void)p;
    // Menu id -> 1 + index into the empire's allocation (DS:927e).
    static const int16 SLOT[10] = { 0, 1, 2, 3, 6, 0, 0, 0, 4, 5 };
    Menu m("Production", "Supply Production", 0xb, 9, 3, 0xf, 0xb, 0xe, 7, 7, 7, 7);
    m.Add("Soldiers", '1', 1);
    m.Add("Fighters", '2', 2);
    m.Add("Defense Stations", '3', 3);
    m.Add("Generals", '4', 4);
    m.Add("Heavy Cruisers", '8', 8);
    m.Add("Carriers", '9', 9);
    for (int16 i = 1; i <= 9; i++) {
        if (!SLOT[i])
            continue;
        int16 v = e->b297[SLOT[i] - 1];
        m.Find(i)->AddColumn((int32)v);
        m.Find(i)->AddColumn("%");
    }
    m.Show();
    kout << c_normal << "\nEnter the percent of supply production dedicated to\n";
    uint16 left = 100;
    for (int16 i = 1; i <= 9; i++) {
        if (!SLOT[i])
            continue;
        uint16 def = (uint16)(int16)e->b297[SLOT[i] - 1];
        if (def > left)
            def = left;
        kout << c_normal << setw(0x14) << m.Find(i)->text;
        kout << setw(0) << c_bracket << '[' << c_bold << "MAX=" << left << c_normal << c_bracket
             << ']' << c_normal << "  ";
        int16 v;
        kin >> maxval((int32)left) >> defval((int32)def) >> v;
        left -= v;
        int8 &slot = e->b297[SLOT[i] - 1];
        slot = (int8)v;
        if (slot < 0)
            slot = 0;
        if (slot > 100)
            slot = 100;
    }
}

// 6a9b:0000
void Operations(Player *p, Empire *e, Galaxy *g)
{
    int16 r;
    do {
        TimeCheck();
        Menu m("Operations", "Operations Menu", 0xe, 4, 4, 0xc, 0xf, 7, 7, 7, 7, 7);
        m.columns = 2;
        if (g_system.regstate)
            m.Add("Visit the Bank", '2', 2);
        if (e->agents > 0)
            m.Add("Covert Operations", '3', 3);
        if (e->year >= 0 && r_trading.Bool())
            m.Add("Trading", '4', 4);
        m.Add("Display Status", '5', 5);
        if (r_messages.Bool())
            m.Add("Communications", '6', 6);
        if (g_system.regstate && r_lottery.Bool())
            m.Add("Buy Super Lottery Tickets", '7', 7);
        m.Add("See Scores", '8', 8);
        if (r_diplomacy.Bool())
            m.Add("Diplomacy", '9', 9);
        m.Add("Visit Advisors", 'A', 10);
        m.Add("Tax & Draft Rates", 'T', 11);
        if (e->planets[P_SUPPLY].count > 0)
            m.Add("Supply Planet Production", 'S', 12);
        m.Add("Preferences", 'P', 13);
        m.Add("Instructions File", 'I', 14);
        m.Add("More Help", 'H', 15);
        m.Add("Abdicate", '^', 16);
        r = m.Run(1, -1);
        switch (r) {
        case 2:
            Bank(p, e, g);
            break;
        case 3:
            CovertOps(p, e, g);
            break;
        case 4:
            Trade(p, e);
            break;
        case 5:
            ShowStatus(e);
            break;
        case 6:
            Communications(p, e);
            break;
        case 7:
            BuyTickets(p, e, g);
            break;
        case 8:
            ScoreListAll(g);
            break;
        case 9:
            TreatyMessages(p, e);
            DiplomacyMenu(p, e);
            break;
        case 10:
            Advisors(p, e);
            break;
        case 11:
            TaxRates(p, e);
            break;
        case 12:
            SupplyProduction(p, e);
            break;
        case 13:
            Preferences(p);
            break;
        case 14:
            DisplayFile("Text/Player.MSG", 0);
            break;
        case 15:
            MoreHelp();
            break;
        case 16: {
            if (!YesNo(0, "Are you sure you wish to abdicate"))
                break;
            kout << "You are about to lose your empire.  If you are sure you would like to quit (and possibly start over), type in your empire name as displayed:\n";
            kout << ">> " << c_empire << Untr(e->name) << c_normal << '\n';
            kout << ">> ";
            char buf[0x51];
            kin >> maxlen(0x50) >> buf >> maxlen(0);
            if (!StrEqI(String(e->name), String(buf))) {
                kout << "Aborted!\n";
                break;
            }
            kout << "Your reign over " << Untr(e->name) << " has ended!\n";
            DeleteEmpire(e);
            return;
        }
        }
    } while (r != -1);
}
