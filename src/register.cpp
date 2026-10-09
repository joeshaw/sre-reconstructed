// Registration keys (segment 68a3).
#include "sre.h"

static const char *KEY_FILE = "STATUS.SRE";                         // DS:8cb0
static const char *KEY_REGISTERED = "REGISTERED";                   // DS:8cb8
static const char *KEY_BETA = "BETA";                               // DS:8cbc
static const char *DEFAULT_BANK = "First Intergalactic Bank of The Solar Realm";  // 8cdc
const char *g_regname = 0;                                          // DS:8cb4

// 68a3:0000: decode a registration line; 1 if its check letters fail.
static int16 DecodeLine(const char *in, char *out)
{
    char c1 = in[0];
    char c2 = in[1];
    int16 o = 0;
    int16 chk = 0;
    int16 i;
    for (i = 2; in[i + 2] != 0; i += 2) {
        int16 v = (int16)(((int8)in[i] - '0') * 10 + (int8)in[i + 1] - '0');
        v += (int16)(((int8)c1 - 32) - ((int8)c2 - 32));
        while (v < 0)
            v += 100;
        v %= 100;
        c1 = c2;
        c2 = (char)(v + 32);
        out[o++] = (char)(v + 32);
        chk = (int16)((chk * 3 + v) % 676);
    }
    out[o] = 0;
    if ((int8)in[i] != chk / 26 + 'A' || (int8)in[i + 1] != chk % 26 + 'A') {
        out[0] = 0;
        return 1;
    }
    return 0;
}

// 68a3:0124: read STATUS.SRE.
void CheckRegistration()
{
    System *s = &g_system;
    char line[0x100];
    char key[0x100];
    IFStream f(KEY_FILE, 0);
    int16 kind = 0;
    s->regstate = 0;
    strcpy(s->bank, DEFAULT_BANK);
    if (f.fail())
        return;
    f >> line;
    if (line[0] == 'A' && line[1] == 'B')
        kind = 2;
    if (f.fail() || DecodeLine(line, s->regto))
        return;
    if (line[0] == 'H' && line[1] == 'G')
        kind = 1;
    f >> line;
    if (f.fail() || DecodeLine(line, s->bank)) {
        strcpy(s->bank, DEFAULT_BANK);
        return;
    }
    g_regname = s->regto;
    f >> line;
    if (f.fail() || DecodeLine(line, key)) {
        strcpy(s->bank, DEFAULT_BANK);
        return;
    }
    if (strcmp(key, KEY_REGISTERED) != 0 && strcmp(key, KEY_BETA) != 0) {
        strcpy(s->bank, DEFAULT_BANK);
        return;
    }
    if (kind == 1 && s->bank[0] == 'M' && b_tolower((int)(int8)s->bank[1]) == 'y' && s->regto[0] == 'G' &&
        s->regto[5] == 'P')
        key[3] = 0;
    if (kind == 2 && s->regto[0] == 'W' && b_tolower((int)(int8)s->regto[1]) == 'a' &&
        s->regto[7] == 'J' && s->regto[13] == 'K')
        key[3] = 0;
    s->regstate = (key[(int8)key[0] - 0x4f] == 'I') ? 2 : 0;
    if (strcmp(key, KEY_BETA) == 0)
        s->regstate = 4;
}

// 68a3:039f: read a registration line, colouring as it is typed.
static void ReadKeyLine(char *buf)
{
    int16 n = 0;
    char c;
    buf[0] = 0;
    for (;;) {
        kin >> c;
        if (c >= 'a' && c <= 'z')
            c = (char)(c + 'A' - 'a');
        if (c == 8) {
            if (n > 0) {
                n--;
                buf[n] = 0;
                kout << "\b \b";
            }
        } else if ((c >= '0' && c <= '9' && n >= 2) || (c >= 'A' && c <= 'Z')) {
            if (n < 2)
                kout << Color(15);
            else if (((n - 2) / 3) & 1)
                kout << Color(11);
            else if (c >= 'A' && c <= 'Z')
                kout << Color(10);
            else
                kout << Color(14);
            kout << c;
            buf[n++] = c;
            buf[n] = 0;
        }
        if (c == '\n' || c == '\r') {
            kout << '\n';
            return;
        }
    }
}

// 68a3:0500: enter the registration lines (sysop).
int16 Register()
{
    System *s = &g_system;
    char lines[4][0x50];
    char in[0x100];
    OFStream f(KEY_FILE, 0);
    kout << Color(14) << "\nRegistration of the new SRE\n ";
    HLine(Color(12), 0x4e);
    for (int16 i = 1; i <= 3; i++) {
        kout << Color(7) << "Registration Line #" << Color(11) << i << c_normal << ":  ";
        ReadKeyLine(in);
        int16 bad = DecodeLine(in, lines[i]);
        if (!bad && i == 3)
            bad = strcmp(lines[3], KEY_REGISTERED) != 0 && strcmp(lines[3], KEY_BETA) != 0;
        if (bad) {
            kout << Color(14) << "\nInvalid registration line.\n";
            kout << c_normal << "SRE is currently unregistered.\n";
            s->regstate = 0;
            return 0;
        }
        f << in << '\n';
    }
    kout << "\n ";
    HLine(Color(12), 0x4e);
    s->regstate = 2;
    if (strcmp(lines[3], KEY_BETA) == 0)
        s->regstate = 4;
    kout << c_normal << "This program is registered to:  " << Color(14) << lines[1] << '\n';
    kout << c_normal << "Bank name is:  \"" << Color(11) << lines[2] << c_normal << "\"\n";
    kout << "\nThank you for registering this program!\n";
    return 1;
}
