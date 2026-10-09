// The galaxy menu and joining a game (segment 62b1), and the comparison
// of empire names (segment 7458).
#include "sre.h"

void PlayGame(Player *p);                               // 1e7b:0197
void ReadMessages(Player *p, Empire *e);                // 58b6:1eb3
int16 ReadPublicMessages();                             // 58b6:2901
void SendMessage(int16 from, String name);              // 58b6:1ce9
void TreatyMessages(Player *p, Empire *e);                   // 70d2:1ddf
void DiplomacyMenu(Player *p, Empire *e);                // 70d2:0000
void DeleteEmpire(Empire *e);                           // 6924:057d
void PlayerInitIndex(Player *p, int16 index);           // 64d2:090c
extern Resource r_messages;                             // 62e8
extern Resource r_diplomacy;                            // b01e

// ---------------------------------------------------------------------
// 7458

// 7458:0000: trim spaces and convert to upper case.
static int16 NameTrim(String &s)
{
    while (s.length() > 0 && (ctype_signed(s.at(0)) & CT_SP))
        s.erase(0, 1);
    while (s.length() > 0 && (ctype_signed(s.at((int16)(s.length() - 1))) & CT_SP))
        s.erase((int16)(s.length() - 1), 1);
    for (int16 i = 0; i < s.length(); i++)
        s.raw()[(uint16)i] = (char)b_toupper((int)(int8)s.at(i));
    return s.length();
}

// 7458:00c0: 0xff becomes a space; runs of spaces become one.
static void NameSpaces(String &s)
{
    for (;;) {
        int16 i = s.find(String("\xff"), 0);
        if (i < 0)
            break;
        s.raw()[(uint16)i] = ' ';
    }
    for (;;) {
        int16 i = s.find(String("  "), 0);
        if (i < 0)
            break;
        s.erase(i, 1);
    }
}

// 7458:014c: letters and digits only.
static void NameAlnum(String &s)
{
    String t("");
    for (int16 i = 0; i < s.length(); i++) {
        char c = s.at(i);
        if ((ctype_signed(c) & (CT_DIG | CT_UPP | CT_LOW)) && (uint8)c < 0xa8)
            t += c;
    }
    s = t;
}

// 7458:01de
void NameNormalize(String &s, int16 level)
{
    if (level < 1)
        return;
    NameTrim(s);
    if (level < 2)
        return;
    NameSpaces(s);
    if (level < 3)
        return;
    NameAlnum(s);
}

// 7458:021f: how alike two names are (0 = different, 2 = the same but
// for spacing, 3 = the same or similar).
int16 NameSimilarity(String a, String b)
{
    NameTrim(a);
    NameTrim(b);
    if (StrEqI(b, a))
        return 3;
    NameSpaces(a);
    NameSpaces(b);
    if (StrEqI(b, a))
        return 2;
    NameAlnum(a);
    NameAlnum(b);
    int16 la = a.length(), lb = b.length();
    int16 diff = 0;
    if (la - lb > 2 || lb - la > 2)
        return 0;
    for (int16 i = 0; i < la && i < lb; i++) {
        if (b_toupper((int)(int8)a.at(i)) != b_toupper((int)(int8)b.at(i)))
            diff++;
    }
    if (diff * 10 < la + lb)
        return 3;
    return 0;
}

// ---------------------------------------------------------------------

static void Key(char c)
{
    kout << c_bracket << '[' << c_bold << c << c_bracket << ']' << c_normal;
}

// 62b1:05b9: the settings of the game.
static void GalaxyInfo(Galaxy *g)
{
    kout << c_normal << "\nThis is " << Color(15) << g->name << c_normal << ".\n";
    kout << "There are " << c_number << g->grantturns << c_normal << " turns per day.\n";
    if (g->grantdays > 1)
        kout << "Each \"day\" (maintenance interval) lasts for " << c_number << g->grantdays
             << c_normal << " real days.\n";
    kout << "Each empire receives " << c_number << g->protection << c_normal
         << " turns of protection.\n";
    kout << c_number << g->d392 << c_normal << " planets are available.\n";
    kout << c_number << g->d388 << c_normal << " megatons of food are available.\n";
    if (g->d38e >= 0)
        kout << c_number << g->d38e << c_normal << " planets can be bought each turn.\n";
    if (g_system.regstate)
        kout << "The interest rate is " << g->interest << "%.\n";
    kout << "This game is set up with ";
    if (g->w384)
        kout << "traditional inflationary economics.\n";
    else
        kout << "non-inflationary prices.\n";
    if (g->enddate < Midnight(0x7fffffff)) {
        kout << "This is a tournament game that ";
        if (g->t44 <= g_today)
            kout << "started ";
        else
            kout << "starts ";
        PutTime(kout, g->t44) << ", and ends ";
        PutTime(kout, g->enddate) << ".\n";
    }
}

// 62b1:0964
static void NewNews(Galaxy *g, int16 id)
{
    Empire *e = &g_empires[id];
    if (!Exists(*e))
        return;
    kout << "New news since ";
    PutTime(kout, e->d2ec) << ":\n";
    for (char c = '9'; c >= '0'; c--) {
        char name[] = "News/NEWS0.MSG";
        name[9] = c;
        ShowNews(String(name), e->d2ec);
    }
    e->d2ec = b_time(0);
}

// 62b1:0a82
static void OldNews()
{
    kout << "See news from how many days ago?  " << c_bracket << '[' << c_bold
         << "0=Today, or 1-9" << c_bracket << ']' << c_normal << "  ";
    for (;;) {
        char c;
        kin >> c;
        if (c == '\r' || c == 27 || c == 'Q' || c == 'q') {
            kout << "Quit\n";
            return;
        }
        if (c < '0' || c > '9')
            continue;
        kout << c << " day";
        if (c != '1')
            kout << 's';
        kout << '\n';
        char name[] = "News/NEWS_.MSG";
        name[9] = c;
        if (!ShowNews(String(name), 0))
            kout << "The news file from " << c << " days ago is missing.\n";
        return;
    }
}

// 62b1:0c07
static void JoinGame(Galaxy *g)
{
    if (g_today >= g->t4c) {
        kout << "No new players can join this game.\n\n";
        return;
    }
    if (!YesNo(0, "Are you sure you would like to join this game"))
        return;
    int16 pl = PlayerFind(String(g_door.name));
    int16 isnew = 0;
    if (pl == -1) {
        pl = PlayerFree();
        if (pl == -1) {
            kout << "Sorry, SRE is unable to add you to the system.\n";
            return;
        }
        Player np;
        PlayerLoad(&np, pl);
        PlayerInitIndex(&np, pl);
        StatusBar(g_door.name, "Creating New Player", '*');
        for (;;) {
            ShowHelpTopic("reset-new-player");
            kout << "Enter the name of your ruler:\n> ";
            char name[0x20];
            kin >> maxlen(0x1f) >> name >> maxlen(0);
            if (name[0] == 0) {
                kout << "Ruler not created.\n";
                PlayerSave(&np);
                return;
            }
            PlayerSetName(&np, String(name));
            PlayerSetUser(&np, String(g_door.name));
            kout << c_normal << "Are you ";
            Key('F');
            kout << "emale or ";
            Key('M');
            kout << "ale?  " << Color(15);
            char c = 0;
            do {
                kin >> c;
                c = (char)b_toupper((int)(int8)c);
            } while (c != 'M' && c != 'F');
            kout << (c == 'M' ? "Male" : "Female") << c_normal << '\n';
            np.male = (c == 'M') ? 1 : 0;
            kout << "You will be:  " << String(np.male ? "Emperor" : "Emperess") << c_empire << ' '
                 << Untr(np.name) << c_normal << '\n';
            if (YesNo(1, "Is this correct"))
                break;
        }
        ShowScreen(0, "NewPlaye", 0);
        isnew = 1;
        PlayerSave(&np);
    }
    Player p;
    PlayerLoad(&p, pl);
    if (p.name[0] == 0) {
        kout << "There is a problem with SRE:  error #<galaxy:joingame>.\n";
        PlayerSave(&p);
        return;
    }
    if (isnew) {
        kout << "Welcome, " << Color(14) << String(p.male ? "Emperor" : "Emperess") << ' '
             << Untr(p.name) << c_normal << "!\n\n";
        if (YesNo(1, "Do you wish to view the instructions"))
            DisplayFile("Text/Player.Msg", 0);
    }
    int16 slot = -1;
    int16 n = 0;
    int32 open = 0;
    GalaxyOpen();
    for (int16 i = 24; i >= 0; i--) {
        if (!Exists(g_empires[i])) {
            MaskSet(&open, i);
            n++;
            slot = i;
        }
    }
    if (n == 0) {
        if (!ShowScreen(0, "GameFull", 0))
            kout << "Sorry - this game is full.\n";
        PlayerSave(&p);
        return;
    }
    if (n > 1) {
        kout << "There are " << c_number << n << c_normal << " open slots for new players:\n    "
             << c_empire;
        for (int16 i = 0; i < MAXEMPIRES; i++)
            if (MaskTest(open, i))
                kout << (char)(i + 'A') << ' ';
        kout << c_normal << "\nWhich one would you like?  [ENTER to pick the first slot]  "
             << c_input;
        char c;
        for (;;) {
            kin >> c;
            if (c == '\r')
                break;
            if (c == 27) {
                kout << c_normal << "Aborted!" << c_normal << '\n';
                PlayerSave(&p);
                return;
            }
            c = (char)b_toupper((int)(int8)c);
            if (c < 'A' || c > 'Y')
                continue;
            if (MaskTest(open, (int16)(c - 'A')))
                break;
        }
        if (c >= 'A' && c <= 'Y') {
            slot = (int16)(c - 'A');
            kout << c;
        } else {
            kout << "[first slot]";
        }
        kout << c_normal << '\n';
    }
    GalaxyClose();
    Empire *e = &g_empires[slot];
    if (!Exists(*e)) {
        EmpireInitId(e, slot);
        DeleteEmpire(e);
        EmpireInitId(e, slot);
        StatusBar(g_door.name, "Creating New Empire", '*');
        char name[0x40];
        for (;;) {
            kout << "Choose a name for your new empire:\n> ";
            kin >> maxlen(0x3f) >> name >> maxlen(0);
            if (name[0] == 0) {
                kout << "An empire was not created.\n";
                PlayerSave(&p);
                return;
            }
            int16 letters = 0, nonspace = 0;
            for (int16 i = 0; String(name).length() > i; i++) {
                char ch = name[i];
                if ((ctype_signed(ch) & (CT_DIG | CT_UPP | CT_LOW)) && (uint8)ch < 0xa8)
                    letters++;
                if (!(ctype_signed(ch) & CT_SP))
                    nonspace++;
            }
            if (nonspace / 2 > letters && nonspace > 3) {
                kout << "Please use a name rather than symbols.\n";
                continue;
            }
            if (letters < 3) {
                kout << "Please use a longer name.\n";
                continue;
            }
            int16 match = -1, how = 0;
            for (int16 i = 0; i < MAXEMPIRES; i++) {
                if (!Exists(g_empires[i]))
                    continue;
                String other(g_empires[i].name);
                int16 r = NameSimilarity(String(name), other);
                if (r > 0) {
                    how = r;
                    match = i;
                }
            }
            if (match != -1) {
                if (how == 1)
                    kout << "That name is already being used by empire " << c_empire
                         << (char)(match + 'A') << c_normal << '\n';
                if (how == 2)
                    kout << "Empire " << (char)(match + 'A') << " is named '" << c_empire
                         << Untr(g_empires[match].name) << c_normal
                         << "'.  Please choose another name.\n";
                if (how == 3)
                    kout << "That name is similar to '" << c_empire << Untr(g_empires[match].name)
                         << c_normal << "'.  To avoid confusion, please choose a different name.\n";
                continue;
            }
            kout << "Name your empire " << c_empire << name << c_normal;
            if (YesNo(1, ""))
                break;
        }
        EmpireSetName(e, String(name));
    }
    e->flags = g->w386;
    e->player = p.index;
    kout << "You are:  " << String(p.male ? "Emperor" : "Emperess") << Color(14) << ' '
         << Untr(p.name) << c_normal;
    kout << ", and you now rule " << c_empire << Untr(e->name) << c_normal << ".\n";
    PlayerSave(&p);
}

// 62b1:1bd7
static void TournamentResults(Galaxy *g)
{
    kout << Color(14) << "\nTournament Results\n";
    HLine(Color(4), 0x4b);
    kout << c_normal << "Tournament started:  " << Color(12);
    PutTime(kout, Midnight(g->t44)) << '\n';
    kout << c_normal << "Tournament ended:    " << Color(12);
    PutTime(kout, Midnight(g->enddate)) << '\n';
    kout << c_normal;
    int16 idx[25];
    int32 pts[26];
    for (int16 i = 0; i < MAXEMPIRES; i++) {
        idx[i] = i;
        pts[i] = 0;
        Empire *e = &g_empires[i];
        if (!Exists(*e))
            continue;
        if (Exists(*e) && TotalPlanets(e) > 0 && e->population > 0)
            pts[i] = e->d178;
    }
    pts[25] = 0;
    for (int16 i = 0; i < 24; i++) {
        for (int16 j = i + 1; j < 25; j++) {
            if (pts[i] < pts[j]) {
                int32 t = pts[i];
                pts[i] = pts[j];
                pts[j] = t;
                int16 k = idx[i];
                idx[i] = idx[j];
                idx[j] = k;
            }
        }
    }
    int16 last = -1;
    for (int16 i = 0; i < MAXEMPIRES; i++)
        if (pts[i] > 0)
            last = i;
    if (last == -1) {
        kout << "There were no winners.\n";
    } else {
        kout << "\nEmpires are listed by rank:\n";
        kout << " #      Empire name                          Points\n";
        int16 rank = 0;
        for (int16 i = 0; i <= last; i++) {
            kout << setw(2) << Color(14) << (int16)(rank + 1) << setw(0) << Color(6) << ":  ";
            kout << Color(12) << (char)(idx[i] + 'A') << "  ";
            kout << setw(0x23) << Color(15) << Untr(g_empires[idx[i]].name);
            kout << setw(8) << Color(10) << pts[i] << setw(0) << '\n';
            if (pts[i + 1] < pts[rank])
                rank = (int16)(i + 1);
        }
    }
    HLine(Color(4), 0x4b);
}

// 62b1:0000
void GalaxyMenu(Galaxy *g)
{
    int16 def = 6;
    int16 r;
    CreatePirates();
    do {
        TimeCheck();
        StatusBar(g_door.name, g->name, '*');
        int16 pl = PlayerFind(String(g_door.name));
        int16 id = EmpireOfPlayer(pl);
        if (id != -1)
            g_current = &g_empires[id];
        Empire *e = id != -1 ? &g_empires[id] : 0;
        Menu m("Galaxy", "Galaxy Menu", 0xe, 4, 0xc, 0xf, 7, 0xd, 0xe, 7, 7, 7);
        m.Add("Status", '1', 1);
        if (id != -1)
            m.Add("New News", '2', 2);
        m.Add("Daily News", '3', 3);
        if (g_today > g->enddate) {
            m.Add("See Tournament Results", '4', 11);
        } else if (id == -1) {
            m.Add("Join this Game", '4', 10);
            if (def == 6)
                def = 8;
        } else {
            m.Add("Play SRE!", '4', 4);
            if (e->turnsused < e->turnsperday && Exists(*e) && TotalPlanets(e) > 0 &&
                e->population > 0)
                def = 4;
            else if (def == 4)
                def = -1;
        }
        if (id != -1)
            m.Add("See Your Status", '5', 5);
        if (r_messages.Bool()) {
            m.Add("Read Messages", '6', 6);
            m.Add("Send Messages", '7', 7);
        } else if (def == 6) {
            def = -1;
        }
        m.Add("Scores", '8', 8);
        if (id != -1 && Exists(*e) && TotalPlanets(e) > 0 && e->population > 0 &&
            g_today <= g->enddate && r_diplomacy.Bool())
            m.Add("Diplomacy", '9', 9);
        r = m.Run(1, def);
        switch (r) {
        case 1: GalaxyInfo(g); break;
        case 2: NewNews(g, id); break;
        case 3: OldNews(); break;
        case 4: {
            Player p;
            PlayerLoad(&p, pl);
            PlayGame(&p);
            PlayerSave(&p);
            break;
        }
        case 5:
            if (Exists(g_empires[id]))
                ShowStatus(&g_empires[id]);
            break;
        case 6:
            if (pl != -1 && id != -1) {
                Player p;
                PlayerLoad(&p, pl);
                ReadMessages(&p, &g_empires[id]);
                PlayerSave(&p);
            } else {
                ReadPublicMessages();
            }
            if (def == 6)
                def = -1;
            break;
        case 7:
            if (pl != -1 && id != -1)
                SendMessage(id, String(g_empires[id].name));
            else
                SendMessage(-1, String("Visitor: ") + g_door.name);
            break;
        case 8:
            ScoreListAll(g);
            ShowMasters(g);
            if (def == 8)
                def = -1;
            break;
        case 9: {
            Player p;
            PlayerLoad(&p, pl);
            TreatyMessages(&p, &g_empires[id]);
            DiplomacyMenu(&p, &g_empires[id]);
            PlayerSave(&p);
            break;
        }
        case 10: JoinGame(g); break;
        case 11: TournamentResults(g); break;
        }
    } while (r != -1);
}
