// The sysop's empire viewer, player editor and pirate editor (segment
// 473a).  The empire and player lists are drawn on the local screen.
#include "sre.h"
#include "conio.h"
#include "menu.h"
#include "files.h"

int16 ChoosePirates();                          // 5fe0:0000 (pirateraid.cpp)
void PirateSetName(Pirate *p, const char *s);   // 4b60:01ab
void PlayerListNames(PickList *l);              // 64d2:0835 (player.cpp)

static int16 g_chosen = 1;                      // DS:2632

// 473a:0000: one line of the empire list.
static void EmpireLine(int16 row, char letter, String name, int32 planets, int32 score,
                       int32 networth, int16 attr)
{
    kwin.At(1, row);
    kwin << Color(2, 0) << '<' << Color(15, 2) << letter << Color(2, 0) << '>';
    kwin.SetColor(attr >> 4, attr & 0xf);
    kwin << ' ' << setw(0x1e) << name << setw(0) << ' ';
    kwin << Color(11) << setw(10) << planets << setw(0);
    kwin << Color(14) << setw(11) << score << setw(0);
    kwin << Color(13) << setw(13) << networth << setw(0);
    kwin << Color(7, 0);
    kwin.At(2, row);
}

static void EmpireLine(int16 row, int16 id, int16 attr)
{
    Empire *e = &g_empires[id];
    EmpireLine(row, (char)('A' + id), String(e->name), TotalPlanets(e), e->score, NetWorth(e),
               attr);
}

// 473a:01f8: pick an empire from a list on the local screen; its id, or
// -1.  (The galaxy is left open.)
static int16 ChooseEmpire()
{
    int16 highlight = (int16)((g_sbcolor << 4) | 0xf);
    int16 normal = 7;
    int16 n = 0;
    int16 ids[MAXEMPIRES];
    GalaxyOpen();
    FullWindow();
    kwin.SetColor(0, 7);
    kwin.Clear();
    for (int16 i = 0; i < MAXEMPIRES; i++) {
        if (g_empires[i].name[0] == 0)
            continue;
        ids[n++] = i;
        EmpireLine(n, i, n == g_chosen ? highlight : normal);
    }
    if (g_chosen > n)
        g_chosen = n;
    if (g_chosen < 1)
        g_chosen = 1;
    if (n == 0) {
        kwin << "Sorry, there are no users here!\nPress a key...";
        bioskey0();
        return -1;
    }
    kwin.At(2, 1);
    char c;
    for (;;) {
        int16 prev = g_chosen;
        kwin.raw = 1;
        kwinin >> c;
        if (c == 0) {
            kwinin >> c;
            if (c == 0x2d)              // Alt-X
                break;
            if (c == 0x50)
                g_chosen++;
            if (c == 0x48)
                g_chosen--;
            if (c == 0x4f)
                g_chosen = n;
            if (c == 0x47)
                g_chosen = 1;
            if (c == 0x51)
                g_chosen += 5;
            if (c == 0x49)
                g_chosen -= 5;
            if (g_chosen < 1)
                g_chosen = 1;
            if (g_chosen > n)
                g_chosen = n;
            c = 0;
        }
        kwin.raw = 0;
        if (c == '\r' || c == 0x1b)
            break;
        c = (char)b_toupper(c);
        if (c >= 'A' && c <= 'Z') {
            for (int16 i = 0; i < n; i++)
                if (c == ids[i] + 'A')
                    g_chosen = (int16)(i + 1);
        }
        if (prev != g_chosen) {
            EmpireLine(prev, ids[prev - 1], normal);
            EmpireLine(g_chosen, ids[g_chosen - 1], highlight);
        }
    }
    kwin.raw = 0;
    kwin.At(1, 1);
    kwin.SetColor(2, 15);
    kwin.ClearEol();
    kwin.SetColor(0, 7);
    kwin.Clear();
    if (c == '\r')
        return ids[g_chosen - 1];
    return -1;
}

// 473a:05d4: view and change an empire; 0 when the sysop is done.
static int16 ViewEmpire()
{
    IdleFn idle = SetIdle(0);
    int16 id = ChooseEmpire();
    if (id == -1)
        return 0;
    Empire *e = &g_empires[id];
    if (e->name[0] == 0) {
        kout << c_normal << "Empire doesn't exist.\n";
        SetIdle(idle);
        return 0;
    }
    char key;
    do {
        PutUntranslated(kout << "\nStatus of empire:  " << c_empire, String(e->name))
            << c_normal << '\n';
        kout << "Controlled by player #" << c_number << e->player << c_normal;
        Player p;
        PlayerLoad(&p, e->player);
        PutUntranslated(kout << '(', String(p.name)) << ") Username is " << String(p.user)
                                                     << '\n';
        PlayerSave(&p);
        ShowStatus(e);
        kout << "Change " << c_bracket << '[' << c_bold << 'P' << c_bracket << ']' << c_normal
             << "layer, ";
        kout << c_bracket << '[' << c_bold << 'E' << c_bracket << ']' << c_normal
             << "mpire Name, ";
        kout << c_bracket << '[' << c_bold << 'D' << c_bracket << ']' << c_normal
             << "elete Empire, or ";
        kout << c_bracket << '[' << c_bold << 'Q' << c_bracket << ']' << c_normal << "uit?  "
             << c_input;
        key = GetChoice("QPDE", 1);
        char buf[0x20];
        if (key == 'P') {
            kout << "Change Player" << c_normal << "\nNew username:  ";
            kin >> maxlen(0x1e) >> buf >> maxlen(0);
            b_strupr(buf);
            int16 who = PlayerFind(String(buf));
            if (who == -1 && String(buf).length() > 0) {
                kout << "Player record not found.  If you wish to change the handle of a user, "
                        "use the Player Editor; if you wish to transfer control, the second "
                        "player must create a player record by Joining the Game but not "
                        "entering an Empire name; then the player will have a record and you "
                        "can return here to change the empire's controlling player.\n";
            } else {
                kout << "Change player to #" << who << " (" << c_empire << buf << c_normal
                     << ')';
                if (YesNo(0, "")) {
                    Player q;
                    PlayerLoad(&q, who);
                    e->player = q.index;
                    PlayerSave(&q);
                }
            }
            key = 0;
            Pause();
        } else if (key == 'E') {
            kout << "Change Empire Name" << c_normal << "\nNew empire name:  ";
            kin >> maxlen(0x1e);
            strcpy(buf, e->name);
            kin.GetLine(buf);
            kin >> maxlen(0);
            kout << "Change empire name to " << c_empire << buf << c_normal;
            if (YesNo(0, "")) {
                PutUntranslated(g_log2 << "Empire " << c_empire, String(e->name))
                    << c_normal << " changed name to " << Color(14) << buf << c_normal
                    << ".\n";
                EmpireSetName(e, String(buf));
            }
            key = 0;
            Pause();
        } else if (key == 'D') {
            kout << "Delete Player" << c_normal << '\n';
            if (!YesNo(0, "Delete this player")) {
                kout << Color(14) << "Aborted\n";
                key = 0;
            }
            Pause();
        }
    } while (key != 'D' && key != 'Q');
    if (key == 'D') {
        kout << Color(14) << "Deleting player...\n";
        PutUntranslated(g_log2 << "Empire " << c_empire, String(e->name))
            << c_normal << " was deleted by the sysop.\n";
        DeleteEmpire(e);
        EmpireInit(e);
    } else {
        kout << "Quit & Save" << c_normal << '\n';
    }
    GalaxyClose();
    SetIdle(idle);
    return 1;
}

// 473a:0e9a
void EmpireViewer()
{
    g_pagelen = 0;
    while (ViewEmpire())
        ;
}

// The player chosen from a list under a heading (the Delete and Edit
// commands of 473a:0eaf); -1 if none.
static int16 PickPlayer(const char *heading)
{
    kwin.Clear();
    kwin << Color(14, 1) << heading;
    kwin.ClearEol();
    TextInfo t;
    gettextinfo(&t);
    SetWindow(t.winleft, (int16)(t.wintop + 1), t.winright, t.winbottom);
    PickList *l = new PickList(0x17, 0x7f);
    PlayerListNames(l);
    const char *s = l->Item(l->Run());
    String name(s ? s : "");
    kwin.SetColor(0, 7);
    SetWindow(t.winleft, t.wintop, t.winright, t.winbottom);
    kwin.Clear();
    int16 who = PlayerFind(name);
    if (who == -1)
        who = (int16)(b_atol(name.c_str()) - 1);
    delete l;
    return who;
}

// 473a:0eaf
void PlayerEditor()
{
    kout << Color(14) << "Player Editor\n\n";
    for (;;) {
        kout << c_bracket << '[' << c_bold << 'L' << c_bracket << ']' << c_normal
             << "ist players, ";
        kout << c_bracket << '[' << c_bold << 'E' << c_bracket << ']' << c_normal
             << "dit a player, ";
        kout << c_bracket << '[' << c_bold << 'D' << c_bracket << ']' << c_normal
             << "elete a player, or ";
        kout << c_bracket << '[' << c_bold << 'Q' << c_bracket << ']' << c_normal << "uit? ";
        char key = GetChoice("QLED", 1);
        kout << c_input << key << c_normal << '\n';
        if (key == 'Q')
            return;
        if (key == 'L') {
            PickList *l = new PickList(0x17, 0x7f);
            PlayerListNames(l);
            kwin.Clear();
            l->Run();
            kwin.SetColor(0, 7);
            kwin.Clear();
            delete l;
        } else if (key == 'D') {
            int16 who = PickPlayer(" Delete Player");
            if (who == -1) {
                kout << "Player not found!\n";
                continue;
            }
            GalaxyOpen();
            int16 i;
            for (i = 0; i < MAXEMPIRES; i++) {
                if (g_empires[i].name[0] && g_empires[i].player == who) {
                    kout << "This player is currently controlling empire " << (char)('A' + i)
                         << ".  Please delete the empire first if you wish to delete this "
                            "player.\n";
                    break;
                }
            }
            GalaxyClose();
            if (i < MAXEMPIRES)
                continue;
            Player p;
            PlayerLoad(&p, who);
            p.name[0] = 0;
            PlayerSave(&p);
        } else if (key == 'E') {
            int16 who = PickPlayer(" Edit Player");
            if (who == -1) {
                kout << "Player not found!\n";
                continue;
            }
            Player p;
            PlayerLoad(&p, who);
            kout << c_normal << "Player #" << who << "\n";
            kout << "User:  " << String(p.user) << '\n';
            kout << "Name:  " << String(p.name) << '\n';
            char buf[0x52];
            if (YesNo(0, "Change the username")) {
                kout << "To?  ";
                kin >> maxlen(0x50);
                strcpy(buf, p.user);
                kin.GetLine(buf);
                if (buf[0]) {
                    if (PlayerFind(String(buf)) != -1)
                        kout << "There is already another player with that username.\n";
                    else
                        PlayerSetUser(&p, String(buf));
                } else {
                    kout << "(not changed)\n";
                }
            }
            if (YesNo(0, "Change the player name")) {
                kout << "To?  ";
                kin >> maxlen(0x50);
                strcpy(buf, p.name);
                kin.GetLine(buf);
                if (buf[0])
                    PlayerSetName(&p, String(buf));
                else
                    kout << "(not changed)\n";
            }
            PlayerSave(&p);
        }
    }
}

// 473a:16eb
void PirateEditor()
{
    kout << Color(14) << "\nEnemy Name Editor" << c_normal << '\n';
    GalaxyOpen();
    CreatePirates();
    int16 team;
    while ((team = ChoosePirates()) != -1) {
        Pirate *pt = &g_pirates[team - 1];
        char name[0x22];
        kout << c_normal << "Change the name to:  ";
        strcpy(name, PirateName(pt));
        kin >> maxlen(0x20);
        kin.GetLine(name);
        kin >> maxlen(0);
        kout << c_normal << "Change the color number to:  ";
        int16 color = PirateColor(pt);
        kin >> maxval(0xf) >> defval((int32)color) >> color;
        if (color <= 0)
            color = 8;
        kout << c_normal << "The pirate team will be called " << Color(color);
        if (name[0])
            kout << name;
        else
            kout << PirateName(pt);
        kout << c_normal;
        if (YesNo(1, ".  Is this OK")) {
            if (name[0])
                PirateSetName(pt, name);
            pt->color = (uint8)color;
        }
    }
    GalaxyClose();
}
