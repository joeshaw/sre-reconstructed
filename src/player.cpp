// Players (segment 64d2): one record per BBS user in Data/Player.II.
#include "sre.h"

static const char *PLAYER_FILE = "Data/Player.II";      // DS:8094

// 64d2:0253
void PlayerReset(Player *p)
{
    p->index = -1;
    p->user[0] = 0;
    p->name[0] = 0;
    p->b42[0] = 0;
    p->male = 0;
    p->prefs1 &= ~0x0f;
    p->prefs1 |= 0xf0;
    p->prefs2 |= 0x03;
    p->prefs2 &= ~0x1c;
    p->prefs2 &= 0x1f;
    p->w6e = 0;
    p->lastplay = g_now;
    p->created = g_now;
    p->w70 = 0;
}

// 64d2:0000
void PlayerLoad(Player *p, int16 index)
{
    p->created = b_time(0);
    p->lastplay = b_time(0);
    PlayerReset(p);
    if (index == -1)
        return;
    DataFile f(PLAYER_FILE, 0x72, 0, 1);
    if ((int32)index > f.numrecs()) {
        g_errlog << "Tried to open non-existent player #" << index << "!\n";
        PlayerReset(p);
        return;
    }
    if (!f.read(index, p, 1)) {
        if ((int32)index != f.numrecs())
            g_errlog << "Error reading player #" << index << "!\n";
        PlayerReset(p);
        p->index = index;
    }
    if (p->name[0] == 0) {
        PlayerReset(p);
        p->index = index;
    }
}

// 64d2:0183
void PlayerSave(Player *p)
{
    if (p->index == -1)
        return;
    DataFile f(PLAYER_FILE, 0x72, 0, 1);
    if ((int32)p->index > f.numrecs())
        g_errlog << "Error writing invalid player #" << p->index << "!\n";
    else
        f.write(p->index, p, 1);
}

// 64d2:0320
void PlayerSetName(Player *p, const String &s)
{
    strncpy(p->name, s.c_str(), 0x1f);
    p->name[0x1f] = 0;
}

// 64d2:035a
void PlayerSetUser(Player *p, const String &s)
{
    strncpy(p->user, s.c_str(), 0x1f);
    p->user[0x1f] = 0;
}

// 64d2:0394: he/him/his (type 0/1/2) for male, she/her/her otherwise.
String Pronoun(int16 type, int16 male)
{
    const char *s;
    if (type == 0)
        s = male == 1 ? "he" : "she";
    else if (type == 1)
        s = male == 1 ? "him" : "her";
    else if (type == 2)
        s = male == 1 ? "his" : "her";
    else {
        g_errlog << "Unknown pronountype in Pronoun(" << type << ")\n";
        s = "??it??";
    }
    return String(s);
}

// 64d2:041f
String PlayerPronoun(Player *p, int16 type)
{
    return Pronoun(type, p->male);
}

// 64d2:0443
String RandomPronoun(int16 type)
{
    return Pronoun(type, (int16)(((int32)b_rand() << 1) / 0x8000));
}

// 64d2:0479: the record number of a BBS user, or -1.
int16 PlayerFind(const String &user)
{
    Player p;
    PlayerLoad(&p, -1);
    DataFile f(PLAYER_FILE, 0x72, 0, 1);
    int32 n = f.numrecs();
    for (int32 i = 0; i < n; i++) {
        f.read(i, &p, 1);
        if (StrEqI(user, String(p.user))) {
            p.index = -1;
            return (int16)i;
        }
    }
    p.index = -1;
    return -1;
}

// 64d2:05d6: the first free record.
int16 PlayerFree()
{
    Player p;
    PlayerLoad(&p, -1);
    DataFile f(PLAYER_FILE, 0x72, 0, 1);
    int32 n = f.numrecs();
    int32 i;
    for (i = 0; i < n; i++) {
        f.read(i, &p, 1);
        if (p.name[0] == 0)
            break;
    }
    p.index = -1;
    return (int16)i;
}

// 64d2:069c: list all players (sysop).
void PlayerList()
{
    Player p;
    PlayerLoad(&p, -1);
    DataFile f(PLAYER_FILE, 0x72, 0, 1);
    int32 n = f.numrecs();
    for (int32 i = 0; i < n; i++) {
        f.read(i, &p, 1);
        if (p.name[0] == 0)
            continue;
        kout << setw(5) << c_number << (int32)(i + 1) << setw(0) << ' ';
        kout << setw(0x20) << Color(14) << p.user;
        kout << setw(0x20) << Color(15) << p.name << setw(0) << '\n';
    }
    p.index = -1;
}

// 64d2:090c
void PlayerInitIndex(Player *p, int16 index)
{
    PlayerReset(p);
    p->index = index;
}

// 64d2:0926
void PlayerWhoAmI(Player *p)
{
    kout << c_normal << "You are " << String(p->male ? "Emperor" : "Emperess").c_str();
    kout << ' ' << Color(15) << String(p->name).c_str() << c_normal << ".\n";
}
