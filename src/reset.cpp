// Resetting the game and deleting empires (segment 6924).
#include "sre.h"
#include "events.h"
#include "battle.h"

void ShowHelpTopic(String topic);
void InputDate(int32 *t);                       // 206a:0498
int16 MessagesEmpireDeleted(int16 id);          // 58b6:25f9
void ClearMessages();                           // 58b6:2a17
int16 TradesEmpireDeleted(int16 id);            // 6e3f:2497
void ClearTrades();                             // 6e3f:245d
void PirateSetName(Pirate *p, const char *s);   // 4b60:01ab

// 6924:0000: ask for a number in [min, max].
int16 AskNumber(const char *prompt, int16 min, int16 max, int16 def)
{
    int16 v = 0;
    do {
        if (v < min || v > max)
            v = def;
        kout << prompt << "?  " << c_bracket << '[' << c_bold;
        kout << Color(9) << "MAX=" << Color(3) << max << c_bracket << ']' << c_normal << ' ';
        kin >> defval(def) >> maxval(max) >> v;
    } while (v < min || v > max);
    return v;
}

// 6924:0128: set up a new game.
void ResetGalaxy(Galaxy *g)
{
    Resource localonly("System.LocalOnly", (int32)0);
    g->w386 = localonly.Bool();
    HostMkdir("DATA");
    HostMkdir("BACKUP");
    if (g->w386) {
        kout << "This game is configured for LOCAL PLAY ONLY.\n\n";
        kout << "You will be able to play an unlimited number of turns.\n";
    }
    ShowHelpTopic("reset-inflation");
    g->w384 = YesNo(1, "Do you want Inflation");
    ShowHelpTopic("reset-maintenance-interval");
    g->grantdays = AskNumber("How many days before maintenance runs", 1, 14, 1);
    if (g_system.regstate >= 2) {
        g->grantturns = AskNumber("How many turns should be allowed per maintenance interval", 1,
                                  12, 5);
        g->protection = AskNumber("How many turns of protection", 0, 30, 20);
    }
    if (g_system.regstate >= 2) {
        ShowHelpTopic("reset-galactic-coordinator-taxes");
        g->w3b0 = AskNumber("What is the galactic tax rate [per 1000 credits]", 0, 100, 15);
    }
    if (g_system.regstate >= 2) {
        ShowHelpTopic("reset-planet-market");
        g->d392 = AskNumber("How many planets are initially available", 0, 25000, 100);
    }
    ShowHelpTopic("reset-planet-limit");
    g->d38e = AskNumber("How many planets per turn can be bought", -1, 25000, -1);
    if (g_system.regstate >= 2) {
        ShowHelpTopic("reset-planet-regeneration");
        g->w38c = AskNumber("How many seconds", 1, 32000, 60);
    }
    ShowHelpTopic("reset-food-market");
    g->d388 = 1000 * (int32)AskNumber("How many thousands of megatons of food", 0, 1000, 70);
    for (int16 i = 0; i <= 9; i++) {
        char name[] = "NEWS/NEWS_.MSG";
        name[9] = (char)('0' + i);
        HostRemove(name);
    }
    g_log2 << Color(14) << "\\   The Galactic Times                                        "
           << Color(7);
    PutTime(g_log2, g_today) << "\n";
    g_log2 << Color(9)
           << "\\___________________________________________________________________________\n\n"
           << c_normal;
    g_log2 << "The game was reset.\n";
    ClearEvents();
    ClearStreamFile(0);
    ClearMessages();
    ClearCovertReports();
    ClearBattleReports();
    ClearTrades();
    for (int16 i = 1; i <= 9; i++)
        PirateInit(&g_pirates[i - 1]);
    if (YesNo(1, "Would you like new pirate names")) {
        for (int16 i = 1; i <= 9; i++)
            PirateSetName(&g_pirates[i - 1], "");
    }
}

// 6924:052c
void AskGameDates(Galaxy *g)
{
    kout << "\nEnter the date that users can start playing:  ";
    InputDate(&g->t44);
    kout << "\nEnter the last date that users can play:      ";
    InputDate(&g->enddate);
}

// 6924:057d: remove an empire from the game.
void DeleteEmpire(Empire *e)
{
    if (e->id == -1)
        return;
    for (int16 i = 0; i < MAXEMPIRES; i++) {
        Empire *o = &g_empires[i];
        MaskClear(&o->d22f[5], e->id);
        MaskClear(&o->d22f[0], e->id);
        MaskClear(&o->d22f[1], e->id);
        MaskClear(&o->d22f[2], e->id);
        MaskClear(&o->d22f[3], e->id);
        MaskClear(&o->d22f[4], e->id);
        MaskClear(&o->d22f[6], e->id);
        SetTreaty(o->treaty[e->id], 0, 0);
        o->treaty[e->id].setc(0);
        o->b2d3[e->id] = 0;
    }
    for (int16 j = 0; j < 10; j++)
        Planets(e, j)->count = 0;
    e->population = -1;
    TradesEmpireDeleted(e->id);
    MessagesEmpireDeleted(e->id);
    ReportsEmpireDeleted(e->id, BATTLE_FILE);
    ReportsEmpireDeleted(e->id, 0);
    ReportsEmpireDeleted(e->id, COVERT_FILE);
    EventsEmpireDeleted(e->id);
}

// 6924:0aab
static void SystemSetup(System *s, String sysop, String bbs)
{
    SystemInit();
    SystemSetSysop(sysop);
    SystemSetBBS(bbs);
    CheckRegistration();
}

// 6924:0993: first-time set-up (sre /Reset with no game).
void ResetGame()
{
    HostMkdir("DATA");
    HostMkdir("BACKUP");
    HostMkdir("NEWS");
    SystemInit();
    char sysop[0x41], bbs[0x41];
    kout << Color(10) << "\nSysop: ";
    kin >> maxlen(0x40) >> sysop >> maxlen(0);
    kout << Color(10) << "BBS:   ";
    kin >> maxlen(0x40) >> bbs >> maxlen(0);
    SystemSetup(&g_system, String(sysop), String(bbs));
}
