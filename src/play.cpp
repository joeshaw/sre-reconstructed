// Time limits (segment 1cf7) and the turn sequence (segment 1e7b).
#include "sre.h"
#include "events.h"
#include "battle.h"

// Turn phases implemented in other modules.
void TradeArrivals(Player *p, Empire *e);              // 6e3f:1f14
void TurnReport(Player *p, Empire *e);                 // 462a:0b58
void Advisor(Player *p, Empire *e);                    // 6572:2459
void PayUpkeep(Player *p, Empire *e, Galaxy *g);       // 5e48:0b9f
void FoodMarket(Player *p, Empire *e, Galaxy *g);      // 512f:018b
void CovertOps(Player *p, Empire *e, Galaxy *g);       // 409c:0a6d
void Bank(Player *p, Empire *e, Galaxy *g);            // 3e41:0000
void Spending(Player *p, Empire *e, Galaxy *g);        // 1af6:07dd
void Industry(Player *p, Empire *e);                   // 432f:006f
void Trade(Player *p, Empire *e);                      // 6e3f:2429
void SendMessage(int16 from, String name);             // 58b6:1ce9
void ReadMessages(Player *p, Empire *e);               // 58b6:1eb3
int16 Attack(Player *p, Empire *e, char how);          // 5090:0121
void EndOfTurn(Player *p, Empire *e, Galaxy *g);       // 5f38:07ae
void Population(Player *p, Empire *e);                 // 5f38:0000
void Upkeep(Player *p, Empire *e);                     // 44e2:07cc
void Growth(Player *p, Empire *e);                     // 44e2:0000
void TreatyMessages(Player *p, Empire *e);                  // 70d2:1ddf
void DiplomacyMenu(Player *p, Empire *e);               // 70d2:0000
void RegeneratePlanets(Galaxy *g);                     // 1c59:0855
extern Resource r_messages;                            // 62e8
extern Resource r_diplomacy;                           // b01e
extern Resource r_trading;                             // aa0a

static Resource r_strict("StrictTime", (int32)0);          // a712
static Resource r_saveoften("System.SaveOften", (int32)0); // b71e
static Resource r_lottery("Lottery", (int32)1);            // b72a

static int32 g_start = b_time(0);       // DS:d2d8 time the door started
static int16 g_warned;                  // DS:b738
static int16 g_local;                   // DS:b736 unlimited turns

// 1cf7:0cfa: minutes left in the door.
int32 MinutesLeft()
{
    int32 now = b_time(0);
    if (g_door.minutes >= 1000 || g_door.minutes < 0)
        return 999;
    return (int32)g_door.minutes - (now - g_start) / 60;
}

// 1cf7:0d69
static int16 TimeIsUp(DoorInfo *d)
{
    int32 now = b_time(0);
    if (d->minutes >= 1000)
        return 0;
    return now - g_start > 60 * (int32)d->minutes && d->minutes > -1;
}

// 1cf7:179e: planets regenerate while the player thinks.
static void IdlePlanets()
{
    static int32 last = b_time(0);      // DS:d2e4
    int32 now = b_time(0);
    if (now - last > 1) {
        last = now;
        if (g_open)
            RegeneratePlanets(&g_galaxy);
    }
}

// 1cf7:15b8: the keyboard idle function (the sysop's status bars are
// not reconstructed).
void IdleHook()
{
    IdlePlanets();
}

// 1e7b:000f
static void TimeUpCheck()
{
    if (!TimeIsUp(&g_door))
        return;
    kout << '\n';
    ShowScreen(0, "TimeUp", 0);
    kout << Color(14) << "Sorry, your time is up.  Please play again tomorrow!\n";
    exit(1);
}

// 1e7b:008d
void TimeCheck()
{
    TimeUpCheck();
    if (MinutesLeft() <= 1 && !g_warned) {
        kout << Color(14) << "Warning:  " << c_normal
             << "You have less than 1 minute remaining in this door!\n";
        g_warned = 1;
    }
}

// 1e7b:00fa
static void Collapsed(Empire *e)
{
    if (TotalPlanets(e) <= 0 && !ShowScreen(0, "Collapse", 0))
        kout << "Your empire has collapsed because you have no planets left.\n";
    if (e->population <= 0 && !ShowScreen(0, "Abandone", 0))
        kout << "Your people have abandoned you, and your empire has collapsed.\n";
}

static int16 Alive(Empire *e)
{
    return Exists(*e) && TotalPlanets(e) > 0 && e->population > 0;
}

static void Key(char c)
{
    kout << c_bracket << '[' << c_bold << c << c_bracket << ']' << c_normal;
}

// The start of a turn's phases (inlined in 1e7b:0731 and 1e7b:0f61).
static void StartTurn(Empire *e)
{
    e->phase = 0;
    e->m1bc.set(0);
    e->b1c4 &= ~1;
    e->b1c4 &= ~2;
    e->b1c4 &= ~4;
    e->b1c4 &= ~8;
    ClearSpending(e->u_1c7);
    e->b1c6 = 0;
    e->b1c5 = 0;
}

// Phase 14: attacks, trades and messages.
static void ActionPhase(Player *p, Empire *e)
{
    int16 cantrade = r_trading.Bool() && e->year >= 0;
    int16 canattack = 1;
    int16 canmessage = r_messages.Bool();
    do {
        int16 n = cantrade + canattack + canmessage;
        kout << "\nDo you wish to";
        char c = 'N';
        if (n == 1) {
            if (canattack)
                c = YesNo(0, " send an attack") ? 'A' : 'N';
            if (cantrade)
                c = YesNo(0, " trade") ? 'T' : 'N';
            if (canmessage)
                c = YesNo(0, " send a message") ? 'M' : 'N';
        } else {
            if (canattack) {
                kout << ' ';
                Key('A');
                kout << "ttack,";
            }
            if (cantrade) {
                kout << ' ';
                Key('T');
                kout << "rade,";
            }
            if (canmessage) {
                kout << " send a ";
                Key('M');
                kout << "essage,";
            }
            kout << " or ";
            Key('N');
            kout << "one? ";
            for (;;) {
                kin >> c;
                c = (char)b_toupper((int)(int8)c);
                if (c >= '1' && c <= '9' && canattack)
                    break;
                if (c == 'A' && canattack)
                    break;
                if (c == 'T' && cantrade)
                    break;
                if (c == 'M' && canmessage)
                    break;
                if (c == '\r' || c == 27 || c == '0')
                    c = 'N';
                if (c == 'N')
                    break;
            }
            kout << c_input;
            switch (c) {
            case '1': case '2': case '3': case '4': case '5': case '6': case '7': case '8':
            case '9': case 'A':
                kout << "Attack";
                break;
            case 'T': kout << "Trade"; break;
            case 'M': kout << "Message"; break;
            case 'N': kout << "None"; break;
            }
            kout << c_normal << '\n';
        }
        if (c == 'T')
            Trade(p, e);
        else if (c == 'M')
            SendMessage(e->id, String(e->name));
        else if (c != 'N')
            canattack = !Attack(p, e, c);
        if (c == 'N')
            break;
    } while (cantrade || canattack || canmessage);
}

// 1e7b:0731: one phase of a turn; 0 when the player is done.
static int16 Phase(Player *p, int16 phase, Empire *e, Galaxy *g)
{
    switch (phase) {
    case 0:
        StartTurn(e);
        break;
    case 1:
        e->w192 = 0;
        BuildCmdShip(e);
        BankInterest(e);
        break;
    case 2:
        TradeArrivals(p, e);
        ProcessEvents();
        PrevPhase(e);
        ShowNewReports(p, e);
        HLine(Color(8), 0x4b);
        break;
    case 3:
        TurnReport(p, e);
        break;
    case 4:
        PrevPhase(e);
        Advisor(p, e);
        Pause();
        break;
    case 5:
        PrevPhase(e);
        ShowStatus(e);
        break;
    case 6:
        PrevPhase(e);
        PayUpkeep(p, e, g);
        break;
    case 7:
        PrevPhase(e);
        FoodMarket(p, e, g);
        break;
    case 8:
        e->d22f[0] = 0;
        e->d22f[1] = 0;
        e->d22f[2] = 0;
        break;
    case 9:
        PrevPhase(e);
        CovertOps(p, e, g);
        break;
    case 10:
        PrevPhase(e);
        if (g_system.regstate)
            Bank(p, e, g);
        break;
    case 11:
        PrevPhase(e);
        Spending(p, e, g);
        break;
    case 12:
        Industry(p, e);
        break;
    case 13:
        break;
    case 14:
        ActionPhase(p, e);
        break;
    case 15:
        break;
    case 16:
        EndOfTurn(p, e, g);
        break;
    case 17:
        Population(p, e);
        break;
    case 18:
        UpdateScore(e);
        UpdateD1b0(e);
        break;
    case 19:
        Upkeep(p, e);
        Growth(p, e);
        break;
    case 20:
        PlanetsMoveToward(e);
        break;
    case 21: {
        int16 done = e->turnsused >= e->turnsperday && !g_local;
        if (done)
            Pause();
        if (!done && !YesNo(1, "Continue"))
            done = 1;
        if (done) {
            ScoreListAll(g);
            ShowMasters(g);
            Pause();
            return 0;
        }
        break;
    }
    default:
        kout << "Not yet installed\n";
        break;
    }
    return 1;
}

// 1e7b:0f61: the first turn of a day.
static void FirstTurn(Player *p, Empire *e)
{
    if (r_diplomacy.Bool()) {
        TreatyMessages(p, e);
        DiplomacyMenu(p, e);
    }
    int32 bonus = ProtectionBonus(e);
    StartTurn(e);
    NextPhase(e);
    if (bonus > 0)
        kout << "You receive " << c_number << bonus << c_normal
             << " credits from the Galactic Coordinator!\n";
    if (r_lottery.Bool())
        Lottery(p, e, &g_galaxy);
}

// 1e7b:0197
void PlayGame(Player *p)
{
    if (!GalaxyPlayable(&g_galaxy))
        return;
    GalaxyOpen();
    if (!g_open)
        return;
    int16 id = EmpireOfPlayer(p->index);
    if (id == -1) {
        kout << "You do not have an empire.\n";
        return;
    }
    Empire *e = &g_empires[id];
    if (!Alive(e)) {
        Collapsed(e);
        GalaxyClose();
        return;
    }
    g_current = e;
    e->flags |= g_galaxy.w386;
    g_local = e->flags;
    {
        String name(e->name);
        StatusBar(g_door.name, name.c_str(), (char)(e->id + 'A'));
    }
    if (e->flags && g_door.port) {
        kout << "This game is configured to be a local-only game.  Remote players are not allowed to play.\n";
        GalaxyClose();
        return;
    }
    const char *today = "\nYou are today's ";
    int16 n = 0;
    if (e->id == g_galaxy.solarmaster) {
        n++;
        kout << today << Color(14) << "Solar Master" << c_normal << '!';
    }
    if (e->id == g_galaxy.planetmaster) {
        n++;
        kout << today << Color(11) << "Planet Master" << c_normal << '!';
    }
    if (e->id == g_galaxy.weaponsmaster) {
        n++;
        kout << today << Color(13) << "Weapons Master" << c_normal << '!';
    }
    if (n)
        kout << '\n';
    if (n == 3)
        kout << "The Galactic Coordinator extends Its " << Color(14) << "Personal ";
    if (n == 2)
        kout << "The Galactic Coordinator sends the message:  " << Color(3);
    if (n)
        kout << "Congratulations" << c_normal << "!\n";
    ShowCovertReports(p, e);
    ShowBattleReports(p, e);
    if (e->phase == 0 && e->turnsused == 0)
        FirstTurn(p, e);
    e->lastplay = g_now;
    p->lastplay = g_now;
    e->w2a1 = (int16)r_inactivetimeout.Long();
    if (r_messages.Bool())
        ReadMessages(p, e);
    if (e->turnsused >= e->turnsperday && !g_local) {
        kout << "You have used all of your turns for today.\n\n";
        if (e->year < 0 && g_galaxy.grantdays > 1)
            kout << "(You receive " << g_galaxy.grantturns << " turns every "
                 << g_galaxy.grantdays << " days.\n";
        return;
    }
    TreatyMessages(p, e);
    if (e->phase > 1)
        kout << "Continuing previously interrupted turn...\n";
    while (e->turnsused < e->turnsperday || g_local) {
        int16 ph = e->phase;
        NextPhase(e);
        int16 r = Phase(p, ph, e, &g_galaxy);
        if (e->phase == ph)
            NextPhase(e);
        if (r_saveoften.Bool())
            SystemSaveAll();
        if (!Alive(e)) {
            Collapsed(e);
            break;
        }
        if (!r)
            break;
        TimeCheck();
    }
    StatusBar(g_door.name, p->name, ' ');
    GalaxyClose();
}
