// Door file, terminal set-up and command line (segments 7389 and 4018).
#include "sre.h"

void MoreHelp();
void ShowHelpTopic(String topic);

static const char *DOOR_FILE = "DOORFILE.SR";       // DS:bc10

// 7389:0000
static void DoorDefaults(DoorInfo *d)
{
    d->name[0] = 0;
    d->ansi = 1;
    d->ibm = 1;
    d->pagelen = 0x19;
    d->baud = 0x960;
    d->port = 0;
    d->minutes = -1;
    d->loaded = 0;
}

// 7389:006f: read DOORFILE.SR (name, ansi, ibm, page length, baud,
// port, minutes, ...), then the resources that override it.
void DoorRead(DoorInfo *d)
{
    char buf[0x148];
    DoorDefaults(d);
    IFStream f(DOOR_FILE, 0);
    if (!f.fail()) {
        f >> d->name;
        f >> buf;
        d->ansi = (int16)b_atol(buf);
        f >> buf;
        d->ibm = (int16)b_atol(buf);
        f >> buf;
        d->pagelen = (int16)b_atol(buf);
        f >> buf;
        int32 v = b_atol(buf);
        d->baud = (v > 0xffff) ? 0x9600 : (uint16)v;
        f >> buf;
        d->port = (int16)b_atol(buf);
        f >> buf;
        v = b_atol(buf);
        d->minutes = v > 0x7fff ? 0x7fff : (int16)v;
    }
    if (!f.fail())
        d->loaded = 1;
    d->lockbaud = d->baud;
    Resource ransi("Ansi", (int32)d->ansi);
    Resource ribm("IBMCharacters", (int32)d->ibm);
    Resource rcheck("CheckTimeLimit", (int32)1);
    Resource rlocal("Local", (int32)0);
    Resource rlock("LockBaudRate", (int32)0);
    Resource rtimeout("TimeOut", (int32)g_timeout);
    g_timeout = (int16)rtimeout.Long();
    if (ransi.Long() != -1)
        d->ansi = ransi.Bool();
    if (ribm.Long() != -1)
        d->ibm = ribm.Bool();
    if (!rcheck.Bool())
        d->minutes = -1;
    if (rlocal.Bool())
        d->port = 0;
    if (rlock.Long() != 0) {
        int32 b = rlock.Long();
        if (b >= 300 && b <= 0xffff) {
            d->baud = (uint16)b;
            klocal << "Locking baud rate at " << Color(11) << d->baud << Color(7) << " baud.\n";
        } else {
            klocal << Color(14)
                   << "Invalid resource 'LockBaudRate':  baud rate must be from 300 to 65535 baud.\n"
                   << Color(7);
        }
    }
    g_ansi = d->ansi;
    g_ibm = d->ibm;
}

// 7389:04b1
int16 DoorOpen(DoorInfo *d)
{
    if (!d->loaded) {
        kout << DOOR_FILE << " not found.  Please run the install program!\n";
        exit(1);
    }
    if (d->pagelen == 0 || d->pagelen < 10)
        d->pagelen = 0x17;
    g_remote = d->port;
    if (d->port == 0) {
        if (d->ansi == -1)
            d->ansi = 1;
    } else if (d->ansi == -1) {
        kout << "Do you have ANSI color?  [Y/n] ";
        kout.raw = 1;
        char c;
        do {
            kin >> c;
            c = (char)b_toupper((int)(int8)c);
            if (c == '\r')
                c = 'Y';
        } while (c != 'Y' && c != 'N');
        kout.raw = 0;
        d->ansi = (c == 'Y');
        kout << c << '\n';
    }
    if (d->ibm == -1)
        d->ibm = d->ansi;
    else
        d->ibm &= 1;
    g_ansi = d->ansi;
    g_ibm = d->ibm;
    g_remote = d->port;
    kout.noansi = g_ansi ? 0 : 1;
    return 1;
}

// ---------------------------------------------------------------------
// 4018:0000: COM port addresses and interrupts (no effect here).
void CommPorts()
{
    for (int16 i = 1; i <= 8; i++) {
        String name = String("COM") + IntStr(i);
        Resource r(name.c_str(), "");
        (void)r.Str();
    }
}

// 4018:01dc
void CommInit()
{
    Resource effects("SpecialEffects", (int32)g_effects);
    Resource video("Display.Video", "Direct");
    Resource vga("Display.VGA", (int32)0);
    Resource multi("System.Multitasker", (int32)0);
    Resource fossil("System.Fossil", (int32)0);
    Resource bios("BIOS", (int32)0);
    g_effects = effects.Bool();
    {
        Resource comma("Output.Comma", (int32)1);
        if (!comma.Bool()) {
            String s(comma.Str());
            if (s.length() == 1)
                g_comma = s.at(0);
            else
                g_comma = 0;
        }
    }
    {
        Resource sep("Output.Time.Separator", ":");
        String s(sep.Str());
        if (s.length() == 1)
            g_timesep = s.at(0);
    }
    Resource h24("Output.Time.24Hour", (int32)0);
    g_time24 = h24.Bool();
}

// 4018:0603
void CommandLine()
{
    if (g_argc > 1 && b_stricmp(g_argv[1], "/?") == 0) {
        ShowHelpTopic("SRE-command-line-options");
        Pause();
        exit(0);
    }
    if (HasCommandArg("HELP")) {
        StatusBar("Help", "", '?');
        MoreHelp();
        Pause();
        exit(0);
    }
    if (HasCommandArg("CreateINSTRUCTIONS")) {
        StatusBar("Creating Instructions Files", "!", '?');
        g_pagelen = 0;
        kout.localon = 0;
        kout.OpenLog("Text/Player.ANS");
        klog.noansi = 0;
        DisplayFile("Text/Player.MSG", 0);
        kout.CloseLog();
        kout.OpenLog("Text/Player.TXT");
        klog.noansi = 1;
        DisplayFile("Text/Player.MSG", 0);
        kout.CloseLog();
        exit(0);
    }
    Resource log("Game.Log", "");
    if (strlen(log.Str()) > 0)
        kout.OpenLog(log.Str());
}

// 4018:078a
void LockoutCheck()
{
    IFStream f("Text/Lockout.LST", 0);
    while (!f.fail()) {
        char name[0x100];
        f >> name;
        if (b_stricmp(name, g_door.name) == 0) {
            if (!ShowScreen(0, "Lockout", 0))
                kout << "You are not to play this game.\n";
            exit(1);
        }
    }
}
