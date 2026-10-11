// The game "system" (segment 6c38): data files, main menu, rankings.
#include "sre.h"

System g_system;
Galaxy g_galaxy;
Empire *g_empires;
Pirate *g_pirates;
Empire *g_current;
int16 g_open = 0;

static const char *SYSTEM_FILE = "Data/System.II";      // DS:a1a8
static const char *GALAXY_FILE = "Data/Galaxy.II";      // DS:a1ac
static const char *EMPIRE_FILE = "Data/Empire.II";      // DS:a1b0
static const char *PIRATE_FILE = "Data/Pirate.II";      // DS:a1b4

// 6c38:0000
void SystemLoad()
{
    g_system.created = b_time(0);
    g_system.inuse = b_time(0);
    g_system.galaxy.clear();
    DataFile f(SYSTEM_FILE, 0x142, 0, 1);
    if (!f.read(0, &g_system, 1))
        SystemInit();
    CheckRegistration();
}

// 6c38:00f6
void SystemClearInUse()
{
    g_system.inuse = 0;
}

// 6c38:011d
void SystemInit()
{
    g_system.created = g_now;
    g_system.inuse = g_now;
    g_system.galaxy.clear();
    for (int16 i = 0; i < 0x40; i++) {
        g_system.bbs[i] = 0;
        g_system.sysop[i] = 0;
    }
    g_system.regstate = RS_UNREGISTERED;
}

// 6c38:0188
void SystemSetSysop(const String &s)
{
    strncpy(g_system.sysop, s.c_str(), 0x3f);
    g_system.sysop[0x3f] = 0;
}

// 6c38:01be
void SystemSetBBS(const String &s)
{
    strncpy(g_system.bbs, s.c_str(), 0x3f);
    g_system.bbs[0x3f] = 0;
}

// 6c38:01f8
void SystemShowInfo()
{
    kout << Color(14) << "\nSystem Information\n" << c_normal;
    kout << "BBS:   " << c_number << g_system.bbs << c_normal << '\n';
    kout << "Sysop: " << c_number << g_system.sysop << c_normal << '\n';
}

// 6c38:02b8
void SystemMaintenance()
{
    Maintenance(&g_galaxy);
}

// 6c38:02c9
int16 SystemValid()
{
    if (!(g_system.inuse > 0x12c684c0 && g_system.inuse < 0x38538e40))
        return 0;
    if (!(g_system.created > 0x12c684c0 && g_system.created < 0x38538e40))
        return 0;
    if (g_system.regstate < 0 || g_system.regstate > 5)
        return 0;
    return 1;
}

// 6c38:0349: read Galaxy.II, Empire.II and Pirate.II.
int16 SystemLoadData()
{
    int32 n;
    {
        DataFile f(GALAXY_FILE, 1, 0, 1);
        n = f.numrecs();
    }
    DataFile gf(GALAXY_FILE, (int16)n, 0, 1);
    GalaxyInit(&g_galaxy);
    int16 ok = gf.read(0, &g_galaxy, 1) != 0;
    Checksum gc = g_system.galaxy;
    int16 check = CheckChecksum(&g_galaxy, (uint16)n, gc);
    if (ok && !check && g_galaxy.name[0]) {
        g_log2 << TimeStr(g_now)
               << ": It appears that the data files for this game have been tampered with.\n";
        GalaxyInit(&g_galaxy);
        g_errlog << TimeStr(g_now) << ":  Game data files corrupt!\n";
    }
    {
        DataFile f(EMPIRE_FILE, 1, 0, 1);
        n = f.numrecs() / MAXEMPIRES;
    }
    if (n > 0x2fc) {
        g_open = 0;
        g_errlog << TimeStr(g_now)
                 << ":  FATAL ERROR!  Empire file format has changed!  This version of SRE cannot handle files created by newer versions!\n";
        exit(5);
    }
    if (n > 0 && n < 0x2fc)
        g_errlog << TimeStr(g_now)
                 << ":  NOTICE!  Empire file format has changed.  SRE is assuming you have recently installed a newer version.\n";
    if (ok && n > 0) {
        DataFile f(EMPIRE_FILE, (int16)n, 0, 1);
        for (int16 i = 0; i < MAXEMPIRES; i++) {
            f.read(i, &g_empires[i], 1);
            Checksum c = g_galaxy.empire[i];
            int16 good = CheckChecksum(&g_empires[i], (uint16)n, c);
            if (!good && g_galaxy.name[0]) {
                EmpireInit(&g_empires[i]);
                if (Exists(g_empires[i]) && g_galaxy.name[0]) {
                    g_log2 << TimeStr(g_now) << ":  It appears that the data files for ";
                    PutUntranslated(g_log2, String(g_empires[i].name));
                    g_log2 << " have been tampered with.\n";
                    g_errlog << TimeStr(g_now) << ": Resetting EMPIRE " << i << "!\n";
                }
            }
            g_empires[i].localplay |= g_galaxy.localonly;
        }
    } else {
        for (int16 i = 0; i < MAXEMPIRES; i++)
            EmpireInit(&g_empires[i]);
    }
    {
        DataFile f(PIRATE_FILE, 1, 0, 1);
        n = f.numrecs() / MAXPIRATES;
    }
    if (ok && n > 0 && n <= 0x106) {
        DataFile f(PIRATE_FILE, (int16)n, 0, 1);
        for (int16 i = 1; i <= MAXPIRATES; i++) {
            f.read(i - 1, &g_pirates[i - 1], 1);
            Checksum c = g_galaxy.pirate[i - 1];
            int16 good = CheckChecksum(&g_pirates[i - 1], (uint16)n, c);
            if (!good && g_galaxy.name[0])
                PirateInit(&g_pirates[i - 1]);
        }
    } else {
        for (int16 i = 1; i <= MAXPIRATES; i++)
            PirateInit(&g_pirates[i - 1]);
    }
    return ok;
}

// 6c38:0a34
void SystemSave()
{
    if (SystemValid()) {
        DataFile f(SYSTEM_FILE, 0x142, 0, 1);
        f.write(0, &g_system, 1);
    }
}

// 6c38:0a9a: checksum, encrypt and write everything, then decrypt again.
void SystemSaveAll()
{
    for (int16 i = 0; i < MAXEMPIRES; i++)
        g_galaxy.empire[i] = MakeChecksum(&g_empires[i], 0x2fc);
    for (int16 i = 1; i <= MAXPIRATES; i++)
        g_galaxy.pirate[i - 1] = MakeChecksum(&g_pirates[i - 1], 0x106);
    g_system.galaxy = MakeChecksum(&g_galaxy, 0x3c6);
    SystemSave();
    {
        DataFile gf(GALAXY_FILE, 0x3c6, 0, 1);
        gf.write(0, &g_galaxy, 1);
        DataFile ef(EMPIRE_FILE, 0x2fc, 0, 1);
        ef.write(0, g_empires, MAXEMPIRES);
        DataFile pf(PIRATE_FILE, 0x106, 0, 1);
        pf.write(0, g_pirates, MAXPIRATES);
    }
    if (!Unscramble(&g_galaxy, 0x3c6, g_system.galaxy))
        abort();
    for (int16 i = 0; i < MAXEMPIRES; i++)
        if (!Unscramble(&g_empires[i], 0x2fc, g_galaxy.empire[i]))
            abort();
    for (int16 i = 1; i <= MAXPIRATES; i++)
        if (!Unscramble(&g_pirates[i - 1], 0x106, g_galaxy.pirate[i - 1]))
            abort();
}

// 6c38:10f9
static void MenuStatus()
{
    SystemShowInfo();
}

// 6c38:110a
static void MenuGalaxy()
{
    GalaxyOpen();
    if (!g_open) {
        kout << "The game has not been reset.  Please inform the sysop.\n";
        return;
    }
    Maintenance(&g_galaxy);
    GalaxyMenu(&g_galaxy);
    GalaxyClose();
}

// 6c38:1149
static void MenuWhoAmI(int16 pl)
{
    Player p;
    PlayerLoad(&p, pl);
    PlayerWhoAmI(&p);
    PlayerSave(&p);
}

// 6c38:1180
static void MenuSettings(int16 pl)
{
    Player p;
    PlayerLoad(&p, pl);
    Preferences(&p);
    PlayerSave(&p);
}

// 6c38:0e04
void MainMenu()
{
    int16 def = 2;
    int16 r;
    do {
        TimeCheck();
        int16 pl = PlayerFind(String(g_door.name));
        StatusBar(g_door.name, "Main Menu", ' ');
        Menu m("System", "System", 0xe, 8, 7, 0xf, 7, 8, 8, 7, 7, 7);
        m.Add("Status", '1', 1);
        m.Add("Visit the Galaxy", '2', 2);
        if (pl != -1)
            m.Add("Who am I?", '3', 3);
        m.Add("Bulletins", '4', 4);
        m.Add("Rankings", '8', 8);
        if (pl != -1)
            m.Add("Change your settings", 'P', 13);
        m.Add("More Help", 'H', 15);
        m.Add("Quit", 'Q', 99);
        m.Add("", '0', 99);
        r = m.Run(1, def);
        switch (r) {
        case 1:
            MenuStatus();
            break;
        case 2:
            MenuGalaxy();
            def = 99;
            break;
        case 3:
            MenuWhoAmI(pl);
            break;
        case 4:
            Bulletins();
            break;
        case 8:
            Rankings();
            break;
        case 13:
            MenuSettings(pl);
            break;
        case 15:
            MoreHelp();
            break;
        case 99:
            r = -1;
            break;
        }
    } while (r != -1);
    ShowScreen(0, "Return", 0);
}

// 6c38:11b7: order idx[] by val[idx[]] descending.
static void SortRanks(int16 *idx, const int32 *val, int16 n)
{
    for (int16 i = 0; i < n - 1; i++) {
        for (int16 j = i + 1; j < n; j++) {
            if (val[idx[i]] < val[idx[j]]) {
                int16 t = idx[i];
                idx[i] = idx[j];
                idx[j] = t;
            }
        }
    }
}

// 6c38:126b
static void RankMenu(const int16 *idx, const int16 *emp, const int32 *val, int16 n)
{
    Menu m("Rankings", "", 0xf, 8, 0, 0xa, 7, 3, 0xb, 7, 7, 7);
    String names[25];
    for (int16 i = 0; i < n; i++) {
        names[i] = String(g_empires[emp[idx[i]]].name);
        m.Add(String("#") + IntStr(i + 1), (char)(emp[idx[i]] + 'A'), i + 1);
        m.Find(i + 1)->AddColumn(names[i].c_str());
        m.Find(i + 1)->AddColumn(val[idx[i]]);
    }
    m.Show();
}

// 6c38:1486
void Rankings()
{
    int16 idx[25], emp[25];
    int32 networth[25], planets[25], score[25], points[25];
    GalaxyOpen();
    int16 n = 0;
    for (int16 i = 0; i < MAXEMPIRES; i++) {
        if (!Exists(g_empires[i]))
            continue;
        idx[n] = n;
        emp[n] = i;
        networth[n] = NetWorth(&g_empires[i]);
        planets[n] = TotalPlanets(&g_empires[i]);
        score[n] = g_empires[i].score;
        points[n] = g_empires[i].points;
        n++;
    }
    if (n == 0) {
        kout << "There are no empires in this galaxy.\n";
    } else {
        SortRanks(idx, planets, n);
        kout << Color(14) << "\nEmpires Sorted by Planets";
        RankMenu(idx, emp, planets, n);
        Pause();
        SortRanks(idx, networth, n);
        kout << Color(14) << "\nEmpires Sorted by Net Worth";
        RankMenu(idx, emp, networth, n);
        Pause();
        SortRanks(idx, score, n);
        kout << Color(14) << "\nEmpires Sorted by Score";
        RankMenu(idx, emp, score, n);
        Pause();
        if (g_galaxy.enddate < NeverDate()) {
            SortRanks(idx, points, n);
            kout << Color(14) << "\nEmpires Sorted by Tournament Points";
            RankMenu(idx, emp, points, n);
            Pause();
        }
    }
    GalaxyClose();
}
