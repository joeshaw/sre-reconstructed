// Screens, prompts and text files (segments 43e0 and 7389).
#include "screens.h"
#include "devices.h"
#include "menu.h"
#include "resource.h"
#include "translate.h"
#include "files.h"
#include "brtl.h"

String g_version("0.994b");
String g_copyright("");
String g_flavordir;
char g_answer;
int16 g_sbcolor = 2;            // DS:a6e6 StatusBar.Background
int16 g_statusbars = 1;         // DS:a6e8

static Resource r_gamename("StatusBar.Game.Name", "");          // 1d64
static Resource r_intlask("Output.InternationalAsk", (int32)0);  // 1d70

// 43e0:0000
void ClearScreen()
{
    int16 m = kout.mode;
    kout.SetMode(1);
    if (!g_ansi)
        kout << '\f';
    else
        kout << "\x1b[0m\x1b[2J";
    kout.SetMode(m);
}

// 43e0:0056
void Centered(int16 fg, const String &s)
{
    for (int16 i = s.length() / 2 + 0x23; i < 0x46; i++)
        kout << ' ';
    kout << Color(fg);
    PutUntranslated(kout, s);
    kout << '\n';
}

// 43e0:00da
void TitleScreen()
{
    int16 savedlog = kout.logon;
    if (kout.logopen)
        kout.logon = 0;
    ClearScreen();
    if (ShowScreen(0, "Title", 0)) {
        kout << Color(g_sbcolor) << '\xae' << Color(g_sbcolor + 8) << '\xae' << Color(14) << "SRE:II "
             << Color(15) << g_version.c_str() << Color(g_sbcolor + 8) << '\xaf' << Color(g_sbcolor)
             << '\xaf' << c_normal;
        kout << "\nReleased on " << Color(11) << "Mar 07 1995" << Color(3) << '\xf9' << "17:40:37"
             << c_normal << '\n';
    } else {
        HLine(Color(g_sbcolor), 0x46);
        Centered(15, "Solar Realms Elite for DOS");
        Centered(14, "Interstellar Imperialists");
        Centered(2, "By");
        Centered(10, "Amit Patel");
        Centered(2, "at The Solar Realm");
        Centered(3, String("Version ") + g_version + ": released on Mar 07 1995\xfa" "17:40:37");
        HLine(Color(g_sbcolor), 0x46);
    }
    PutUntranslated(kout, g_copyright);
    Pause();
    if (kout.logopen)
        kout.logon = savedlog;
}

// 43e0:042c: the sysop's status line; not part of the user's view.
void StatusBar(const char *who, const char *what, char c)
{
    (void)who;
    (void)what;
    (void)c;
}

// 43e0:08a4
int16 ShowFileBase(String base, int16 paged)
{
    String ans = base + ".ANS";
    String msg = base + ".MSG";
    String txt = base + ".TXT";
    int16 savedpage = g_pagelen;
    g_pagelen = 0;
    int16 savedraw = kout.raw;
    int16 savedmode = kout.mode;
    if (!paged)
        kout.raw = 1;
    kout.SetMode(1);
    if (g_ansi && g_ibm && DisplayFile(ans.c_str(), 0)) {
        g_pagelen = savedpage;
        if (!savedraw)
            kout.raw = 0;
        kout.SetMode(savedmode);
        return 1;
    }
    g_pagelen = savedpage;
    if (!savedraw)
        kout.raw = 0;
    kout.SetMode(savedmode);
    if (DisplayFile(msg.c_str(), 0) || DisplayFile(txt.c_str(), 0))
        return 1;
    return 0;
}

// 43e0:0aa1
int16 ShowScreen(const char *dir, String name, int16 paged)
{
    if (!dir)
        dir = "Screens/";
    if (ShowFileBase(g_flavordir + name, paged))
        return 1;
    if (ShowFileBase(String(dir) + name, paged))
        return 1;
    return 0;
}

// 43e0:0b60
int16 YesNo(int16 def, const char *question)
{
    kout << setw(0);
    kout << (String(question) + "?  ").c_str() << c_bracket << '[' << c_bold;
    if (r_intlask.Bool()) {
        if (def == 1)
            kout << '1' << Color(kout.newfg & 7) << "/0";
        else
            kout << '0' << Color(kout.newfg & 7) << "/1";
    } else if (def == 2) {
        kout << "y/n";
    } else if (def == 1) {
        kout << "Y/n";
    } else {
        kout << "y/N";
    }
    kout << c_bracket << ']' << c_normal << "  ";
    int16 savedlog = kout.logon;
    if (kout.logopen)
        kout.logon = 0;
    for (;;) {
        kout << c_normal << ' ';
        for (int16 spin = 0; g_effects && !kin.KeyReady() && spin < 20000; spin++) {
            kout << Color(11) << "\b<";
            b_delay(50);
            kout << Color(14) << "\bv";
            b_delay(50);
            kout << Color(10) << "\b>";
            b_delay(50);
            kout << Color(9) << "\b^";
            b_delay(50);
            kout << "\b ";
        }
        kout << '\b' << c_input;
        kin >> g_answer;
        g_answer = (char)b_toupper((int)(int8)g_answer);
        if (g_answer == '1')
            g_answer = 'Y';
        if (g_answer == '0')
            g_answer = 'N';
        if (g_answer == '\r') {
            if (def == 1)
                g_answer = 'Y';
            else if (def == 0)
                g_answer = 'N';
        }
        if (g_answer == 'Y' || g_answer == 'N')
            break;
    }
    if (kout.logopen)
        kout.logon = savedlog;
    if (r_intlask.Bool()) {
        if (g_answer == 'Y')
            kout << (int16)1;
        else
            kout << (int16)0;
    } else if (g_answer == 'Y') {
        kout << "Yes";
    } else {
        kout << "No";
    }
    kout << c_normal << "\n";
    return g_answer == 'Y';
}

// 43e0:0f44
void HLine(Color c, int16 n)
{
    kout << c;
    String s;
    for (int16 i = 0; i < n; i++)
        s += '\xc4';
    kout << s.c_str() << c_normal << '\n';
}

// ---------------------------------------------------------------------
static int16 HexDigit(char c)
{
    if (c > 0x40)
        return (int16)((int8)c - 55);
    return (int16)((int8)c - 48);
}

// 7389:063a
void PutCoded(const String &s, OStream &o)
{
    int16 start = 0;
    for (int16 i = 0; i < s.length(); i++) {
        if (s.at(i) != '^')
            continue;
        if (!(i + 3 < s.length()))
            continue;
        if (s.at(i + 1) != '\\')
            continue;
        int16 bg = HexDigit(s.at(i + 2));
        int16 fg = HexDigit(s.at(i + 3));
        o << s.substr(start, i - start).c_str() << Color(fg, bg);
        start = i + 4;
    }
    o << s.substr(start, -1).c_str();
}

// 7389:07c6
int16 DisplayFile(const char *name, uint8 key)
{
    IFStream f(name, 1);
    if (f.fail())
        return 0;
    String line;
    while (!f.fail()) {
        char c;
        f >> c;
        c ^= (char)key;
        if (f.fail())
            break;
        line += c;
        if (c == '\n' && kin.KeyReady()) {
            char k;
            kin >> k;
            if (k == ' ') {
                kout << Color(4) << "\nAborted\n";
                kout << Color(7);
                return 1;
            }
        }
        if (c == '\n') {
            PutCoded(line, kout);
            line = "";
        }
    }
    PutCoded(line, kout);
    kout << Color(7);
    return 1;
}
