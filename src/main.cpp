// Program entry (segment 1848).
#include "sre.h"
#include <stdio.h>
#include <unistd.h>

Resource r_unregistered("System.Unregistered", (int32)0);  // 50d0
Resource r_flavor("Flavor", (int32)0);                     // 50dc
DoorInfo g_door;


void DoorRead(DoorInfo *d);
int16 DoorOpen(DoorInfo *d);
void CommInit();
void CommPorts();
void CommandLine();
void LockoutCheck();
int16 ResearchInit();
void SysopMenu();
void ResetGame();
int16 Register();
void TimeStart();
void IdleHook();
void ScoreFiles();

// 1848:0067: file error handler (logs to SYSOP.LOG).
static void FileErrorLog(const char *msg)
{
    g_errlog << TimeStr(g_now) << ":  " << msg;
}

// 1848:00a2 (atexit)
static void AtExit()
{
    while (g_open)
        GalaxyClose();
    HostRemove("InUse.SR");
}

// 1848:0bde: 1 if another node is playing.
static int16 InUse()
{
    int16 stale = 0;
    int32 now = b_time(0);
    {
        IFStream f("InUse.SR", 0);
        if (!f.fail()) {
            char name[0x52];
            int32 t;
            f >> maxlen(0x50) >> name;
            f >> maxlen(0);
            f >> t;
            if (now - t > 3600)
                stale = 1;
        } else {
            stale = 1;
        }
    }
    if (!stale)
        return 1;
    OFStream o("InUse.SR", 0);
    o << g_door.name << '\n';
    o << now << '\n';
    o << "\nThis file tells SRE that this user is still playing SRE.";
    o << "\nIf you delete this file, another user will be able to play.";
    o << "\nThis is to prevent multiple nodes from playing simultaneously.\n";
    return 0;
}

static void WriteUserList()
{
    OFStream o("Text/Userlist.TXT", 1);
    GalaxyOpen();
    for (int16 i = 0; i < MAXEMPIRES; i++) {
        Empire *e = &g_empires[i];
        if (!Exists(*e))
            continue;
        Player p;
        PlayerLoad(&p, e->player);
        o << String(p.user).c_str() << '\n';
        PlayerSave(&p);
    }
    GalaxyClose();
}

// 1848:00c1
static int RealMain()
{
    b_srand((uint16)b_time(0));
    kout << Color(7) << '\n';
    if (r_flavor.Str()[0]) {
        g_flavordir = String("flavors\\") + r_flavor.Str() + "\\";
        LoadResourceFile((g_flavordir + "resource.dat").c_str());
    }
    {
        Resource sb("StatusBar.Background", (int32)g_sbcolor);
        g_sbcolor = (int16)sb.Long();
        g_sbback = (uint8)g_sbcolor;
        g_sbmid = (uint8)(g_sbcolor + 8);
        g_sbfore = 15;
    }
    SetFileErrorHandler(FileErrorLog);
    DoorRead(&g_door);
    // The terminal is the sysop's screen for the local functions and for
    // local play; otherwise it is the remote user's.
    g_console = g_door.port == 0;
    static const char *const localargs[] = { "Reset", "Register", "Sysop", "Edit", "Userlist",
                                             "Maintenance" };
    for (const char *a : localargs)
        if (HasCommandArg(a))
            g_console = 1;
    CommInit();
    Resource statusbars("System.StatusBars", (int32)1);
    if (statusbars.Bool())
        StatusBar("SRE:II", "", '*');
    else
        g_statusbars = 0;
    // The original reports coreleft() and its overlay buffer; these are
    // the figures it gives in a 640K DOS session.
    kout << c_number << (int32)285520 << c_normal << " bytes free";
    Resource expanded("ExpandedSwapping", (int32)0);
    Resource extended("ExtendedSwapping", (int32)0);
    kout << "; " << c_number << (int32)86016 << c_normal
         << " bytes (conventional) used for swap space.\n";
    if (extended.Bool()) {
        kout << "Extended memory swapping\n";
    } else if (expanded.Bool()) {
        kout << "EMS swapping\n";
    } else {
        kout << "Disk swapping\n";
    }
    TimeStart();
    if (InUse()) {
        klocal << "Error:  Game in use on another node.\n";
        CommPorts();
        CommandLine();
        DoorOpen(&g_door);
        kout << "Sorry, it appears that another user is playing this game of SRE at this time.\n";
        SystemClearInUse();
        exit(2);
    }
    atexit(AtExit);
    SetIdle(IdleHook);
    if (HasCommandArg("Reset")) {
        if (!SystemValid() || g_system.sysop[0] == 0)
            ResetGame();
        SysopMenu();
        return 0;
    }
    if (!SystemValid() || g_system.bbs[0] == 0) {
        kout << "Sorry, this game has not been reset.\n";
        kmodem << "Please inform your sysop.\n";
        return 1;
    }
    if (HasCommandArg("Register")) {
        if (!Register())
            kout << c_normal << "Registration has not been completed.\n";
        return 0;
    }
    if (HasCommandArg("Sysop") || HasCommandArg("Edit")) {
        SysopMenu();
        return 0;
    }
    if (HasCommandArg("Userlist")) {
        WriteUserList();
        return 0;
    }
    if (HasCommandArg("Maintenance")) {
        kout << "Running system maintenance:\n";
        SystemMaintenance();
        return 0;
    }
    ResearchInit();
    CommPorts();
    CommandLine();
    DoorOpen(&g_door);
    if (g_door.port == 0)
        g_pagelen--;
    if (g_door.port != 0 && g_door.baud <= 1200)
        g_effects = 0;
    LockoutCheck();
    Resource blank("System.BlankLocalScreen", (int32)0);
    if (blank.Bool() && g_door.port != 0) {
        kout.localon = 0;
        klocal << "Someone is playing SRE:II!\n";
    }
    TitleScreen();
    StatusBar(g_door.name, "SRE", '*');
    if (kout.logon) {
        klog.noansi = 1;
        klog << Color(8)
             << "\n\xda\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4"
                "\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4"
                "\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4"
                "\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\xc4\n";
        klog << '\xb3' << Color(15) << " Log File for '" << g_door.name << "':  \n" << c_normal;
    }
    LoadTranslation();
    MainMenu();
    ScoreFiles();
    if (g_system.regstate != RS_UNREGISTERED) {
        if (Random(3) == 0) {
            PutUntranslated(kout, String("\nThis game is registered to ")) << Color(14);
            PutUntranslated(kout, String(g_system.regto)) << c_normal << '\n';
        }
    } else if (!r_unregistered.Bool() && Random(32) == 0) {
        PutUntranslated(kout, String("\nThis game is currently unregistered!\n"));
    }
    News(0);
    return 0;
}

// 1848 main + Borland start-up: static data, then main.
// Testing aid: SRE_DUMP=<file> writes the empire array at exit (before
// it is saved), to compare with the original's memory.
static void DumpEmpires()
{
    const char *f = getenv("SRE_DUMP");
    if (!f || !g_empires)
        return;
    FILE *fp = fopen(f, "wb");
    if (fp) {
        fwrite(g_empires, sizeof(Empire), MAXEMPIRES, fp);
        fclose(fp);
    }
}

int main(int argc, char **argv)
{
    g_argc = argc;
    g_argv = argv;
    const char *dir = getenv("SRE_DIR");
    if (dir && chdir(dir) != 0) {
        fprintf(stderr, "sre: cannot change to %s\n", dir);
        return 1;
    }
    const char *clk = getenv("SRE_CLOCK");
    if (clk)
        b_setclock((int32)atol(clk), 0);
    TermInit();
    InitTime();
    g_empires = new Empire[MAXEMPIRES];
    g_pirates = new Pirate[MAXPIRATES];
    for (int16 i = 0; i < MAXEMPIRES; i++) {
        memset(&g_empires[i], 0, sizeof(Empire));
        EmpireInit(&g_empires[i]);
    }
    for (int16 i = 0; i < MAXPIRATES; i++)
        PirateNew(&g_pirates[i]);
    GalaxyInit(&g_galaxy);
    SystemLoad();
    atexit(DumpEmpires);
    int r = RealMain();
    exit(r);
}

// 1848:00c1 (end): write the score files named by Scores.File.*.
void ScoreFiles()
{
    GalaxyOpen();
    if (g_open) {
        Resource r("Scores.File.ANSI", "SREScore.ANS");
        const char *name = r.Str();
        if (name[0]) {
            OFStream f(name, 0);
            f.noansi = 0;
            int32 mask = EmpireMask();
            ScoreList(&g_galaxy, mask, &f);
        }
    }
    if (g_open) {
        Resource r("Scores.File.Text", "SREScore.TXT");
        const char *name = r.Str();
        if (name[0]) {
            OFStream f(name, 0);
            f.noansi = 1;
            int32 mask = EmpireMask();
            ScoreList(&g_galaxy, mask, &f);
        }
    }
    GalaxyClose();
}
