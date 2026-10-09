// The sysop menu (segment 6bd1).
#include "sre.h"
#include "menu.h"

void ResetGalaxy(Galaxy *g);                    // 6924:0128 (reset.cpp)
void AskGameDates(Galaxy *g);                   // 6924:052c (reset.cpp)
void EmpireViewer();                            // 473a:0e9a
void PlayerEditor();                            // 473a:0eaf
void PirateEditor();                            // 473a:16eb

// 6bd1:0389: name and reset the galaxy; 1 if it was reset.
static int16 ResetTheGame()
{
    char name[0x40];
    for (;;) {
        kout << "Enter the name of your galaxy:\n> ";
        kin >> maxlen(0x3f) >> name >> maxlen(0);
        if (name[0] == 0) {
            kout << "The galaxy was not created.\n";
            return 0;
        }
        kout << "Name your galaxy " << c_empire << name << c_normal;
        if (YesNo(1, ""))
            break;
    }
    if (!YesNo(0, "Are you sure you want to reset the game"))
        return 0;
    GalaxyReset(&g_galaxy);
    GalaxyOpen();
    GalaxySetName(&g_galaxy, String(name));
    ResetGalaxy(&g_galaxy);
    GalaxyClose();
    return 1;
}

// 6bd1:048d
static void Tournament()
{
    kout << "Tournament setup\n";
    if (ResetTheGame()) {
        GalaxyOpen();
        AskGameDates(&g_galaxy);
        GalaxyClose();
    }
}

// 6bd1:04c7
static void ChangeNames()
{
    char sysop[0x41], bbs[0x41], name[0x40];
    kout << "Change names\n";
    kout << Color(10) << "Sysop: ";
    kin >> maxlen(0x40) >> sysop >> maxlen(0);
    if (sysop[0])
        SystemSetSysop(String(sysop));
    kout << Color(10) << "BBS:   ";
    kin >> maxlen(0x40) >> bbs >> maxlen(0);
    if (bbs[0])
        SystemSetBBS(String(bbs));
    kout << "Enter the name of your galaxy:\n> ";
    kin >> maxlen(0x3f) >> name >> maxlen(0);
    if (name[0]) {
        GalaxyOpen();
        GalaxySetName(&g_galaxy, String(name));
        ResetGalaxy(&g_galaxy);
        GalaxyClose();
    } else {
        kout << "The name was not changed.\n";
    }
}

// 6bd1:0000
void SysopMenu()
{
    int16 r;
    do {
        Menu m("Sysop", "Sysop", 0xe, 8, 7, 0xf, 7, 8, 8, 7, 7, 7);
        m.Add("Status", '1', 1);
        m.Add("Reset the Game", '2', 2);
        m.Add("Reset and set up a Tournament", '3', 3);
        m.Add("Change System Names", '4', 4);
        m.Add("Read Messages", '6', 6);
        m.Add("Send a Message", '7', 7);
        m.Add("Empire Viewer", 'E', 10);
        m.Add("Player Editor", 'P', 11);
        m.Add("Pirate Editor", 'T', 12);
        if (!g_system.regstate)
            m.Add("What happens if I register", '$', 13);
        r = m.Run(1, -1);
        switch (r) {
        case 1:
            SystemShowInfo();
            break;
        case 2:
            ResetTheGame();
            break;
        case 3:
            Tournament();
            break;
        case 4:
            ChangeNames();
            break;
        case 6:
            GalaxyOpen();
            SysopMessages();
            GalaxyClose();
            break;
        case 7:
            GalaxyOpen();
            SendMessage(-1, String(""));
            GalaxyClose();
            break;
        case 10:
            EmpireViewer();
            break;
        case 11:
            PlayerEditor();
            break;
        case 12:
            PirateEditor();
            break;
        case 13:
            ShowHelpTopic("info-register");
            kout << c_normal << "It will show up as:\n\nThis game is registered to " << Color(14)
                 << "(your real name here)";
            kout << c_normal << "\n\nYour bank name will show up as:\n\nWelcome to " << Color(14)
                 << "(your bank name here)";
            kout << c_normal
                 << "\n\nWe only register to your real name, so if you do not wish your real name to appear to your users, please do not register.\n";
            break;
        }
    } while (r != -1);
}
